use inkwell::{builder::Builder, context::Context, values::{FunctionValue, PointerValue}};
use crate::{expr::Expr, types::TypeInfo};
use super::{LoweredValue, Lowering};

impl<'ctx> Lowering {
    pub(crate) fn lower_struct(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        type_info: &TypeInfo,
        fields: &[(String, Expr)],
    ) -> Result<LoweredValue<'ctx>, String> {
        if type_info != expected_type {
            return Err(format!("struct literal type {:?} does not match expected type {:?}", type_info, expected_type));
        }
        let expected_fields = match type_info {
            TypeInfo::Struct { fields, .. } => fields,
            _ => return Err("struct literal requires a struct type".into()),
        };
        let llvm_type = Self::llvm_struct_type(context, type_info)?;
        let mut value = llvm_type.const_zero();
        for (name, expr) in fields {
            let index = expected_fields.iter().position(|field| field.name == *name)
                .ok_or_else(|| format!("field `{}` not found in struct", name))?;
            let field_type = &expected_fields[index].type_info;
            let lowered = self.lower_expr(context, builder, function, arguments, argument_types, field_type, expr)?;
            let lowered = self.materialize_value(context, builder, lowered)?;
            value = builder.build_insert_value(value, lowered, index as u32, &format!("struct_{}", name))
                .map_err(|error| format!("failed to insert struct field `{}`: {:?}", name, error))?
                .into_struct_value();
        }
        Ok(LoweredValue::Value(value.into()))
    }
}
