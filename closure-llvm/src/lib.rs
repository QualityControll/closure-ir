use std::marker::PhantomData;

use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{BasicTypeEnum, StructType},
    values::{BasicValueEnum, PointerValue},
    AddressSpace,
    OptimizationLevel,
};

pub use closure_llvm_macro::{
    call,
    compile_closure,
    CompileType,
};


// ============================================================
// Type information
// ============================================================

#[derive(Debug, Clone)]
pub enum TypeInfo {
    F64,
    I32,
    Bool,

    Struct {
        name: String,
        fields: Vec<FieldInfo>,
    },
}


#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_info: TypeInfo,
}


// ============================================================
// CompileType
// ============================================================

pub trait CompileType {
    fn type_info() -> TypeInfo;

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx>;
}


// ============================================================
// Primitive CompileType implementations
// ============================================================

impl CompileType for f64 {
    fn type_info() -> TypeInfo {
        TypeInfo::F64
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.f64_type().into()
    }
}


impl CompileType for i32 {
    fn type_info() -> TypeInfo {
        TypeInfo::I32
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i32_type().into()
    }
}


impl CompileType for bool {
    fn type_info() -> TypeInfo {
        TypeInfo::Bool
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.bool_type().into()
    }
}


// ============================================================
// Runtime values
// ============================================================

#[derive(Debug)]
pub enum Value {
    F64(f64),
    I32(i32),
    Bool(bool),
}


// ============================================================
// Expression IR
// ============================================================

#[derive(Debug)]
pub enum Expr {
    Argument(usize),

    Constant(Value),

    Field {
        object: Box<Expr>,
        name: String,
    },

    Add {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Sub {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Mul {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Div {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}


// ============================================================
// Closure description
// ============================================================

pub struct Closure {
    pub arguments: Vec<TypeInfo>,
    pub return_type: TypeInfo,
    pub body: Expr,
}


// ============================================================
// Compiled closure
// ============================================================

pub struct CompiledClosure<'ctx, Args, Ret> {
    engine: ExecutionEngine<'ctx>,
    function_name: String,

    _marker: PhantomData<fn(Args) -> Ret>,
}


impl<'ctx, Args, Ret>
    CompiledClosure<'ctx, Args, Ret>
{
    fn new(
        engine: ExecutionEngine<'ctx>,
        function_name: String,
    ) -> Self {
        Self {
            engine,
            function_name,
            _marker: PhantomData,
        }
    }


    // --------------------------------------------------------
    // Call the JIT function
    // --------------------------------------------------------

    pub unsafe fn call(
        &self,
        value: &Args,
    ) -> Ret
    where
        Args: CompileType,
        Ret: CompileType,
    {
        jit_call::<Args, Ret>(
            &self.engine,
            &self.function_name,
            value,
        )
    }
}


// ============================================================
// Generic JIT invocation
// ============================================================

unsafe fn jit_call<Args, Ret>(
    engine: &ExecutionEngine<'_>,
    function_name: &str,
    value: &Args,
) -> Ret
where
    Args: CompileType,
    Ret: CompileType,
{
    let address =
        engine
            .get_function_address(
                function_name,
            )
            .expect(
                "failed to get JIT function address",
            );


    type JitFn<Ret> =
        unsafe extern "C" fn(
            *const u8,
        ) -> Ret;


    let function:
        JitFn<Ret> =
        std::mem::transmute(
            address,
        );


    function(
        value as *const Args
            as *const u8,
    )
}


// ============================================================
// Compiler
// ============================================================

pub struct Compiler<'ctx> {
    context: &'ctx Context,
}


impl<'ctx> Compiler<'ctx> {
    pub fn new(
        context: &'ctx Context,
    ) -> Self {
        Self {
            context,
        }
    }


    // --------------------------------------------------------
    // Compile
    // --------------------------------------------------------

    pub fn compile<Args, Ret>(
        &self,
        closure: &Closure,
    ) -> Result<
        CompiledClosure<'ctx, Args, Ret>,
        String,
    >
    where
        Args: CompileType,
        Ret: CompileType,
    {
        if closure.arguments.len() != 1 {
            return Err(
                "only one closure argument is currently supported"
                    .to_string(),
            );
        }


        let module =
            self.context.create_module(
                "closure_module",
            );


        let function_name =
            "compiled_closure";


        self.generate_function(
            &module,
            function_name,
            closure,
        )?;


        println!(
            "Generated LLVM IR:\n{}",
            module
                .print_to_string()
                .to_string()
        );


        let engine =
            module
                .create_jit_execution_engine(
                    OptimizationLevel::None,
                )
                .map_err(|error| {
                    format!(
                        "failed to create JIT: {:?}",
                        error
                    )
                })?;


        Ok(
            CompiledClosure::new(
                engine,
                function_name.to_string(),
            )
        )
    }


    // --------------------------------------------------------
    // Generate LLVM function
    // --------------------------------------------------------

    fn generate_function(
        &self,
        module: &Module<'ctx>,
        function_name: &str,
        closure: &Closure,
    ) -> Result<(), String> {
        if closure.arguments.len() != 1 {
            return Err(
                "only one closure argument is currently supported"
                    .to_string(),
            );
        }


        let return_type =
            llvm_type(
                self.context,
                &closure.return_type,
            )?;


        // LLVM 15+ uses opaque pointers.
        let argument_pointer_type =
            self.context.ptr_type(
                AddressSpace::default(),
            );


        // ----------------------------------------------------
        // Inkwell 0.10.0:
        //
        // BasicTypeEnum itself does not provide fn_type().
        //
        // We must unwrap the concrete BasicTypeEnum variant
        // and call fn_type() on that concrete type.
        // ----------------------------------------------------

        let function_type =
            match return_type {

                BasicTypeEnum::FloatType(
                    ty
                ) => {
                    ty.fn_type(
                        &[
                            argument_pointer_type
                                .into(),
                        ],
                        false,
                    )
                }


                BasicTypeEnum::IntType(
                    ty
                ) => {
                    ty.fn_type(
                        &[
                            argument_pointer_type
                                .into(),
                        ],
                        false,
                    )
                }


                BasicTypeEnum::StructType(
                    ty
                ) => {
                    ty.fn_type(
                        &[
                            argument_pointer_type
                                .into(),
                        ],
                        false,
                    )
                }


                _ => {
                    return Err(
                        "unsupported return type for JIT function"
                            .to_string(),
                    );
                }
            };


        let function =
            module.add_function(
                function_name,
                function_type,
                None,
            );


        let entry =
            self.context.append_basic_block(
                function,
                "entry",
            );


        let builder =
            self.context.create_builder();


        builder.position_at_end(
            entry,
        );


        let argument =
            function
                .get_nth_param(0)
                .ok_or_else(|| {
                    "missing function argument"
                        .to_string()
                })?
                .into_pointer_value();


        let arguments =
            vec![argument];


        let value =
            self.lower_expr(
                &builder,
                &arguments,
                &closure.arguments,
                &closure.body,
            )?;


        let value =
            self.materialize_value(
                &builder,
                value,
            )?;


        builder
            .build_return(
                Some(&value),
            )
            .map_err(|error| {
                format!(
                    "failed to build return: {:?}",
                    error,
                )
            })?;


        if function.verify(true) {
            Ok(())
        } else {
            Err(
                "LLVM function verification failed"
                    .to_string(),
            )
        }
    }


    // --------------------------------------------------------
    // Lower expression
    // --------------------------------------------------------

    fn lower_expr(
        &self,
        builder: &Builder<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expr: &Expr,
    ) -> Result<
        LoweredValue<'ctx>,
        String,
    > {
        match expr {

            // ------------------------------------------------
            // Argument
            // ------------------------------------------------

            Expr::Argument(index) => {
                let pointer =
                    *arguments
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "argument index {} out of bounds",
                                index,
                            )
                        })?;


                let type_info =
                    argument_types
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "argument type index {} out of bounds",
                                index,
                            )
                        })?
                        .clone();


                Ok(
                    LoweredValue::Pointer {
                        pointer,
                        type_info,
                    }
                )
            }


            // ------------------------------------------------
            // Constant
            // ------------------------------------------------

            Expr::Constant(value) => {
                match value {

                    Value::F64(value) => {
                        Ok(
                            LoweredValue::Value(
                                self.context
                                    .f64_type()
                                    .const_float(
                                        *value,
                                    )
                                    .into(),
                            )
                        )
                    }


                    Value::I32(value) => {
                        Ok(
                            LoweredValue::Value(
                                self.context
                                    .i32_type()
                                    .const_int(
                                        *value as u64,
                                        true,
                                    )
                                    .into(),
                            )
                        )
                    }


                    Value::Bool(value) => {
                        Ok(
                            LoweredValue::Value(
                                self.context
                                    .bool_type()
                                    .const_int(
                                        if *value {
                                            1
                                        } else {
                                            0
                                        },
                                        false,
                                    )
                                    .into(),
                            )
                        )
                    }
                }
            }


            // ------------------------------------------------
            // Field
            // ------------------------------------------------

            Expr::Field {
                object,
                name,
            } => {
                let object =
                    self.lower_expr(
                        builder,
                        arguments,
                        argument_types,
                        object,
                    )?;


                let (
                    object_pointer,
                    object_type,
                ) =
                    match object {

                        LoweredValue::Pointer {
                            pointer,
                            type_info,
                        } => {
                            (
                                pointer,
                                type_info,
                            )
                        }


                        LoweredValue::Value(_) => {
                            return Err(
                                format!(
                                    "cannot access field `{}` on a value",
                                    name,
                                )
                            );
                        }
                    };


                let fields =
                    match &object_type {

                        TypeInfo::Struct {
                            fields,
                            ..
                        } => fields,


                        _ => {
                            return Err(
                                format!(
                                    "cannot access field `{}` on non-struct type",
                                    name,
                                )
                            );
                        }
                    };


                let (
                    field_index,
                    field_type,
                ) =
                    fields
                        .iter()
                        .enumerate()
                        .find_map(
                            |(index, field)| {
                                if field.name == *name {
                                    Some((
                                        index,
                                        field.type_info
                                            .clone(),
                                    ))
                                } else {
                                    None
                                }
                            }
                        )
                        .ok_or_else(|| {
                            format!(
                                "field `{}` not found",
                                name,
                            )
                        })?;


                let struct_type =
                    llvm_struct_type(
                        self.context,
                        &object_type,
                    )?;


                let field_pointer =
                    builder
                        .build_struct_gep(
                            struct_type,
                            object_pointer,
                            field_index as u32,
                            &format!(
                                "{}_ptr",
                                name,
                            ),
                        )
                        .map_err(|error| {
                            format!(
                                "failed to build GEP for field `{}`: {:?}",
                                name,
                                error,
                            )
                        })?;


                Ok(
                    LoweredValue::Pointer {
                        pointer:
                            field_pointer,

                        type_info:
                            field_type,
                    }
                )
            }


            // ------------------------------------------------
            // Add
            // ------------------------------------------------

            Expr::Add {
                lhs,
                rhs,
            } => {
                self.lower_float_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Add,
                )
            }


            // ------------------------------------------------
            // Sub
            // ------------------------------------------------

            Expr::Sub {
                lhs,
                rhs,
            } => {
                self.lower_float_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Sub,
                )
            }


            // ------------------------------------------------
            // Mul
            // ------------------------------------------------

            Expr::Mul {
                lhs,
                rhs,
            } => {
                self.lower_float_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Mul,
                )
            }


            // ------------------------------------------------
            // Div
            // ------------------------------------------------

            Expr::Div {
                lhs,
                rhs,
            } => {
                self.lower_float_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Div,
                )
            }
        }
    }


    // --------------------------------------------------------
    // Floating point binary operation
    // --------------------------------------------------------

    fn lower_float_binary(
        &self,
        builder: &Builder<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        lhs: &Expr,
        rhs: &Expr,
        operation: BinaryOp,
    ) -> Result<
        LoweredValue<'ctx>,
        String,
    > {
        let lhs =
            self.lower_expr(
                builder,
                arguments,
                argument_types,
                lhs,
            )?;


        let lhs =
            self.materialize_value(
                builder,
                lhs,
            )?;


        let rhs =
            self.lower_expr(
                builder,
                arguments,
                argument_types,
                rhs,
            )?;


        let rhs =
            self.materialize_value(
                builder,
                rhs,
            )?;


        let lhs =
            lhs.into_float_value();


        let rhs =
            rhs.into_float_value();


        let result =
            match operation {

                BinaryOp::Add => {
                    builder
                        .build_float_add(
                            lhs,
                            rhs,
                            "add",
                        )
                        .map_err(|error| {
                            format!(
                                "failed to build fadd: {:?}",
                                error,
                            )
                        })?
                }


                BinaryOp::Sub => {
                    builder
                        .build_float_sub(
                            lhs,
                            rhs,
                            "sub",
                        )
                        .map_err(|error| {
                            format!(
                                "failed to build fsub: {:?}",
                                error,
                            )
                        })?
                }


                BinaryOp::Mul => {
                    builder
                        .build_float_mul(
                            lhs,
                            rhs,
                            "mul",
                        )
                        .map_err(|error| {
                            format!(
                                "failed to build fmul: {:?}",
                                error,
                            )
                        })?
                }


                BinaryOp::Div => {
                    builder
                        .build_float_div(
                            lhs,
                            rhs,
                            "div",
                        )
                        .map_err(|error| {
                            format!(
                                "failed to build fdiv: {:?}",
                                error,
                            )
                        })?
                }
            };


        Ok(
            LoweredValue::Value(
                result.into(),
            )
        )
    }


    // --------------------------------------------------------
    // Materialize pointer -> value
    // --------------------------------------------------------

    fn materialize_value(
        &self,
        builder: &Builder<'ctx>,
        value: LoweredValue<'ctx>,
    ) -> Result<
        BasicValueEnum<'ctx>,
        String,
    > {
        match value {

            LoweredValue::Value(value) => {
                Ok(value)
            }


            LoweredValue::Pointer {
                pointer,
                type_info,
            } => {
                let llvm_type =
                    llvm_type(
                        self.context,
                        &type_info,
                    )?;


                builder
                    .build_load(
                        llvm_type,
                        pointer,
                        "load",
                    )
                    .map_err(|error| {
                        format!(
                            "failed to build load: {:?}",
                            error,
                        )
                    })
            }
        }
    }
}


// ============================================================
// Lowered value
// ============================================================

enum LoweredValue<'ctx> {
    Value(
        BasicValueEnum<'ctx>
    ),

    Pointer {
        pointer:
            PointerValue<'ctx>,

        type_info:
            TypeInfo,
    },
}


// ============================================================
// Binary operations
// ============================================================

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}


// ============================================================
// TypeInfo -> LLVM type
// ============================================================

fn llvm_type<'ctx>(
    context: &'ctx Context,
    type_info: &TypeInfo,
) -> Result<
    BasicTypeEnum<'ctx>,
    String,
> {
    match type_info {

        TypeInfo::F64 => {
            Ok(
                context
                    .f64_type()
                    .into()
            )
        }


        TypeInfo::I32 => {
            Ok(
                context
                    .i32_type()
                    .into()
            )
        }


        TypeInfo::Bool => {
            Ok(
                context
                    .bool_type()
                    .into()
            )
        }


        TypeInfo::Struct {
            fields,
            ..
        } => {
            let field_types =
                fields
                    .iter()
                    .map(|field| {
                        llvm_type(
                            context,
                            &field.type_info,
                        )
                    })
                    .collect::<
                        Result<
                            Vec<_>,
                            _,
                        >
                    >()?;


            Ok(
                context
                    .struct_type(
                        &field_types,
                        false,
                    )
                    .into()
            )
        }
    }
}


// ============================================================
// TypeInfo -> StructType
// ============================================================

fn llvm_struct_type<'ctx>(
    context: &'ctx Context,
    type_info: &TypeInfo,
) -> Result<
    StructType<'ctx>,
    String,
> {
    match llvm_type(
        context,
        type_info,
    )? {

        BasicTypeEnum::StructType(
            struct_type
        ) => {
            Ok(struct_type)
        }


        _ => {
            Err(
                "expected struct LLVM type"
                    .to_string()
            )
        }
    }
}