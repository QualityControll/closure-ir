use inkwell::{
    context::Context,
    execution_engine::JitFunction,
    module::Module,
    types::BasicTypeEnum,
    values::{BasicValueEnum, IntValue},
    OptimizationLevel,
};

pub use closure_llvm_macro::{
    compile_closure,
    CompileType,
};


// ============================================================
// Type system
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

    // Store the function instead of calling it while
    // initializing a static.
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
// Primitive CompileType implementations
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
// TypeInfo helpers
// ============================================================

impl TypeInfo {

    pub fn field(
        &self,
        name: &str,
    ) -> Option<(usize, TypeInfo)> {

        match self {

            TypeInfo::Struct {
                fields,
                ..
            } => {

                fields
                    .iter()
                    .enumerate()
                    .find(|(_, field)| {
                        field.name == name
                    })
                    .map(|(index, field)| {
                        (
                            index,
                            field.ty(),
                        )
                    })
            }

            _ => None,
        }
    }


    pub fn llvm_type<'ctx>(
        &self,
        context: &'ctx Context,
    ) -> BasicTypeEnum<'ctx> {

        match self {

            TypeInfo::I32 => {
                context.i32_type().into()
            }

            TypeInfo::I64 => {
                context.i64_type().into()
            }

            TypeInfo::F32 => {
                context.f32_type().into()
            }

            TypeInfo::F64 => {
                context.f64_type().into()
            }

            TypeInfo::Bool => {
                context.bool_type().into()
            }

            TypeInfo::Struct {
                fields,
                ..
            } => {

                let llvm_fields: Vec<_> =
                    fields
                        .iter()
                        .map(|field| {
                            field
                                .ty()
                                .llvm_type(context)
                        })
                        .collect();

                context
                    .struct_type(
                        &llvm_fields,
                        false,
                    )
                    .into()
            }
        }
    }
}


// ============================================================
// Values
// ============================================================

#[derive(Debug, Clone)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Bool(bool),
}


// ============================================================
// Closure IR
// ============================================================

#[derive(Debug, Clone)]
pub enum Expr {
    Argument(usize),

    Constant(Value),

    Field {
        object: Box<Expr>,
        name: &'static str,
    },

    Add(
        Box<Expr>,
        Box<Expr>,
    ),

    Sub(
        Box<Expr>,
        Box<Expr>,
    ),

    Mul(
        Box<Expr>,
        Box<Expr>,
    ),

    Div(
        Box<Expr>,
        Box<Expr>,
    ),

    Neg(
        Box<Expr>,
    ),
}


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


impl<'ctx> Compiler<'ctx> {

    pub fn new(
        context: &'ctx Context,
    ) -> Self {
        Self {
            context,
        }
    }


    // --------------------------------------------------------
    // Current demonstration compiler
    //
    // Compiles:
    //
    // fn(i32, i32) -> i32
    // --------------------------------------------------------

    pub fn compile_i32_binary(
        &self,
        closure: &Closure,
    ) -> Result<
        JitFunction<
            'ctx,
            unsafe extern "C" fn(i32, i32) -> i32,
        >,
        String,
    > {

        let module =
            self.context
                .create_module("closure");

        let builder =
            self.context
                .create_builder();

        let i32_type =
            self.context.i32_type();

        let fn_type =
            i32_type.fn_type(
                &[
                    i32_type.into(),
                    i32_type.into(),
                ],
                false,
            );

        let function =
            module.add_function(
                "closure",
                fn_type,
                None,
            );

        let entry =
            self.context.append_basic_block(
                function,
                "entry",
            );

        builder.position_at_end(entry);

        let arguments = [
            function
                .get_nth_param(0)
                .unwrap(),

            function
                .get_nth_param(1)
                .unwrap(),
        ];

        let result =
            self.compile_i32_expr(
                &builder,
                &closure.body,
                &arguments,
            )?;

        builder
            .build_return(Some(&result))
            .map_err(|e| e.to_string())?;

        module
            .verify()
            .map_err(|e| e.to_string())?;

        println!("Generated LLVM:");

        module.print_to_stderr();

        let engine =
            module
                .create_jit_execution_engine(
                    OptimizationLevel::Default,
                )
                .map_err(|e| e.to_string())?;

        unsafe {
            engine
                .get_function("closure")
                .map_err(|e| e.to_string())
        }
    }


    // --------------------------------------------------------
    // Compile i32 expression
    // --------------------------------------------------------

    fn compile_i32_expr(
        &self,

        builder:
            &inkwell::builder::Builder<'ctx>,

        expr: &Expr,

        arguments:
            &[BasicValueEnum<'ctx>],
    ) -> Result<
        IntValue<'ctx>,
        String,
    > {

        match expr {

            // ------------------------------------------------
            // Argument
            // ------------------------------------------------

            Expr::Argument(index) => {

                let value =
                    arguments
                        .get(*index)
                        .ok_or_else(|| {
                            format!(
                                "invalid argument index {}",
                                index
                            )
                        })?;

                Ok(
                    value.into_int_value()
                )
            }


            // ------------------------------------------------
            // Constant
            // ------------------------------------------------

            Expr::Constant(
                Value::I32(value)
            ) => {

                Ok(
                    self.context
                        .i32_type()
                        .const_int(
                            *value as u64,
                            true,
                        )
                )
            }


            // ------------------------------------------------
            // Addition
            // ------------------------------------------------

            Expr::Add(
                lhs,
                rhs,
            ) => {

                let lhs =
                    self.compile_i32_expr(
                        builder,
                        lhs,
                        arguments,
                    )?;

                let rhs =
                    self.compile_i32_expr(
                        builder,
                        rhs,
                        arguments,
                    )?;

                builder
                    .build_int_add(
                        lhs,
                        rhs,
                        "add",
                    )
                    .map_err(|e| e.to_string())
            }


            // ------------------------------------------------
            // Subtraction
            // ------------------------------------------------

            Expr::Sub(
                lhs,
                rhs,
            ) => {

                let lhs =
                    self.compile_i32_expr(
                        builder,
                        lhs,
                        arguments,
                    )?;

                let rhs =
                    self.compile_i32_expr(
                        builder,
                        rhs,
                        arguments,
                    )?;

                builder
                    .build_int_sub(
                        lhs,
                        rhs,
                        "sub",
                    )
                    .map_err(|e| e.to_string())
            }


            // ------------------------------------------------
            // Multiplication
            // ------------------------------------------------

            Expr::Mul(
                lhs,
                rhs,
            ) => {

                let lhs =
                    self.compile_i32_expr(
                        builder,
                        lhs,
                        arguments,
                    )?;

                let rhs =
                    self.compile_i32_expr(
                        builder,
                        rhs,
                        arguments,
                    )?;

                builder
                    .build_int_mul(
                        lhs,
                        rhs,
                        "mul",
                    )
                    .map_err(|e| e.to_string())
            }


            // ------------------------------------------------
            // Division
            // ------------------------------------------------

            Expr::Div(
                lhs,
                rhs,
            ) => {

                let lhs =
                    self.compile_i32_expr(
                        builder,
                        lhs,
                        arguments,
                    )?;

                let rhs =
                    self.compile_i32_expr(
                        builder,
                        rhs,
                        arguments,
                    )?;

                builder
                    .build_int_signed_div(
                        lhs,
                        rhs,
                        "div",
                    )
                    .map_err(|e| e.to_string())
            }


            // ------------------------------------------------
            // Negation
            // ------------------------------------------------

            Expr::Neg(value) => {

                let value =
                    self.compile_i32_expr(
                        builder,
                        value,
                        arguments,
                    )?;

                builder
                    .build_int_neg(
                        value,
                        "neg",
                    )
                    .map_err(|e| e.to_string())
            }


            // ------------------------------------------------
            // Struct field
            //
            // Not implemented by the i32 demonstration
            // compiler yet.
            // ------------------------------------------------

            Expr::Field { .. } => {
                Err(
                    "struct field access requires \
                     struct argument lowering"
                        .into()
                )
            }


            // ------------------------------------------------
            // Wrong constant type
            // ------------------------------------------------

            Expr::Constant(_) => {
                Err(
                    "non-i32 constant in i32 expression"
                        .into()
                )
            }
        }
    }
}


// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {

    use super::*;


    #[derive(Debug, CompileType)]
    struct Point {
        x: f64,
        y: f64,
    }


    #[test]
    fn point_type_information() {

        let info =
            Point::type_info();

        match info {

            TypeInfo::Struct {
                name,
                fields,
            } => {

                assert_eq!(
                    name,
                    "Point"
                );

                assert_eq!(
                    fields.len(),
                    2
                );

                assert_eq!(
                    fields[0].name,
                    "x"
                );

                assert_eq!(
                    fields[1].name,
                    "y"
                );

                assert!(
                    matches!(
                        fields[0].ty(),
                        TypeInfo::F64
                    )
                );
            }

            _ => {
                panic!(
                    "expected struct"
                );
            }
        }
    }
}