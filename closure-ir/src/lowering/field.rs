use inkwell::{builder::Builder, context::Context, values::{FunctionValue, PointerValue}};
use crate::{expr::Expr, types::TypeInfo};
use super::{LoweredValue, Lowering};

impl<'ctx> Lowering {
    pub(crate) fn lower_field(
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
        let object = self.lower_expr(context, builder, function, arguments, argument_types, expected_type, object)?;
        let (object_pointer, object_type) = match object {
            LoweredValue::Pointer { pointer, type_info } => (pointer, type_info),
            LoweredValue::Value(_) => return Err(format!("cannot access field `{}` on a value", name)),
        };
        let fields = match &object_type {
            TypeInfo::Struct { fields, .. } => fields,
            _ => return Err(format!("cannot access field `{}` on non-struct type", name)),
        };
        let (field_index, field_type) = fields.iter().enumerate()
            .find_map(|(index, field)| (field.name == name).then(|| (index, field.type_info.clone())))
            .ok_or_else(|| format!("field `{}` not found", name))?;
        let struct_type = Self::llvm_struct_type(context, &object_type)?;
        let field_pointer = builder.build_struct_gep(struct_type, object_pointer, field_index as u32, &format!("{}_ptr", name))
            .map_err(|error| format!("failed to build GEP for field `{}`: {:?}", name, error))?;
        Ok(LoweredValue::Pointer { pointer: field_pointer, type_info: field_type })
    }
}
