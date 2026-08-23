use inkwell::{builder::Builder, context::Context, values::{BasicValueEnum, FunctionValue, PointerValue}};
use crate::{compiler::llvm_type, expr::{Block, Statement}, lowering::{LoweredValue, Lowering}, types::TypeInfo};

pub(crate) fn lower_closure_block<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    arguments: &[PointerValue<'ctx>],
    argument_types: &[TypeInfo],
    return_type: &TypeInfo,
    block: &Block,
) -> Result<BasicValueEnum<'ctx>, String> {
    let lowering = Lowering;
    let (_, _, value) = lower_block(context, builder, function, arguments, argument_types, argument_types.len(), block, Some(return_type))?;
    let value = value.ok_or_else(|| "closure block has no result expression".to_string())?;
    let value = lowering.materialize_value(context, builder, value)?;
    if value.get_type() != llvm_type(context, return_type)? {
        return Err("closure result type does not match declared return type".to_string());
    }
    Ok(value)
}

fn lower_block<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    function: FunctionValue<'ctx>,
    arguments: &[PointerValue<'ctx>],
    argument_types: &[TypeInfo],
    argument_count: usize,
    block: &Block,
    expected_result_type: Option<&TypeInfo>,
) -> Result<(Vec<PointerValue<'ctx>>, Vec<TypeInfo>, Option<LoweredValue<'ctx>>), String> {
    let lowering = Lowering;
    let mut pointers = arguments.to_vec();
    let mut types = argument_types.to_vec();

    for statement in &block.statements {
        match statement {
            Statement::Let { local, type_info, value, .. } => {
                let expected_local = pointers.len().saturating_sub(argument_count);
                if *local != expected_local {
                    return Err(format!("invalid local index {}", local));
                }
                let pointer = builder.build_alloca(llvm_type(context, type_info)?, &format!("local_{}", local))
                    .map_err(|error| format!("failed to allocate local {}: {:?}", local, error))?;
                let value = lowering.lower_expr(context, builder, function, &pointers, &types, type_info, value)?;
                let value = lowering.materialize_value(context, builder, value)?;
                builder.build_store(pointer, value).map_err(|error| format!("failed to initialize local {}: {:?}", local, error))?;
                pointers.push(pointer);
                types.push(type_info.clone());
            }
            Statement::Assign { local, value } => {
                let index = argument_count + local;
                let pointer = *pointers.get(index).ok_or_else(|| format!("local index {} out of bounds", local))?;
                let type_info = types.get(index).ok_or_else(|| format!("local type index {} out of bounds", local))?;
                let value = lowering.lower_expr(context, builder, function, &pointers, &types, type_info, value)?;
                let value = lowering.materialize_value(context, builder, value)?;
                builder.build_store(pointer, value).map_err(|error| format!("failed to assign local {}: {:?}", local, error))?;
            }
            Statement::While { condition, body } => {
                let condition_block = context.append_basic_block(function, "while_condition");
                let body_block = context.append_basic_block(function, "while_body");
                let exit_block = context.append_basic_block(function, "while_exit");
                builder.build_unconditional_branch(condition_block).map_err(|error| format!("failed to enter while loop: {:?}", error))?;

                builder.position_at_end(condition_block);
                let condition = lowering.lower_expr(context, builder, function, &pointers, &types, &TypeInfo::Bool, condition)?;
                let condition = lowering.materialize_value(context, builder, condition)?;
                let condition = match condition {
                    BasicValueEnum::IntValue(value) if value.get_type().get_bit_width() == 1 => value,
                    _ => return Err("while condition must be bool".to_string()),
                };
                builder.build_conditional_branch(condition, body_block, exit_block).map_err(|error| format!("failed to build while branch: {:?}", error))?;

                builder.position_at_end(body_block);
                let _ = lower_block(context, builder, function, &pointers, &types, argument_count, body, None)?;
                let body_end = builder.get_insert_block().ok_or_else(|| "missing while body block".to_string())?;
                if body_end.get_terminator().is_none() {
                    builder.build_unconditional_branch(condition_block).map_err(|error| format!("failed to loop back to condition: {:?}", error))?;
                }
                builder.position_at_end(exit_block);
            }
        }
    }

    let result = match (&block.result, expected_result_type) {
        (Some(expr), Some(type_info)) => Some(lowering.lower_expr(context, builder, function, &pointers, &types, type_info, expr)?),
        (Some(_), None) => return Err("block result has no expected type".to_string()),
        (None, Some(_)) => return Err("closure block has no result expression".to_string()),
        (None, None) => None,
    };

    Ok((pointers, types, result))
}
