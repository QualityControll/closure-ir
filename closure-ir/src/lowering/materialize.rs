use inkwell::{builder::Builder, context::Context, values::{BasicValueEnum, FunctionValue, PointerValue}};
use crate::types::TypeInfo;
use super::{LoweredValue, Lowering};

impl<'ctx> Lowering {
    pub(crate) fn materialize_value(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        value: LoweredValue<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, String> {
        match value {
            LoweredValue::Value(value) => Ok(value),
            LoweredValue::Pointer { pointer, type_info } => {
                let llvm_type = crate::compiler::llvm_type(context, &type_info)?;
                builder.build_load(llvm_type, pointer, "load")
                    .map_err(|error| format!("failed to build load: {:?}", error))
            }
        }
    }
}
