use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::BasicTypeEnum,
    values::PointerValue,
    AddressSpace,
    OptimizationLevel,
};

use crate::{
    expr::Closure,
    jit::{
        CompiledClosure,
        DynamicCompiledClosure,
    },
    lowering::Lowering,
    types::{
        CompileType,
        TypeInfo,
    },
};


// ============================================================
// Compiler
// ============================================================

pub struct Compiler<'ctx> {
    pub(crate) context:
        &'ctx Context,
}


impl<'ctx> Compiler<'ctx> {
    pub fn new(
        context: &'ctx Context,
    ) -> Self {
        Self {
            context,
        }
    }


    // ========================================================
    // Typed compilation
    //
    // Used by compile_closure! and the statically typed API.
    // ========================================================

    pub fn compile_typed<Args, Ret>(
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
        let context =
            self.context;


        let module =
            context.create_module(
                "closure_module"
            );


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


    // ========================================================
    // Dynamic compilation
    //
    // Used when a Closure has been deserialized and therefore
    // the Rust generic argument/return types are not available.
    //
    // The Closure itself contains:
    //
    //     arguments
    //     return_type
    //     body
    //
    // which is sufficient to generate the LLVM function.
    // ========================================================

    pub fn compile(
        &self,
        closure: &Closure,
    ) -> Result<
        DynamicCompiledClosure<'ctx>,
        String,
    > {
        let context =
            self.context;


        let module =
            context.create_module(
                "closure_module"
            );


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
            DynamicCompiledClosure::new(
                engine,
                function_name.to_string(),
                closure.arguments.clone(),
                closure.return_type.clone(),
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
        let argument_pointer_type =
            context.ptr_type(
                AddressSpace::default(),
            );


        let result_pointer_type =
            context.ptr_type(
                AddressSpace::default(),
            );


        // ----------------------------------------------------
        // IMPORTANT:
        //
        // The function now returns void and receives the result
        // as a pointer.
        //
        //     void compiled_closure(
        //         ptr args,
        //         ptr result
        //     )
        // ----------------------------------------------------

        let function_type =
            context
                .void_type()
                .fn_type(
                    &[
                        argument_pointer_type.into(),
                        result_pointer_type.into(),
                    ],
                    false,
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


        builder.position_at_end(
            entry
        );


        // ====================================================
        // Arguments pointer
        // ====================================================

        let argument_pointer =
            function
                .get_nth_param(0)
                .ok_or_else(|| {
                    "missing function argument"
                        .to_string()
                })?
                .into_pointer_value();


        // ====================================================
        // Result pointer
        // ====================================================

        let result_pointer =
            function
                .get_nth_param(1)
                .ok_or_else(|| {
                    "missing result pointer"
                        .to_string()
                })?
                .into_pointer_value();


        // ====================================================
        // Build argument pointers
        // ====================================================

        let arguments =
            self.build_argument_pointers(
                context,
                &builder,
                argument_pointer,
                &closure.arguments,
            )?;


        // ====================================================
        // Lower expression
        // ====================================================

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


        // ====================================================
        // Store result
        // ====================================================

        builder
            .build_store(
                result_pointer,
                value,
            )
            .map_err(|error| {
                format!(
                    "failed to store return value: {:?}",
                    error
                )
            })?;


        // ====================================================
        // Return void
        // ====================================================

        builder
            .build_return(None)
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
                .collect::<Result<
                    Vec<_>,
                    _,
                >>()?;


        let tuple_type =
            context.struct_type(
                &field_types,
                false,
            );


        let mut result =
            Vec::with_capacity(
                argument_types.len()
            );


        for index
            in 0..argument_types.len()
        {
            let pointer =
                builder
                    .build_struct_gep(
                        tuple_type,
                        argument_pointer,
                        index as u32,
                        &format!(
                            "arg{}_ptr",
                            index
                        ),
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
// TypeInfo -> LLVM type
// ============================================================

pub(crate) fn llvm_type<'ctx>(
    context: &'ctx Context,
    type_info: &TypeInfo,
) -> Result<
    BasicTypeEnum<'ctx>,
    String,
> {
    match type_info {
        TypeInfo::F32 =>
            Ok(
                context
                    .f32_type()
                    .into()
            ),

        TypeInfo::F64 =>
            Ok(
                context
                    .f64_type()
                    .into()
            ),

        TypeInfo::I8
        | TypeInfo::U8 =>
            Ok(
                context
                    .i8_type()
                    .into()
            ),

        TypeInfo::I16
        | TypeInfo::U16 =>
            Ok(
                context
                    .i16_type()
                    .into()
            ),

        TypeInfo::I32
        | TypeInfo::U32 =>
            Ok(
                context
                    .i32_type()
                    .into()
            ),

        TypeInfo::I64
        | TypeInfo::U64 =>
            Ok(
                context
                    .i64_type()
                    .into()
            ),

        TypeInfo::I128
        | TypeInfo::U128 =>
            Ok(
                context
                    .i128_type()
                    .into()
            ),

        TypeInfo::Bool =>
            Ok(
                context
                    .bool_type()
                    .into()
            ),

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
                    .collect::<Result<
                        Vec<_>,
                        _,
                    >>()?;


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