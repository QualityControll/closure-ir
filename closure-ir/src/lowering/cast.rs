use inkwell::{builder::Builder, context::Context, values::{BasicValueEnum, FunctionValue}};
use crate::{expr::Expr, types::TypeInfo};
use super::LoweredValue;

pub(crate) fn lower_cast<'ctx>(lowering: &super::Lowering, context: &'ctx Context, builder: &Builder<'ctx>, function: FunctionValue<'ctx>, arguments: &[inkwell::values::PointerValue<'ctx>], argument_types: &[TypeInfo], source: &TypeInfo, target: &TypeInfo, expr: &Expr) -> Result<LoweredValue<'ctx>, String> {
    if !source.is_numeric() || !target.is_numeric() { return Err("casts are only supported between numeric types".into()); }
    let lowered = lowering.lower_expr(context, builder, function, arguments, argument_types, source, expr)?;
    let value = lowering.materialize_value(context, builder, lowered)?;
    if source == target { return Ok(LoweredValue::Value(value)); }
    let result = match (source.is_integer(), target.is_integer(), value) {
        (true, true, BasicValueEnum::IntValue(v)) => {
            let t = crate::compiler::llvm_type(context, target)?.into_int_type();
            let cast = if bits(target) < bits(source) { builder.build_int_truncate(v, t, "cast_trunc") } else if bits(target) > bits(source) { if source.is_signed_integer() { builder.build_int_s_extend(v, t, "cast_sext") } else { builder.build_int_z_extend(v, t, "cast_zext") } } else { builder.build_int_cast(v, t, "cast_int") }.map_err(|e| format!("failed to lower integer cast: {:?}", e))?;
            cast.into()
        }
        (true, false, BasicValueEnum::IntValue(v)) => { let t = crate::compiler::llvm_type(context, target)?.into_float_type(); if source.is_signed_integer() { builder.build_signed_int_to_float(v, t, "cast_sitofp") } else { builder.build_unsigned_int_to_float(v, t, "cast_uitofp") }.map_err(|e| format!("failed to lower integer-to-float cast: {:?}", e))?.into() }
        (false, true, BasicValueEnum::FloatValue(v)) => { let t = crate::compiler::llvm_type(context, target)?.into_int_type(); if target.is_signed_integer() { builder.build_float_to_signed_int(v, t, "cast_fptosi") } else { builder.build_float_to_unsigned_int(v, t, "cast_fptoui") }.map_err(|e| format!("failed to lower float-to-integer cast: {:?}", e))?.into() }
        (false, false, BasicValueEnum::FloatValue(v)) => { let t = crate::compiler::llvm_type(context, target)?.into_float_type(); builder.build_float_cast(v, t, "cast_fp").map_err(|e| format!("failed to lower float cast: {:?}", e))?.into() }
        _ => return Err("cast operand has an unexpected LLVM value type".into()),
    };
    Ok(LoweredValue::Value(result))
}
fn bits(t: &TypeInfo) -> u32 { match t { TypeInfo::Bool=>1, TypeInfo::I8|TypeInfo::U8=>8, TypeInfo::I16|TypeInfo::U16=>16, TypeInfo::I32|TypeInfo::U32=>32, TypeInfo::I64|TypeInfo::U64|TypeInfo::Usize=>64, TypeInfo::I128|TypeInfo::U128=>128, _=>0 } }
