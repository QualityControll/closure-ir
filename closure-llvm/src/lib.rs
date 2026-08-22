use std::marker::PhantomData;

use inkwell::{
    builder::Builder,
    context::Context,
    execution_engine::ExecutionEngine,
    module::Module,
    types::{BasicTypeEnum, StructType},
    values::{
        BasicValueEnum,
        FloatValue,
        FunctionValue,
        IntValue,
        PointerValue,
    },
    AddressSpace,
    FloatPredicate,
    IntPredicate,
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        matches!(self, Self::F32 | Self::F64)
    }

    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::I128
        )
    }

    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
                | Self::U128
        )
    }

    pub fn is_integer(&self) -> bool {
        self.is_signed_integer()
            || self.is_unsigned_integer()
            || self.is_bool()
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool)
    }

    pub fn is_numeric(&self) -> bool {
        self.is_float()
            || self.is_signed_integer()
            || self.is_unsigned_integer()
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
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
// Tuple CompileType implementations
// ============================================================

impl CompileType for () {
    fn type_info() -> TypeInfo {
        TypeInfo::Struct {
            name: "()".to_string(),
            fields: Vec::new(),
        }
    }

    fn llvm_type<'ctx>(
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {
        context.struct_type(&[], false).into()
    }
}


macro_rules! impl_tuple_compile_type {
    ($($T:ident : $index:tt),+) => {
        impl<$($T: CompileType),+> CompileType
            for ($($T,)+)
        {
            fn type_info() -> TypeInfo {
                TypeInfo::Struct {
                    name: stringify!(($($T,)+)).to_string(),

                    fields: vec![
                        $(
                            FieldInfo {
                                name: stringify!($index).to_string(),
                                type_info: $T::type_info(),
                            }
                        ),+
                    ],
                }
            }

            fn llvm_type<'ctx>(
                context: &'ctx Context,
            ) -> BasicTypeEnum<'ctx> {
                context
                    .struct_type(
                        &[
                            $(
                                $T::llvm_type(context)
                            ),+
                        ],
                        false,
                    )
                    .into()
            }
        }
    };
}

impl_tuple_compile_type!(A: 0);
impl_tuple_compile_type!(A: 0, B: 1);
impl_tuple_compile_type!(A: 0, B: 1, C: 2);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14);
impl_tuple_compile_type!(A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15);


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

    IfElse {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
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

impl<'ctx, Args, Ret> CompiledClosure<'ctx, Args, Ret>
where
    Args: CompileType,
    Ret: CompileType + 'static,
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
    ) -> Ret {
        jit_call::<Args, Ret>(
            &self.engine,
            &self.function_name,
            value,
        )
    }
}


// ============================================================
// JIT invocation
// ============================================================

unsafe fn jit_call<Args, Ret>(
    engine: &ExecutionEngine<'_>,
    function_name: &str,
    value: &Args,
) -> Ret
where
    Args: CompileType,
    Ret: CompileType + 'static,
{
    type JitFn<Ret> =
        unsafe extern "C" fn(*const u8) -> Ret;

    let function =
        engine
            .get_function::<JitFn<Ret>>(function_name)
            .expect("failed to get JIT function");

    function.call(
        value as *const Args as *const u8
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

    pub fn compile<Args, Ret>(
        &self,
        closure: &Closure,
    ) -> Result<
        CompiledClosure<'ctx, Args, Ret>,
        String,
    >
    where
        Args: CompileType,
        Ret: CompileType + 'static,
    {
        let context = self.context;

        let module =
            context.create_module("closure_module");

        let function_name =
            "compiled_closure";

        self.generate_function(
            context,
            &module,
            function_name,
            closure,
        )?;

        println!(
            "Generated LLVM IR:\n{}",
            module.print_to_string().to_string()
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


    // ========================================================
    // Function generation
    // ========================================================

    fn generate_function(
        &self,
        context: &'ctx Context,
        module: &Module<'ctx>,
        function_name: &str,
        closure: &Closure,
    ) -> Result<(), String> {
        let return_type =
            llvm_type(
                context,
                &closure.return_type,
            )?;

        let argument_pointer_type =
            context.ptr_type(
                AddressSpace::default(),
            );

        let function_type =
            basic_type_fn_type(
                return_type,
                argument_pointer_type,
            );

        let function =
            module.add_function(
                function_name,
                function_type,
                None,
            );

        let entry =
            context.append_basic_block(
                function,
                "entry",
            );

        let builder =
            context.create_builder();

        builder.position_at_end(entry);

        let argument_pointer =
            function
                .get_nth_param(0)
                .ok_or_else(|| {
                    "missing function argument"
                        .to_string()
                })?
                .into_pointer_value();

        let arguments =
            self.build_argument_pointers(
                context,
                &builder,
                argument_pointer,
                &closure.arguments,
            )?;

        let value =
            self.lower_expr(
                context,
                &builder,
                function,
                &arguments,
                &closure.arguments,
                &closure.return_type,
                &closure.body,
            )?;

        let value =
            self.materialize_value(
                context,
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


    // ========================================================
    // Argument pointers
    // ========================================================

    fn build_argument_pointers(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        argument_pointer: PointerValue<'ctx>,
        argument_types: &[TypeInfo],
    ) -> Result<
        Vec<PointerValue<'ctx>>,
        String,
    > {
        if argument_types.is_empty() {
            return Ok(Vec::new());
        }

        let field_types =
            argument_types
                .iter()
                .map(|type_info| {
                    llvm_type(
                        context,
                        type_info,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

        let tuple_type =
            context.struct_type(
                &field_types,
                false,
            );

        let mut result =
            Vec::with_capacity(
                argument_types.len()
            );

        for index in 0..argument_types.len() {
            let pointer =
                builder
                    .build_struct_gep(
                        tuple_type,
                        argument_pointer,
                        index as u32,
                        &format!("arg{}_ptr", index),
                    )
                    .map_err(|error| {
                        format!(
                            "failed to build argument {} GEP: {:?}",
                            index,
                            error
                        )
                    })?;

            result.push(pointer);
        }

        Ok(result)
    }


    // ========================================================
    // Expression lowering
    // ========================================================

    fn lower_expr(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
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
                        .cloned()
                        .ok_or_else(|| {
                            format!(
                                "argument type index {} out of bounds",
                                index
                            )
                        })?;

                Ok(
                    LoweredValue::Pointer {
                        pointer,
                        type_info,
                    }
                )
            }

            Expr::Constant(value) =>
                self.lower_constant(
                    context,
                    value,
                ),

            Expr::Field {
                object,
                name,
            } =>
                self.lower_field(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    object,
                    name,
                ),

            Expr::IfElse {
                condition,
                then_branch,
                else_branch,
            } =>
                self.lower_if_else(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    condition,
                    then_branch,
                    else_branch,
                ),

            Expr::Add { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Add,
                ),

            Expr::Sub { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Sub,
                ),

            Expr::Mul { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Mul,
                ),

            Expr::Div { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Div,
                ),

            Expr::Rem { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Rem,
                ),

            Expr::Eq { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Eq,
                ),

            Expr::Ne { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Ne,
                ),

            Expr::Lt { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Lt,
                ),

            Expr::Le { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Le,
                ),

            Expr::Gt { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Gt,
                ),

            Expr::Ge { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Ge,
                ),

            Expr::And { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::And,
                ),

            Expr::Or { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Or,
                ),

            Expr::BitAnd { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::BitAnd,
                ),

            Expr::BitOr { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::BitOr,
                ),

            Expr::BitXor { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::BitXor,
                ),

            Expr::Shl { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Shl,
                ),

            Expr::Shr { lhs, rhs } =>
                self.lower_binary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    lhs,
                    rhs,
                    BinaryOp::Shr,
                ),

            Expr::Not { operand } =>
                self.lower_unary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    operand,
                    UnaryOp::Not,
                ),

            Expr::Neg { operand } =>
                self.lower_unary(
                    context,
                    builder,
                    function,
                    arguments,
                    argument_types,
                    expected_type,
                    operand,
                    UnaryOp::Neg,
                ),
        }
    }


    // ========================================================
    // Field
    // ========================================================

    fn lower_field(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        object: &Expr,
        name: &str,
    ) -> Result<LoweredValue<'ctx>, String> {
        let object =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                object,
            )?;

        let (object_pointer, object_type) =
            match object {
                LoweredValue::Pointer {
                    pointer,
                    type_info,
                } => (pointer, type_info),

                LoweredValue::Value(_) =>
                    return Err(format!(
                        "cannot access field `{}` on a value",
                        name
                    )),
            };

        let fields =
            match &object_type {
                TypeInfo::Struct {
                    fields,
                    ..
                } => fields,

                _ =>
                    return Err(format!(
                        "cannot access field `{}` on non-struct type",
                        name
                    )),
            };

        let (field_index, field_type) =
            fields
                .iter()
                .enumerate()
                .find_map(|(index, field)| {
                    if field.name == name {
                        Some((
                            index,
                            field.type_info.clone(),
                        ))
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
            llvm_struct_type(
                context,
                &object_type,
            )?;

        let field_pointer =
            builder
                .build_struct_gep(
                    struct_type,
                    object_pointer,
                    field_index as u32,
                    &format!("{}_ptr", name),
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


    // ========================================================
    // If / else
    // ========================================================

    fn lower_if_else(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: &Expr,
    ) -> Result<LoweredValue<'ctx>, String> {
        let condition =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                &TypeInfo::Bool,
                condition,
            )?;

        let condition =
            self.materialize_value(
                context,
                builder,
                condition,
            )?;

        let condition =
            match condition {
                BasicValueEnum::IntValue(value)
                    if value.get_type().get_bit_width() == 1 =>
                {
                    value
                }

                BasicValueEnum::IntValue(_) =>
                    return Err(
                        "if condition must be bool"
                            .to_string()
                    ),

                _ =>
                    return Err(
                        "if condition must be bool"
                            .to_string()
                    ),
            };

        let then_block =
            context.append_basic_block(
                function,
                "then",
            );

        let else_block =
            context.append_basic_block(
                function,
                "else",
            );

        let merge_block =
            context.append_basic_block(
                function,
                "if_merge",
            );

        builder
            .build_conditional_branch(
                condition,
                then_block,
                else_block,
            )
            .map_err(|error| {
                format!(
                    "failed to build conditional branch: {:?}",
                    error
                )
            })?;

        // ----------------------------------------------------
        // Then
        // ----------------------------------------------------

        builder.position_at_end(then_block);

        let then_value =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                then_branch,
            )?;

        let then_value =
            self.materialize_value(
                context,
                builder,
                then_value,
            )?;

        let then_end =
            builder
                .get_insert_block()
                .ok_or_else(|| {
                    "missing then block"
                        .to_string()
                })?;

        if then_end.get_terminator().is_none() {
            builder
                .build_unconditional_branch(
                    merge_block,
                )
                .map_err(|error| {
                    format!(
                        "failed to branch from then block: {:?}",
                        error
                    )
                })?;
        }

        // ----------------------------------------------------
        // Else
        // ----------------------------------------------------

        builder.position_at_end(else_block);

        let else_value =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                else_branch,
            )?;

        let else_value =
            self.materialize_value(
                context,
                builder,
                else_value,
            )?;

        let else_end =
            builder
                .get_insert_block()
                .ok_or_else(|| {
                    "missing else block"
                        .to_string()
                })?;

        if else_end.get_terminator().is_none() {
            builder
                .build_unconditional_branch(
                    merge_block,
                )
                .map_err(|error| {
                    format!(
                        "failed to branch from else block: {:?}",
                        error
                    )
                })?;
        }

        // ----------------------------------------------------
        // Merge
        // ----------------------------------------------------

        builder.position_at_end(merge_block);

        let phi =
            builder
                .build_phi(
                    llvm_type(
                        context,
                        expected_type,
                    )?,
                    "if_result",
                )
                .map_err(|error| {
                    format!(
                        "failed to build if/else PHI: {:?}",
                        error
                    )
                })?;

        phi.add_incoming(&[
            (&then_value, then_end),
            (&else_value, else_end),
        ]);

        Ok(
            LoweredValue::Value(
                phi.as_basic_value()
            )
        )
    }


    // ========================================================
    // Constants
    // ========================================================

    fn lower_constant(
        &self,
        context: &'ctx Context,
        value: &Value,
    ) -> Result<LoweredValue<'ctx>, String> {
        let value =
            match value {
                Value::F32(value) =>
                    context
                        .f32_type()
                        .const_float(*value as f64)
                        .into(),

                Value::F64(value) =>
                    context
                        .f64_type()
                        .const_float(*value)
                        .into(),

                Value::I8(value) =>
                    context
                        .i8_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I16(value) =>
                    context
                        .i16_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I32(value) =>
                    context
                        .i32_type()
                        .const_int(
                            *value as i64 as u64,
                            true,
                        )
                        .into(),

                Value::I64(value) =>
                    context
                        .i64_type()
                        .const_int(
                            *value as u64,
                            true,
                        )
                        .into(),

                Value::I128(value) =>
                    context
                        .i128_type()
                        .const_int_arbitrary_precision(
                            &[
                                *value as u128 as u64,
                                ((*value as u128) >> 64) as u64,
                            ],
                        )
                        .into(),

                Value::U8(value) =>
                    context
                        .i8_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U16(value) =>
                    context
                        .i16_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U32(value) =>
                    context
                        .i32_type()
                        .const_int(
                            *value as u64,
                            false,
                        )
                        .into(),

                Value::U64(value) =>
                    context
                        .i64_type()
                        .const_int(
                            *value,
                            false,
                        )
                        .into(),

                Value::U128(value) =>
                    context
                        .i128_type()
                        .const_int_arbitrary_precision(
                            &[
                                *value as u64,
                                (*value >> 64) as u64,
                            ],
                        )
                        .into(),

                Value::Bool(value) =>
                    context
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


    // ========================================================
    // Unary
    // ========================================================

    fn lower_unary(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        operand: &Expr,
        operation: UnaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
        let operand =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                expected_type,
                operand,
            )?;

        let operand =
            self.materialize_value(
                context,
                builder,
                operand,
            )?;

        match operation {
            UnaryOp::Not => {
                let value =
                    match operand {
                        BasicValueEnum::IntValue(value) =>
                            value,

                        _ =>
                            return Err(
                                "unary ! requires an integer or bool operand"
                                    .to_string()
                            ),
                    };

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


    // ========================================================
    // Binary
    // ========================================================

    fn lower_binary(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        lhs: &Expr,
        rhs: &Expr,
        operation: BinaryOp,
    ) -> Result<LoweredValue<'ctx>, String> {
        let operand_type =
            binary_operand_type(
                argument_types,
                lhs,
                rhs,
                expected_type,
                &operation,
            )?;

        let lhs =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                &operand_type,
                lhs,
            )?;

        let lhs =
            self.materialize_value(
                context,
                builder,
                lhs,
            )?;

        let rhs =
            self.lower_expr(
                context,
                builder,
                function,
                arguments,
                argument_types,
                &operand_type,
                rhs,
            )?;

        let rhs =
            self.materialize_value(
                context,
                builder,
                rhs,
            )?;

        match (lhs, rhs) {
            (
                BasicValueEnum::FloatValue(lhs),
                BasicValueEnum::FloatValue(rhs),
            ) =>
                self.lower_float_binary(
                    builder,
                    lhs,
                    rhs,
                    operation,
                ),

            (
                BasicValueEnum::IntValue(lhs),
                BasicValueEnum::IntValue(rhs),
            ) =>
                self.lower_int_binary(
                    builder,
                    lhs,
                    rhs,
                    operation,
                    &operand_type,
                ),

            _ =>
                Err(
                    "binary operands must have matching numeric or integer types"
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
        match operation {
            BinaryOp::Add =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_add(
                            lhs,
                            rhs,
                            "add",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Sub =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_sub(
                            lhs,
                            rhs,
                            "sub",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Mul =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_mul(
                            lhs,
                            rhs,
                            "mul",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Div =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_div(
                            lhs,
                            rhs,
                            "div",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Rem =>
                Ok(LoweredValue::Value(
                    builder
                        .build_float_rem(
                            lhs,
                            rhs,
                            "rem",
                        )
                        .map_err(|e| format!("{:?}", e))?
                        .into()
                )),

            BinaryOp::Eq =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OEQ,
                ),

            BinaryOp::Ne =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::ONE,
                ),

            BinaryOp::Lt =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OLT,
                ),

            BinaryOp::Le =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OLE,
                ),

            BinaryOp::Gt =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OGT,
                ),

            BinaryOp::Ge =>
                self.float_compare(
                    builder,
                    lhs,
                    rhs,
                    FloatPredicate::OGE,
                ),

            _ =>
                Err(
                    "unsupported floating-point operator"
                        .to_string()
                ),
        }
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
        type_info: &TypeInfo,
    ) -> Result<LoweredValue<'ctx>, String> {
        let unsigned =
            type_info.is_unsigned_integer();

        let result =
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

                BinaryOp::Div => {
                    if unsigned {
                        builder.build_int_unsigned_div(
                            lhs,
                            rhs,
                            "div",
                        )
                    } else {
                        builder.build_int_signed_div(
                            lhs,
                            rhs,
                            "div",
                        )
                    }
                }

                BinaryOp::Rem => {
                    if unsigned {
                        builder.build_int_unsigned_rem(
                            lhs,
                            rhs,
                            "rem",
                        )
                    } else {
                        builder.build_int_signed_rem(
                            lhs,
                            rhs,
                            "rem",
                        )
                    }
                }

                BinaryOp::Eq =>
                    builder.build_int_compare(
                        IntPredicate::EQ,
                        lhs,
                        rhs,
                        "eq",
                    ),

                BinaryOp::Ne =>
                    builder.build_int_compare(
                        IntPredicate::NE,
                        lhs,
                        rhs,
                        "ne",
                    ),

                BinaryOp::Lt =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::ULT
                        } else {
                            IntPredicate::SLT
                        },
                        lhs,
                        rhs,
                        "lt",
                    ),

                BinaryOp::Le =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::ULE
                        } else {
                            IntPredicate::SLE
                        },
                        lhs,
                        rhs,
                        "le",
                    ),

                BinaryOp::Gt =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::UGT
                        } else {
                            IntPredicate::SGT
                        },
                        lhs,
                        rhs,
                        "gt",
                    ),

                BinaryOp::Ge =>
                    builder.build_int_compare(
                        if unsigned {
                            IntPredicate::UGE
                        } else {
                            IntPredicate::SGE
                        },
                        lhs,
                        rhs,
                        "ge",
                    ),

                BinaryOp::And =>
                    builder.build_and(
                        lhs,
                        rhs,
                        "and",
                    ),

                BinaryOp::Or =>
                    builder.build_or(
                        lhs,
                        rhs,
                        "or",
                    ),

                BinaryOp::BitAnd =>
                    builder.build_and(
                        lhs,
                        rhs,
                        "bitand",
                    ),

                BinaryOp::BitOr =>
                    builder.build_or(
                        lhs,
                        rhs,
                        "bitor",
                    ),

                BinaryOp::BitXor =>
                    builder.build_xor(
                        lhs,
                        rhs,
                        "bitxor",
                    ),

                BinaryOp::Shl =>
                    builder.build_left_shift(
                        lhs,
                        rhs,
                        "shl",
                    ),

                BinaryOp::Shr =>
                    builder.build_right_shift(
                        lhs,
                        rhs,
                        !unsigned,
                        "shr",
                    ),
            }
            .map_err(|e| {
                format!(
                    "failed to build integer operation: {:?}",
                    e
                )
            })?;

        Ok(
            LoweredValue::Value(
                result.into()
            )
        )
    }


    // ========================================================
    // Materialize
    // ========================================================

    fn materialize_value(
        &self,
        context: &'ctx Context,
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
                        context,
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
// Function type helper
// ============================================================

fn basic_type_fn_type<'ctx>(
    return_type: BasicTypeEnum<'ctx>,
    argument_pointer_type:
        inkwell::types::PointerType<'ctx>,
) -> inkwell::types::FunctionType<'ctx> {
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

#[derive(Debug, Clone, Copy)]
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


#[derive(Debug, Clone, Copy)]
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

        TypeInfo::I8 | TypeInfo::U8 =>
            Ok(context.i8_type().into()),

        TypeInfo::I16 | TypeInfo::U16 =>
            Ok(context.i16_type().into()),

        TypeInfo::I32 | TypeInfo::U32 =>
            Ok(context.i32_type().into()),

        TypeInfo::I64 | TypeInfo::U64 =>
            Ok(context.i64_type().into()),

        TypeInfo::I128 | TypeInfo::U128 =>
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
// Struct type
// ============================================================

fn llvm_struct_type<'ctx>(
    context: &'ctx Context,
    type_info: &TypeInfo,
) -> Result<StructType<'ctx>, String> {
    match llvm_type(
        context,
        type_info,
    )? {
        BasicTypeEnum::StructType(struct_type) =>
            Ok(struct_type),

        _ =>
            Err(
                "expected struct LLVM type"
                    .to_string()
            ),
    }
}


// ============================================================
// Expression type inference
// ============================================================

fn expression_type(
    argument_types: &[TypeInfo],
    expr: &Expr,
) -> Result<TypeInfo, String> {
    match expr {
        Expr::Argument(index) =>
            argument_types
                .get(*index)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "argument index {} out of bounds",
                        index
                    )
                }),

        Expr::Constant(value) =>
            Ok(match value {
                Value::F32(_) => TypeInfo::F32,
                Value::F64(_) => TypeInfo::F64,

                Value::I8(_) => TypeInfo::I8,
                Value::I16(_) => TypeInfo::I16,
                Value::I32(_) => TypeInfo::I32,
                Value::I64(_) => TypeInfo::I64,
                Value::I128(_) => TypeInfo::I128,

                Value::U8(_) => TypeInfo::U8,
                Value::U16(_) => TypeInfo::U16,
                Value::U32(_) => TypeInfo::U32,
                Value::U64(_) => TypeInfo::U64,
                Value::U128(_) => TypeInfo::U128,

                Value::Bool(_) => TypeInfo::Bool,
            }),

        Expr::Field {
            object,
            name,
        } => {
            let object_type =
                expression_type(
                    argument_types,
                    object,
                )?;

            let fields =
                match object_type {
                    TypeInfo::Struct {
                        fields,
                        ..
                    } => fields,

                    _ =>
                        return Err(format!(
                            "cannot access field `{}` on non-struct type",
                            name
                        )),
                };

            fields
                .into_iter()
                .find(|field| field.name == *name)
                .map(|field| field.type_info)
                .ok_or_else(|| {
                    format!(
                        "field `{}` not found",
                        name
                    )
                })
        }

        Expr::Add { lhs, .. }
        | Expr::Sub { lhs, .. }
        | Expr::Mul { lhs, .. }
        | Expr::Div { lhs, .. }
        | Expr::Rem { lhs, .. }
        | Expr::BitAnd { lhs, .. }
        | Expr::BitOr { lhs, .. }
        | Expr::BitXor { lhs, .. }
        | Expr::Shl { lhs, .. }
        | Expr::Shr { lhs, .. }
        | Expr::Neg { operand: lhs } =>
            expression_type(
                argument_types,
                lhs,
            ),

        Expr::Eq { .. }
        | Expr::Ne { .. }
        | Expr::Lt { .. }
        | Expr::Le { .. }
        | Expr::Gt { .. }
        | Expr::Ge { .. }
        | Expr::And { .. }
        | Expr::Or { .. }
        | Expr::Not { .. } =>
            Ok(TypeInfo::Bool),

        Expr::IfElse {
            then_branch,
            else_branch,
            ..
        } => {
            let then_type =
                expression_type(
                    argument_types,
                    then_branch,
                )?;

            let else_type =
                expression_type(
                    argument_types,
                    else_branch,
                )?;

            if then_type != else_type {
                return Err(
                    "if/else branches must have the same type"
                        .to_string()
                );
            }

            Ok(then_type)
        }
    }
}


// ============================================================
// Binary operand type
// ============================================================

fn binary_operand_type(
    argument_types: &[TypeInfo],
    lhs: &Expr,
    rhs: &Expr,
    expected_type: &TypeInfo,
    operation: &BinaryOp,
) -> Result<TypeInfo, String> {
    let lhs_type =
        expression_type(
            argument_types,
            lhs,
        )?;

    let rhs_type =
        expression_type(
            argument_types,
            rhs,
        )?;

    match operation {
        BinaryOp::Eq
        | BinaryOp::Ne
        | BinaryOp::Lt
        | BinaryOp::Le
        | BinaryOp::Gt
        | BinaryOp::Ge => {
            if lhs_type != rhs_type {
                return Err(
                    "comparison operands must have the same type"
                        .to_string()
                );
            }

            Ok(lhs_type)
        }

        BinaryOp::And | BinaryOp::Or => {
            if !lhs_type.is_bool()
                || !rhs_type.is_bool()
            {
                return Err(
                    "logical &&/|| operands must be bool"
                        .to_string()
                );
            }

            Ok(TypeInfo::Bool)
        }

        _ => {
            if lhs_type != rhs_type {
                return Err(
                    "binary operands must have the same type"
                        .to_string()
                );
            }

            if !lhs_type.is_numeric()
                && !lhs_type.is_bool()
            {
                return Err(
                    "unsupported binary operand type"
                        .to_string()
                );
            }

            let _ = expected_type;

            Ok(lhs_type)
        }
    }
}