use inkwell::{
    builder::Builder,
    context::Context,
    types::BasicTypeEnum,
    values::{BasicValueEnum, FunctionValue, PointerValue},
    IntPredicate,
};

use crate::{
    compiler::llvm_type,
    expr::{Block, Statement},
    lowering::{LoweredValue, Lowering},
    types::TypeInfo,
};

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
    let (_, _, value) = lower_block(
        context,
        builder,
        function,
        arguments,
        argument_types,
        block,
        false,
    )?;

    let value = value.ok_or_else(|| "closure block has no result expression".to_string())?;
    let value = lowering.materialize_value(context, builder, value)?;
    let expected = llvm_type(context, return_type)?;

    if value.get_type() != expected {
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
    block: &Block,
    allow_no_result: bool,
) -> Result<(Vec<PointerValue<'ctx>>, Vec<TypeInfo>, Option<LoweredValue<'ctx>>), String> {
    let lowering = Lowering;
    let mut pointers = arguments.to_vec();
    let mut types = argument_types.to_vec();

    for statement in &block.statements {
        match statement {
            Statement::Let { local, type_info, value, .. } => {
                if *local != pointers.len().saturating_sub(arguments.len()) {
                    return Err(format!("invalid local index {}", local));
                }

                let llvm_value_type = llvm_type(context, type_info)?;
                let pointer = builder
                    .build_alloca(llvm_value_type, &format!("local_{}", local))
                    .map_err(|error| format!("failed to allocate local {}: {:?}", local, error))?;

                let value = lowering.lower_expr(
                    context,
                    builder,
                    function,
                    &pointers,
                    &types,
                    type_info,
                    value,
                )?;
                let value = lowering.materialize_value(context, builder, value)?;
                builder
                    .build_store(pointer, value)
                    .map_err(|error| format!("failed to initialize local {}: {:?}", local, error))?;

                pointers.push(pointer);
                types.push(type_info.clone());
            }

            Statement::Assign { local, value } => {
                let offset = arguments.len();
                let pointer = *pointers.get(offset + local).ok_or_else(|| {
                    format!("local index {} out of bounds", local)
                })?;
                let type_info = types.get(offset + local).ok_or_else(|| {
                    format!("local type index {} out of bounds", local)
                })?;

                let value = lowering.lower_expr(
                    context,
                    builder,
                    function,
                    &pointers,
                    &types,
                    type_info,
                    value,
                )?;
                let value = lowering.materialize_value(context, builder, value)?;
                builder
                    .build_store(pointer, value)
                    .map_err(|error| format!("failed to assign local {}: {:?}", local, error))?;
            }

            Statement::While { condition, body } => {
                let condition_block = context.append_basic_block(function, "while_condition");
                let body_block = context.append_basic_block(function, "while_body");
                let exit_block = context.append_basic_block(function, "while_exit");

                builder
                    .build_unconditional_branch(condition_block)
                    .map_err(|error| format!("failed to enter while loop: {:?}", error))?;

                builder.position_at_end(condition_block);
                let condition_value = lowering.lower_expr(
                    context,
                    builder,
                    function,
                    &pointers,
                    &types,
                    &TypeInfo::Bool,
                    condition,
                )?;
                let condition_value = lowering.materialize_value(context, builder, condition_value)?;
                let condition_value = match condition_value {
                    BasicValueEnum::IntValue(value) if value.get_type().get_bit_width() == 1 => value,
                    _ => return Err("while condition must be bool".to_string()),
                };

                builder
                    .build_conditional_branch(condition_value, body_block, exit_block)
                    .map_err(|error| format!("failed to build while branch: {:?}", error))?;

                builder.position_at_end(body_block);
                let _ = lower_block(
                    context,
                    builder,
                    function,
                    &pointers,
                    &types,
                    body,
                    true,
                )?;

                let body_end = builder
                    .get_insert_block()
                    .ok_or_else(|| "missing while body block".to_string())?;
                if body_end.get_terminator().is_none() {
                    builder
                        .build_unconditional_branch(condition_block)
                        .map_err(|error| format!("failed to loop back to condition: {:?}", error))?;
                }

                builder.position_at_end(exit_block);
            }
        }
    }

    let result = match &block.result {
        Some(expr) => Some(lowering.lower_expr(
            context,
            builder,
            function,
            &pointers,
            &types,
            block_result_type(context, block, argument_types)?,
            expr,
        )?),
        None => None,
    };

    if result.is_none() && !allow_no_result {
        return Err("closure block has no result expression".to_string());
    }

    Ok((pointers, types, result))
}

fn block_result_type<'ctx>(
    _context: &'ctx Context,
    _block: &Block,
    argument_types: &[TypeInfo],
) -> Result<TypeInfo, String> {
    // The expression lowering only needs a contextual type for literals.
    // Use the final available type when the block result is a local/argument;
    // callers with typed literals pass their type through expression lowering.
    argument_types
        .first()
        .cloned()
        .ok_or_else(|| "cannot infer block result type".to_string())
}
