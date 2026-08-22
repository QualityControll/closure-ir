use std::marker::PhantomData;

use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{BasicType, BasicTypeEnum, StructType},
    values::{BasicValueEnum, FloatValue, IntValue, PointerValue},
    AddressSpace,
    OptimizationLevel,
};

use inkwell::FloatPredicate;
use inkwell::IntPredicate;

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
    F32,
    F64,

    I8,
    I16,
    I32,
    I64,
    I128,

    U8,
    U16,
    U32,
    U64,
    U128,

    Bool,

    Struct {
        name: String,
        fields: Vec<FieldInfo>,
    },
}

impl TypeInfo {
    pub fn is_float(&self) -> bool {
        matches!(
            self,
            TypeInfo::F32 | TypeInfo::F64
        )
    }

    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            TypeInfo::I8
                | TypeInfo::I16
                | TypeInfo::I32
                | TypeInfo::I64
                | TypeInfo::I128
        )
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            TypeInfo::U8
                | TypeInfo::U16
                | TypeInfo::U32
                | TypeInfo::U64
                | TypeInfo::U128
        )
    }

    pub fn is_integer(&self) -> bool {
        self.is_signed_integer()
            || self.is_unsigned_integer()
            || matches!(self, TypeInfo::Bool)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, TypeInfo::Bool)
    }
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


impl CompileType for i8 {
    fn type_info() -> TypeInfo {
        TypeInfo::I8
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i8_type().into()
    }
}


impl CompileType for i16 {
    fn type_info() -> TypeInfo {
        TypeInfo::I16
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i16_type().into()
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


impl CompileType for i128 {
    fn type_info() -> TypeInfo {
        TypeInfo::I128
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i128_type().into()
    }
}


impl CompileType for u8 {
    fn type_info() -> TypeInfo {
        TypeInfo::U8
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i8_type().into()
    }
}


impl CompileType for u16 {
    fn type_info() -> TypeInfo {
        TypeInfo::U16
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i16_type().into()
    }
}


impl CompileType for u32 {
    fn type_info() -> TypeInfo {
        TypeInfo::U32
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i32_type().into()
    }
}


impl CompileType for u64 {
    fn type_info() -> TypeInfo {
        TypeInfo::U64
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i64_type().into()
    }
}


impl CompileType for u128 {
    fn type_info() -> TypeInfo {
        TypeInfo::U128
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.i128_type().into()
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
    F32(f32),
    F64(f64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),

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

    Rem {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Eq {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Ne {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Lt {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Le {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Gt {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Ge {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    And {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Or {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    BitAnd {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    BitOr {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    BitXor {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Shl {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Shr {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },

    Not {
        operand: Box<Expr>,
    },

    Neg {
        operand: Box<Expr>,
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
// Typed compiled closure
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
            .get_function_address(function_name)
            .expect(
                "failed to get JIT function address",
            );

    type JitFn<Ret> =
        unsafe extern "C" fn(*const u8) -> Ret;

    let function: JitFn<Ret> =
        std::mem::transmute(address);

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
        Self { context }
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

        let argument_pointer_type =
            self.context.ptr_type(
                AddressSpace::default(),
            );

        let function_type =
            match return_type {
                BasicTypeEnum::ArrayType(ty) =>
                    ty.fn_type(
                        &[argument_pointer_type.into()],
                        false,
                    ),

                BasicTypeEnum::FloatType(ty) =>
                    ty.fn_type(
                        &[argument_pointer_type.into()],
                        false,
                    ),

                BasicTypeEnum::IntType(ty) =>
                    ty.fn_type(
                        &[argument_pointer_type.into()],
                        false,
                    ),

                BasicTypeEnum::PointerType(ty) =>
                    ty.fn_type(
                        &[argument_pointer_type.into()],
                        false,
                    ),

                BasicTypeEnum::StructType(ty) =>
                    ty.fn_type(
                        &[argument_pointer_type.into()],
                        false,
                    ),

                BasicTypeEnum::VectorType(ty) =>
                    ty.fn_type(
                        &[argument_pointer_type.into()],
                        false,
                    ),

                BasicTypeEnum::ScalableVectorType(ty) =>
                    ty.fn_type(
                        &[argument_pointer_type.into()],
                        false,
                    ),
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

        builder.position_at_end(entry);

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
            .build_return(Some(&value))
            .map_err(|error| {
                format!(
                    "failed to build return: {:?}",
                    error
                )
            })?;

        if function.verify(true) {
            Ok(())
        } else {
            Err(
                "LLVM function verification failed"
                    .to_string()
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
    ) -> Result<LoweredValue<'ctx>, String> {
        match expr {
            Expr::Argument(index) => {
                let pointer =
                    *arguments
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "argument index {} out of bounds",
                                index
                            )
                        })?;

                let type_info =
                    argument_types
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "argument type index {} out of bounds",
                                index
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

            Expr::Constant(value) =>
                self.lower_constant(value),

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
                        } => (
                            pointer,
                            type_info,
                        ),

                        LoweredValue::Value(_) => {
                            return Err(
                                format!(
                                    "cannot access field `{}` on a value",
                                    name
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
                                    name
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
                                        field.type_info.clone(),
                                    ))
                                } else {
                                    None
                                }
                            },
                        )
                        .ok_or_else(|| {
                            format!(
                                "field `{}` not found",
                                name
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
                                name
                            ),
                        )
                        .map_err(|error| {
                            format!(
                                "failed to build GEP for field `{}`: {:?}",
                                name,
                                error
                            )
                        })?;

                Ok(
                    LoweredValue::Pointer {
                        pointer: field_pointer,
                        type_info: field_type,
                    }
                )
            }

            Expr::Add { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Add,
                ),

            Expr::Sub { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Sub,
                ),

            Expr::Mul { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Mul,
                ),

            Expr::Div { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Div,
                ),

            Expr::Rem { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Rem,
                ),

            Expr::Eq { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Eq,
                ),

            Expr::Ne { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Ne,
                ),

            Expr::Lt { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Lt,
                ),

            Expr::Le { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Le,
                ),

            Expr::Gt { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Gt,
                ),

            Expr::Ge { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Ge,
                ),

            Expr::And { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::And,
                ),

            Expr::Or { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Or,
                ),

            Expr::BitAnd { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::BitAnd,
                ),

            Expr::BitOr { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::BitOr,
                ),

            Expr::BitXor { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::BitXor,
                ),

            Expr::Shl { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Shl,
                ),

            Expr::Shr { lhs, rhs } =>
                self.lower_binary(
                    builder,
                    arguments,
                    argument_types,
                    lhs,
                    rhs,
                    BinaryOp::Shr,
                ),

            Expr::Not { operand } =>
                self.lower_unary(
                    builder,
                    arguments,
                    argument_types,
                    operand,
                    UnaryOp::Not,
                ),

            Expr::Neg { operand } =>
                self.lower_unary(
                    builder,
                    arguments,
                    argument_types,
                    operand,
                    UnaryOp::Neg,
                ),
        }
    }


    // --------------------------------------------------------
    // Constants
    // --------------------------------------------------------

    fn lower_constant(
        &self,
        value: &Value,
    ) -> Result<LoweredValue<'ctx>, String> {
        let value =
            match value {
                Value::F32(value) =>
                    self.context
                        .f32_type()
                        .const_float(*value as f64)
                        .into(),

                Value::F64(value) =>
                    self.context
                        .f64_type()
                        .const_float(*value)
                        .into(),

                Value::I8(value) =>
                    self.context
                        .i8_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I16(value) =>
                    self.context
                        .i16_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I32(value) =>
                    self.context
                        .i32_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I64(value) =>
                    self.context
                        .i64_type()
                        .const_int(
                            *value as u64,
                            true,
                        )
                        .into(),

                Value::I128(value) =>
                    self.context
                        .i128_type()
                        .const_int_arbitrary_precision(
                            &[
                                *value as u128 as u64,
                                ((*value as u128) >> 64) as u64,
                            ],
                        )
                        .into(),

                Value::U8(value) =>
                    self.context
                        .i8_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U16(value) =>
                    self.context
                        .i16_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U32(value) =>
                    self.context
                        .i32_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U64(value) =>
                    self.context
                        .i64_type()
                        .const_int(
                            *value,
                            false,
                        )
                        .into(),

                Value::U128(value) =>
                    self.context
                        .i128_type()
                        .const_int_arbitrary_precision(
                            &[
                                *value as u64,
                                (*value >> 64) as u64,
                            ],
                        )
                        .into(),

                Value::Bool(value) =>
                    self.context
                        .bool_type()
                        .const_int(
                            if *value { 1 } else { 0 },
                            false,
                        )
                        .into(),
            };

        Ok(
            LoweredValue::Value(value)
        )
    }


    // --------------------------------------------------------
    // Unary operations
    // --------------------------------------------------------

    fn lower_unary(
        &self,
        builder: &Builder<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        operand: &Expr,
        operation: UnaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
        let operand =
            self.lower_expr(
                builder,
                arguments,
                argument_types,
                operand,
            )?;

        let operand =
            self.materialize_value(
                builder,
                operand,
            )?;

        match operation {
            UnaryOp::Not => {
                let value =
                    operand.into_int_value();

                let result =
                    builder
                        .build_not(
                            value,
                            "not",
                        )
                        .map_err(|e| {
                            format!(
                                "failed to build not: {:?}",
                                e
                            )
                        })?;

                Ok(
                    LoweredValue::Value(
                        result.into()
                    )
                )
            }

            UnaryOp::Neg => {
                match operand {
                    BasicValueEnum::IntValue(value) => {
                        let result =
                            builder
                                .build_int_neg(
                                    value,
                                    "neg",
                                )
                                .map_err(|e| {
                                    format!(
                                        "failed to build integer negation: {:?}",
                                        e
                                    )
                                })?;

                        Ok(
                            LoweredValue::Value(
                                result.into()
                            )
                        )
                    }

                    BasicValueEnum::FloatValue(value) => {
                        let result =
                            builder
                                .build_float_neg(
                                    value,
                                    "neg",
                                )
                                .map_err(|e| {
                                    format!(
                                        "failed to build float negation: {:?}",
                                        e
                                    )
                                })?;

                        Ok(
                            LoweredValue::Value(
                                result.into()
                            )
                        )
                    }

                    _ =>
                        Err(
                            "unary - requires numeric operand"
                                .to_string()
                        ),
                }
            }
        }
    }


    // --------------------------------------------------------
    // Binary operations
    // --------------------------------------------------------

    fn lower_binary(
        &self,
        builder: &Builder<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        lhs: &Expr,
        rhs: &Expr,
        operation: BinaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
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

        match lhs {
            BasicValueEnum::FloatValue(lhs) => {
                let rhs =
                    rhs.into_float_value();

                self.lower_float_binary(
                    builder,
                    lhs,
                    rhs,
                    operation,
                )
            }

            BasicValueEnum::IntValue(lhs) => {
                let rhs =
                    rhs.into_int_value();

                self.lower_int_binary(
                    builder,
                    lhs,
                    rhs,
                    operation,
                    argument_types,
                    expr_type_hint(lhs, argument_types),
                )
            }

            _ =>
                Err(
                    "unsupported binary operand type"
                        .to_string()
                ),
        }
    }


    fn lower_float_binary(
        &self,
        builder: &Builder<'ctx>,
        lhs: FloatValue<'ctx>,
        rhs: FloatValue<'ctx>,
        operation: BinaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
        let result =
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

                BinaryOp::Rem =>
                    builder.build_float_rem(
                        lhs,
                        rhs,
                        "rem",
                    ),

                BinaryOp::Eq =>
                    return self.float_compare(
                        builder,
                        lhs,
                        rhs,
                        FloatPredicate::OEQ,
                    ),

                BinaryOp::Ne =>
                    return self.float_compare(
                        builder,
                        lhs,
                        rhs,
                        FloatPredicate::ONE,
                    ),

                BinaryOp::Lt =>
                    return self.float_compare(
                        builder,
                        lhs,
                        rhs,
                        FloatPredicate::OLT,
                    ),

                BinaryOp::Le =>
                    return self.float_compare(
                        builder,
                        lhs,
                        rhs,
                        FloatPredicate::OLE,
                    ),

                BinaryOp::Gt =>
                    return self.float_compare(
                        builder,
                        lhs,
                        rhs,
                        FloatPredicate::OGT,
                    ),

                BinaryOp::Ge =>
                    return self.float_compare(
                        builder,
                        lhs,
                        rhs,
                        FloatPredicate::OGE,
                    ),

                _ =>
                    return Err(
                        "unsupported floating-point operator"
                            .to_string()
                    ),
            }
            .map_err(|e| {
                format!(
                    "failed to build floating-point operation: {:?}",
                    e
                )
            })?;

        Ok(
            LoweredValue::Value(
                result.into()
            )
        )
    }


    fn float_compare(
        &self,
        builder: &Builder<'ctx>,
        lhs: FloatValue<'ctx>,
        rhs: FloatValue<'ctx>,
        predicate: FloatPredicate,
    ) -> Result<LoweredValue<'ctx>, String> {
        let result =
            builder
                .build_float_compare(
                    predicate,
                    lhs,
                    rhs,
                    "cmp",
                )
                .map_err(|e| {
                    format!(
                        "failed to build float comparison: {:?}",
                        e
                    )
                })?;

        Ok(
            LoweredValue::Value(
                result.into()
            )
        )
    }


    fn lower_int_binary(
        &self,
        builder: &Builder<'ctx>,
        lhs: IntValue<'ctx>,
        rhs: IntValue<'ctx>,
        operation: BinaryOp,
        _argument_types: &[TypeInfo],
        type_info: TypeInfo,
    ) -> Result<LoweredValue<'ctx>, String> {
        let unsigned =
            type_info.is_unsigned_integer();

        let result =
            match operation {
                BinaryOp::Add =>
                    builder
                        .build_int_add(
                            lhs,
                            rhs,
                            "add",
                        )
                        .map(|v| v.into()),

                BinaryOp::Sub =>
                    builder
                        .build_int_sub(
                            lhs,
                            rhs,
                            "sub",
                        )
                        .map(|v| v.into()),

                BinaryOp::Mul =>
                    builder
                        .build_int_mul(
                            lhs,
                            rhs,
                            "mul",
                        )
                        .map(|v| v.into()),

                BinaryOp::Div => {
                    if unsigned {
                        builder
                            .build_int_unsigned_div(
                                lhs,
                                rhs,
                                "div",
                            )
                            .map(|v| v.into())
                    } else {
                        builder
                            .build_int_signed_div(
                                lhs,
                                rhs,
                                "div",
                            )
                            .map(|v| v.into())
                    }
                }

                BinaryOp::Rem => {
                    if unsigned {
                        builder
                            .build_int_unsigned_rem(
                                lhs,
                                rhs,
                                "rem",
                            )
                            .map(|v| v.into())
                    } else {
                        builder
                            .build_int_signed_rem(
                                lhs,
                                rhs,
                                "rem",
                            )
                            .map(|v| v.into())
                    }
                }

                BinaryOp::Eq =>
                    builder
                        .build_int_compare(
                            IntPredicate::EQ,
                            lhs,
                            rhs,
                            "eq",
                        )
                        .map(|v| v.into()),

                BinaryOp::Ne =>
                    builder
                        .build_int_compare(
                            IntPredicate::NE,
                            lhs,
                            rhs,
                            "ne",
                        )
                        .map(|v| v.into()),

                BinaryOp::Lt => {
                    let predicate =
                        if unsigned {
                            IntPredicate::ULT
                        } else {
                            IntPredicate::SLT
                        };

                    builder
                        .build_int_compare(
                            predicate,
                            lhs,
                            rhs,
                            "lt",
                        )
                        .map(|v| v.into())
                }

                BinaryOp::Le => {
                    let predicate =
                        if unsigned {
                            IntPredicate::ULE
                        } else {
                            IntPredicate::SLE
                        };

                    builder
                        .build_int_compare(
                            predicate,
                            lhs,
                            rhs,
                            "le",
                        )
                        .map(|v| v.into())
                }

                BinaryOp::Gt => {
                    let predicate =
                        if unsigned {
                            IntPredicate::UGT
                        } else {
                            IntPredicate::SGT
                        };

                    builder
                        .build_int_compare(
                            predicate,
                            lhs,
                            rhs,
                            "gt",
                        )
                        .map(|v| v.into())
                }

                BinaryOp::Ge => {
                    let predicate =
                        if unsigned {
                            IntPredicate::UGE
                        } else {
                            IntPredicate::SGE
                        };

                    builder
                        .build_int_compare(
                            predicate,
                            lhs,
                            rhs,
                            "ge",
                        )
                        .map(|v| v.into())
                }

                BinaryOp::And =>
                    builder
                        .build_and(
                            lhs,
                            rhs,
                            "and",
                        )
                        .map(|v| v.into()),

                BinaryOp::Or =>
                    builder
                        .build_or(
                            lhs,
                            rhs,
                            "or",
                        )
                        .map(|v| v.into()),

                BinaryOp::BitAnd =>
                    builder
                        .build_and(
                            lhs,
                            rhs,
                            "bitand",
                        )
                        .map(|v| v.into()),

                BinaryOp::BitOr =>
                    builder
                        .build_or(
                            lhs,
                            rhs,
                            "bitor",
                        )
                        .map(|v| v.into()),

                BinaryOp::BitXor =>
                    builder
                        .build_xor(
                            lhs,
                            rhs,
                            "bitxor",
                        )
                        .map(|v| v.into()),

                BinaryOp::Shl =>
                    builder
                        .build_left_shift(
                            lhs,
                            rhs,
                            "shl",
                        )
                        .map(|v| v.into()),

                BinaryOp::Shr => {
                    builder
                        .build_right_shift(
                            lhs,
                            rhs,
                            !unsigned,
                            "shr",
                        )
                        .map(|v| v.into())
                }
            }
            .map_err(|e| {
                format!(
                    "failed to build integer operation: {:?}",
                    e
                )
            })?;

        Ok(
            LoweredValue::Value(result)
        )
    }


    // --------------------------------------------------------
    // Materialize pointer -> value
    // --------------------------------------------------------

    fn materialize_value(
        &self,
        builder: &Builder<'ctx>,
        value: LoweredValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            LoweredValue::Value(value) =>
                Ok(value),

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
                            error
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
    Value(BasicValueEnum<'ctx>),

    Pointer {
        pointer: PointerValue<'ctx>,
        type_info: TypeInfo,
    },
}


// ============================================================
// Operators
// ============================================================

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,

    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,

    And,
    Or,

    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}


enum UnaryOp {
    Not,
    Neg,
}


// ============================================================
// TypeInfo -> LLVM type
// ============================================================

fn llvm_type<'ctx>(
    context: &'ctx Context,
    type_info: &TypeInfo,
) -> Result<BasicTypeEnum<'ctx>, String> {
    match type_info {
        TypeInfo::F32 =>
            Ok(context.f32_type().into()),

        TypeInfo::F64 =>
            Ok(context.f64_type().into()),

        TypeInfo::I8 |
        TypeInfo::U8 =>
            Ok(context.i8_type().into()),

        TypeInfo::I16 |
        TypeInfo::U16 =>
            Ok(context.i16_type().into()),

        TypeInfo::I32 |
        TypeInfo::U32 =>
            Ok(context.i32_type().into()),

        TypeInfo::I64 |
        TypeInfo::U64 =>
            Ok(context.i64_type().into()),

        TypeInfo::I128 |
        TypeInfo::U128 =>
            Ok(context.i128_type().into()),

        TypeInfo::Bool =>
            Ok(context.bool_type().into()),

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
                    .collect::<Result<Vec<_>, _>>()?;

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
) -> Result<StructType<'ctx>, String> {
    match llvm_type(
        context,
        type_info,
    )? {
        BasicTypeEnum::StructType(
            struct_type,
        ) =>
            Ok(struct_type),

        _ =>
            Err(
                "expected struct LLVM type"
                    .to_string()
            ),
    }
}


// ============================================================
// Temporary type helper
// ============================================================
//
// NOTE:
// This is only a compatibility helper for the current one-argument
// compiler design. The important part of the integer fix is that
// signedness is determined by TypeInfo before emitting LLVM integer
// operations.
//
// We will want to replace this with proper expression type propagation
// once literals and nested expressions are fully typed.
//

fn expr_type_hint(
    _value: IntValue<'_>,
    argument_types: &[TypeInfo],
) -> TypeInfo {
    argument_types
        .first()
        .cloned()
        .unwrap_or(TypeInfo::I32)
}