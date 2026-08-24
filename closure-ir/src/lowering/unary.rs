use inkwell::{builder::Builder, context::Context, values::{FunctionValue, PointerValue, BasicValueEnum}};
use crate::{expr::Expr, operators::UnaryOp, types::TypeInfo};
use super::{LoweredValue, Lowering};

impl Lowering {
    pub(crate) fn lower_unary(&self, context: &'ctx Context, builder: &Builder<'ctx>, function: FunctionValue<'ctx>, arguments: &[PointerValue<'ctx>], argument_types: &[TypeInfo], expected_type: &TypeInfo, operand: &Expr, operation: UnaryOp) -> Result<LoweredValue<'ctx>, String> {
        let operand = self.materialize_value(context, builder, self.lower_expr(context, builder, function, arguments, argument_types, expected_type, operand)?)?;
        match operation {
            UnaryOp::Not => { let value = match operand { BasicValueEnum::IntValue(value) => value, _ => return Err("unary ! requires an integer or bool operand".to_string()) }; Ok(LoweredValue::Value(builder.build_not(value, "not").map_err(|e| format!("failed to build not: {:?}", e))?.into())) }
            UnaryOp::Neg => match operand { BasicValueEnum::IntValue(value) => Ok(LoweredValue::Value(builder.build_int_neg(value, "neg").map_err(|e| format!("failed to build integer negation: {:?}", e))?.into())), BasicValueEnum::FloatValue(value) => Ok(LoweredValue::Value(builder.build_float_neg(value, "neg").map_err(|e| format!("failed to build float negation: {:?}", e))?.into())), _ => Err("unary - requires numeric operand".to_string()) },
        }
    }
}
