use inkwell::{builder::Builder, context::Context, values::{BasicValueEnum, FunctionValue, PointerValue}};
use crate::{expr::Expr, types::TypeInfo};
use super::{LoweredValue, Lowering};

impl Lowering {
    pub(crate) fn lower_if_else(
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
        let condition = self.lower_expr(context, builder, function, arguments, argument_types, &TypeInfo::Bool, condition)?;
        let condition = self.materialize_value(context, builder, condition)?;
        let condition = match condition {
            BasicValueEnum::IntValue(value) if value.get_type().get_bit_width() == 1 => value,
            _ => return Err("if condition must be bool".to_string()),
        };
        let then_block = context.append_basic_block(function, "then");
        let else_block = context.append_basic_block(function, "else");
        let merge_block = context.append_basic_block(function, "if_merge");
        builder.build_conditional_branch(condition, then_block, else_block).map_err(|error| format!("failed to build conditional branch: {:?}", error))?;
        builder.position_at_end(then_block);
        let then_value = self.materialize_value(context, builder, self.lower_expr(context, builder, function, arguments, argument_types, expected_type, then_branch)?)?;
        let then_end = builder.get_insert_block().ok_or_else(|| "missing then block".to_string())?;
        if then_end.get_terminator().is_none() { builder.build_unconditional_branch(merge_block).map_err(|error| format!("failed to branch from then block: {:?}", error))?; }
        builder.position_at_end(else_block);
        let else_value = self.materialize_value(context, builder, self.lower_expr(context, builder, function, arguments, argument_types, expected_type, else_branch)?)?;
        let else_end = builder.get_insert_block().ok_or_else(|| "missing else block".to_string())?;
        if else_end.get_terminator().is_none() { builder.build_unconditional_branch(merge_block).map_err(|error| format!("failed to branch from else block: {:?}", error))?; }
        builder.position_at_end(merge_block);
        let phi = builder.build_phi(crate::compiler::llvm_type(context, expected_type)?, "if_result").map_err(|error| format!("failed to build if/else PHI: {:?}", error))?;
        phi.add_incoming(&[(&then_value, then_end), (&else_value, else_end)]);
        Ok(LoweredValue::Value(phi.as_basic_value()))
    }
}
