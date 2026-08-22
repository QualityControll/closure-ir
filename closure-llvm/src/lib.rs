use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{
        BasicMetadataTypeEnum,
        BasicTypeEnum,
        FunctionType,
    },
    values::{
        BasicValueEnum,
        FloatValue,
        IntValue,
        PointerValue,
    },
    AddressSpace,
    OptimizationLevel,
};

pub use closure_llvm_macro::{
    compile_closure,
    CompileType,
};


// ============================================================
// Type information
// ============================================================

#[derive(Debug, Clone)]
pub enum TypeInfo {
    I32,
    I64,
    F32,
    F64,
    Bool,

    Struct {
        name: &'static str,
        fields: &'static [FieldInfo],
    },
}


#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: &'static str,

    pub type_info: fn() -> TypeInfo,
}


impl FieldInfo {
    pub fn ty(&self) -> TypeInfo {
        (self.type_info)()
    }
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
// Primitive types
// ============================================================

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


impl CompileType for i64 {
    fn type_info() -> TypeInfo {
        TypeInfo::I64
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i64_type().into()
    }
}


impl CompileType for f32 {
    fn type_info() -> TypeInfo {
        TypeInfo::F32
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.f32_type().into()
    }
}


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
// TypeInfo -> LLVM
// ============================================================

impl TypeInfo {
    pub fn llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        match self {
            TypeInfo::I32 => context.i32_type().into(),

            TypeInfo::I64 => context.i64_type().into(),

            TypeInfo::F32 => context.f32_type().into(),

            TypeInfo::F64 => context.f64_type().into(),

            TypeInfo::Bool => context.bool_type().into(),

            TypeInfo::Struct { fields, .. } => {
                let llvm_fields: Vec<BasicTypeEnum<'ctx>> =
                    fields
                        .iter()
                        .map(|field| {
                            field.ty().llvm_type(context)
                        })
                        .collect();

                context
                    .struct_type(&llvm_fields, false)
                    .into()
            }
        }
    }
}


// ============================================================
// Expression IR
// ============================================================

#[derive(Debug, Clone)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
}


#[derive(Debug, Clone)]
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

    Neg {
        value: Box<Expr>,
    },
}


// ============================================================
// Closure IR
// ============================================================

#[derive(Debug, Clone)]
pub struct Closure {
    pub arguments: Vec<TypeInfo>,

    pub return_type: TypeInfo,

    pub body: Expr,
}


// ============================================================
// Compiler
// ============================================================

pub struct Compiler<'ctx> {
    context: &'ctx Context,
}


pub struct CompiledClosure<'ctx> {
    pub engine: ExecutionEngine<'ctx>,

    pub function_name: String,
}


impl<'ctx> Compiler<'ctx> {
    pub fn new(
        context: &'ctx Context,
    ) -> Self {
        Self { context }
    }


    // ========================================================
    // Argument ABI
    // ========================================================

    fn llvm_argument_type(
        &self,
        ty: &TypeInfo,
    ) -> BasicMetadataTypeEnum<'ctx> {
        match ty {
            TypeInfo::Struct { .. } => {
                self.context
                    .ptr_type(AddressSpace::default())
                    .into()
            }

            TypeInfo::I32 => {
                self.context.i32_type().into()
            }

            TypeInfo::I64 => {
                self.context.i64_type().into()
            }

            TypeInfo::F32 => {
                self.context.f32_type().into()
            }

            TypeInfo::F64 => {
                self.context.f64_type().into()
            }

            TypeInfo::Bool => {
                self.context.bool_type().into()
            }
        }
    }


    // ========================================================
    // Return type
    // ========================================================

    fn llvm_function_type(
        &self,
        return_type: &TypeInfo,
        arguments: &[BasicMetadataTypeEnum<'ctx>],
    ) -> Result<FunctionType<'ctx>, String> {
        match return_type {
            TypeInfo::I32 => {
                Ok(
                    self.context
                        .i32_type()
                        .fn_type(arguments, false)
                )
            }

            TypeInfo::I64 => {
                Ok(
                    self.context
                        .i64_type()
                        .fn_type(arguments, false)
                )
            }

            TypeInfo::F32 => {
                Ok(
                    self.context
                        .f32_type()
                        .fn_type(arguments, false)
                )
            }

            TypeInfo::F64 => {
                Ok(
                    self.context
                        .f64_type()
                        .fn_type(arguments, false)
                )
            }

            TypeInfo::Bool => {
                Ok(
                    self.context
                        .bool_type()
                        .fn_type(arguments, false)
                )
            }

            TypeInfo::Struct { .. } => {
                Err(
                    "struct return values are not yet supported"
                        .to_string()
                )
            }
        }
    }


    // ========================================================
    // Compile closure
    // ========================================================

    pub fn compile(
        &self,
        closure: &Closure,
    ) -> Result<CompiledClosure<'ctx>, String> {
        let module =
            self.context.create_module("closure");

        let builder =
            self.context.create_builder();


        // ----------------------------------------------------
        // Argument types
        // ----------------------------------------------------

        let argument_types:
            Vec<BasicMetadataTypeEnum<'ctx>> =
            closure
                .arguments
                .iter()
                .map(|ty| {
                    self.llvm_argument_type(ty)
                })
                .collect();


        // ----------------------------------------------------
        // Function type
        // ----------------------------------------------------

        let function_type =
            self.llvm_function_type(
                &closure.return_type,
                &argument_types,
            )?;


        let function =
            module.add_function(
                "closure",
                function_type,
                None,
            );


        // ----------------------------------------------------
        // Entry block
        // ----------------------------------------------------

        let entry =
            self.context.append_basic_block(
                function,
                "entry",
            );

        builder.position_at_end(entry);


        // ----------------------------------------------------
        // Arguments
        // ----------------------------------------------------

        let arguments =
            function
                .get_param_iter()
                .collect::<Vec<_>>();


        // ----------------------------------------------------
        // Compile body
        // ----------------------------------------------------

        let result =
            self.compile_expr(
                &builder,
                &closure.body,
                &closure.arguments,
                &arguments,
            )?;


        // ----------------------------------------------------
        // Return
        // ----------------------------------------------------

        builder
            .build_return(Some(&result))
            .map_err(|e| e.to_string())?;


        // ----------------------------------------------------
        // Verify
        // ----------------------------------------------------

        module
            .verify()
            .map_err(|e| e.to_string())?;


        // ----------------------------------------------------
        // Show generated LLVM
        // ----------------------------------------------------

        module.print_to_stderr();


        // ----------------------------------------------------
        // JIT
        // ----------------------------------------------------

        let engine =
            module
                .create_jit_execution_engine(
                    OptimizationLevel::Default,
                )
                .map_err(|e| e.to_string())?;


        Ok(
            CompiledClosure {
                engine,
                function_name: "closure".to_string(),
            }
        )
    }


    // ========================================================
    // Expression lowering
    // ========================================================

    fn compile_expr(
        &self,

        builder: &Builder<'ctx>,

        expr: &Expr,

        argument_types: &[TypeInfo],

        arguments: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match expr {
            Expr::Argument(index) => {
                let argument =
                    arguments
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "invalid argument index {}",
                                index
                            )
                        })?;

                match argument {
                    BasicValueEnum::PointerValue(pointer) => {
                        let ty =
                            argument_types
                                .get(*index)
                                .ok_or_else(|| {
                                    format!(
                                        "missing type for argument {}",
                                        index
                                    )
                                })?;

                        let llvm_type =
                            ty.llvm_type(self.context);

                        builder
                            .build_load(
                                llvm_type,
                                *pointer,
                                &format!("arg_{}", index),
                            )
                            .map_err(|e| e.to_string())
                    }

                    _ => Ok(*argument),
                }
            }


            Expr::Constant(value) => {
                self.compile_constant(value)
            }


            Expr::Field { object, name } => {
                self.compile_field(
                    builder,
                    object,
                    name,
                    argument_types,
                    arguments,
                )
            }


            Expr::Add { lhs, rhs } => {
                self.compile_binary(
                    builder,
                    lhs,
                    rhs,
                    argument_types,
                    arguments,
                    BinaryOp::Add,
                )
            }


            Expr::Sub { lhs, rhs } => {
                self.compile_binary(
                    builder,
                    lhs,
                    rhs,
                    argument_types,
                    arguments,
                    BinaryOp::Sub,
                )
            }


            Expr::Mul { lhs, rhs } => {
                self.compile_binary(
                    builder,
                    lhs,
                    rhs,
                    argument_types,
                    arguments,
                    BinaryOp::Mul,
                )
            }


            Expr::Div { lhs, rhs } => {
                self.compile_binary(
                    builder,
                    lhs,
                    rhs,
                    argument_types,
                    arguments,
                    BinaryOp::Div,
                )
            }


            Expr::Neg { value } => {
                let value =
                    self.compile_expr(
                        builder,
                        value,
                        argument_types,
                        arguments,
                    )?;

                match value {
                    BasicValueEnum::IntValue(value) => {
                        builder
                            .build_int_neg(
                                value,
                                "neg",
                            )
                            .map(|v| v.into())
                            .map_err(|e| e.to_string())
                    }

                    BasicValueEnum::FloatValue(value) => {
                        builder
                            .build_float_neg(
                                value,
                                "neg",
                            )
                            .map(|v| v.into())
                            .map_err(|e| e.to_string())
                    }

                    _ => Err(
                        "negation requires a numeric value"
                            .to_string()
                    ),
                }
            }
        }
    }


    // ========================================================
    // Constants
    // ========================================================

    fn compile_constant(
        &self,
        value: &Value,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            Value::I32(value) => {
                Ok(
                    self.context
                        .i32_type()
                        .const_int(
                            *value as u64,
                            true,
                        )
                        .into()
                )
            }

            Value::I64(value) => {
                Ok(
                    self.context
                        .i64_type()
                        .const_int(
                            *value as u64,
                            true,
                        )
                        .into()
                )
            }

            Value::F32(value) => {
                Ok(
                    self.context
                        .f32_type()
                        .const_float(
                            *value as f64,
                        )
                        .into()
                )
            }

            Value::F64(value) => {
                Ok(
                    self.context
                        .f64_type()
                        .const_float(*value)
                        .into()
                )
            }

            Value::Bool(value) => {
                Ok(
                    self.context
                        .bool_type()
                        .const_int(
                            if *value { 1 } else { 0 },
                            false,
                        )
                        .into()
                )
            }
        }
    }


    // ========================================================
    // Field access
    // ========================================================

    fn compile_field(
        &self,

        builder: &Builder<'ctx>,

        object: &Expr,

        field_name: &str,

        argument_types: &[TypeInfo],

        arguments: &[BasicValueEnum<'ctx>],
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let (
            pointer,
            object_type,
        ) =
            self.compile_lvalue(
                builder,
                object,
                argument_types,
                arguments,
            )?;


        let TypeInfo::Struct {
            fields,
            ..
        } =
            &object_type
        else {
            return Err(
                format!(
                    "cannot access field `{}` on non-struct",
                    field_name
                )
            );
        };


        let (
            index,
            field_type,
        ) =
            fields
                .iter()
                .enumerate()
                .find_map(|(index, field)| {
                    if field.name == field_name {
                        Some((index, field.ty()))
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    format!(
                        "field `{}` not found",
                        field_name
                    )
                })?;


        let struct_type =
            object_type
                .llvm_type(self.context)
                .into_struct_type();


        let field_pointer =
            builder
                .build_struct_gep(
                    struct_type,
                    pointer,
                    index as u32,
                    field_name,
                )
                .map_err(|e| e.to_string())?;


        builder
            .build_load(
                field_type.llvm_type(self.context),
                field_pointer,
                field_name,
            )
            .map_err(|e| e.to_string())
    }


    // ========================================================
    // L-value
    // ========================================================

    fn compile_lvalue(
        &self,

        builder: &Builder<'ctx>,

        expr: &Expr,

        argument_types: &[TypeInfo],

        arguments: &[BasicValueEnum<'ctx>],
    ) -> Result<
        (PointerValue<'ctx>, TypeInfo),
        String,
    > {
        match expr {
            Expr::Argument(index) => {
                let argument =
                    arguments
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "invalid argument index {}",
                                index
                            )
                        })?;


                let pointer =
                    match argument {
                        BasicValueEnum::PointerValue(pointer) => {
                            *pointer
                        }

                        _ => {
                            return Err(
                                format!(
                                    "argument {} is not a pointer",
                                    index
                                )
                            )
                        }
                    };


                let type_info =
                    argument_types
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "missing argument type {}",
                                index
                            )
                        })?
                        .clone();


                Ok((pointer, type_info))
            }


            Expr::Field {
                object,
                name,
            } => {
                let (
                    object_pointer,
                    object_type,
                ) =
                    self.compile_lvalue(
                        builder,
                        object,
                        argument_types,
                        arguments,
                    )?;


                let TypeInfo::Struct {
                    fields,
                    ..
                } =
                    &object_type
                else {
                    return Err(
                        format!(
                            "`{}` is not a struct",
                            name
                        )
                    );
                };


                let (
                    index,
                    field_type,
                ) =
                    fields
                        .iter()
                        .enumerate()
                        .find_map(|(index, field)| {
                            if field.name == name {
                                Some((index, field.ty()))
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            format!(
                                "field `{}` not found",
                                name
                            )
                        })?;


                let struct_type =
                    object_type
                        .llvm_type(self.context)
                        .into_struct_type();


                let field_pointer =
                    builder
                        .build_struct_gep(
                            struct_type,
                            object_pointer,
                            index as u32,
                            name,
                        )
                        .map_err(|e| e.to_string())?;


                Ok((
                    field_pointer,
                    field_type,
                ))
            }


            _ => Err(
                "expression cannot be used as an lvalue"
                    .to_string()
            ),
        }
    }


    // ========================================================
    // Binary operations
    // ========================================================

    fn compile_binary(
        &self,

        builder: &Builder<'ctx>,

        lhs: &Expr,

        rhs: &Expr,

        argument_types: &[TypeInfo],

        arguments: &[BasicValueEnum<'ctx>],

        operation: BinaryOp,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        let lhs =
            self.compile_expr(
                builder,
                lhs,
                argument_types,
                arguments,
            )?;


        let rhs =
            self.compile_expr(
                builder,
                rhs,
                argument_types,
                arguments,
            )?;


        match (lhs, rhs) {
            (
                BasicValueEnum::IntValue(lhs),
                BasicValueEnum::IntValue(rhs),
            ) => {
                let value =
                    match operation {
                        BinaryOp::Add =>
                            builder.build_int_add(
                                lhs,
                                rhs,
                                "add",
                            ),

                        BinaryOp::Sub =>
                            builder.build_int_sub(
                                lhs,
                                rhs,
                                "sub",
                            ),

                        BinaryOp::Mul =>
                            builder.build_int_mul(
                                lhs,
                                rhs,
                                "mul",
                            ),

                        BinaryOp::Div =>
                            builder.build_int_signed_div(
                                lhs,
                                rhs,
                                "div",
                            ),
                    }
                    .map_err(|e| e.to_string())?;

                Ok(value.into())
            }


            (
                BasicValueEnum::FloatValue(lhs),
                BasicValueEnum::FloatValue(rhs),
            ) => {
                let value =
                    match operation {
                        BinaryOp::Add =>
                            builder.build_float_add(
                                lhs,
                                rhs,
                                "add",
                            ),

                        BinaryOp::Sub =>
                            builder.build_float_sub(
                                lhs,
                                rhs,
                                "sub",
                            ),

                        BinaryOp::Mul =>
                            builder.build_float_mul(
                                lhs,
                                rhs,
                                "mul",
                            ),

                        BinaryOp::Div =>
                            builder.build_float_div(
                                lhs,
                                rhs,
                                "div",
                            ),
                    }
                    .map_err(|e| e.to_string())?;

                Ok(value.into())
            }


            _ => Err(
                "binary operands must have the same numeric type"
                    .to_string()
            ),
        }
    }
}


// ============================================================
// Binary operation
// ============================================================

#[derive(Debug, Clone, Copy)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}