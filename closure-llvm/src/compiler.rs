use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    values::{
        PointerValue,
    },
    AddressSpace,
    OptimizationLevel,
};

use crate::{
    expr::Closure,
    jit::CompiledClosure,
    types::{CompileType, TypeInfo},
    lowering::Lowering,
};


// ============================================================
// Compiler
// ============================================================

pub struct Compiler<'ctx> {
    pub(crate) context: &'ctx Context,
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

        let lowering =
            Lowering;

        let value =
            lowering.lower_expr(
                context,
                &builder,
                function,
                &arguments,
                &closure.arguments,
                &closure.return_type,
                &closure.body,
            )?;

        let value =
            lowering.materialize_value(
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
// TypeInfo -> LLVM type
// ============================================================

pub(crate) fn llvm_type<'ctx>(
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