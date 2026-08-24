use inkwell::{builder::Builder, context::Context, values::{FunctionValue, PointerValue}};
use crate::{expr::Expr, types::TypeInfo};
use super::{LoweredValue, Lowering};

impl Lowering {
    pub(crate) fn lower_tuple(&self, context: &'ctx Context, builder: &Builder<'ctx>, function: FunctionValue<'ctx>, arguments: &[PointerValue<'ctx>], argument_types: &[TypeInfo], expected_type: &TypeInfo, elements: &[Expr]) -> Result<LoweredValue<'ctx>, String> {
        let fields = match expected_type { TypeInfo::Struct { fields, .. } => fields, _ => return Err("tuple expression requires a struct/tuple expected type".to_string()) };
        if fields.len() != elements.len() { return Err(format!("tuple has {} elements but expected {}", elements.len(), fields.len())); }
        let struct_type = Self::llvm_struct_type(context, expected_type)?;
        let mut tuple_value = struct_type.const_zero();
        for (index, (element, field)) in elements.iter().zip(fields.iter()).enumerate() { let value = self.lower_expr(context, builder, function, arguments, argument_types, &field.type_info, element)?; let value = self.materialize_value(context, builder, value)?; tuple_value = builder.build_insert_value(tuple_value, value, index as u32, &format!("tuple_{}", index)).map_err(|error| format!("failed to insert tuple element {}: {:?}", index, error))?.into_struct_value(); }
        Ok(LoweredValue::Value(tuple_value.into()))
    }
}
