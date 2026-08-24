use inkwell::{
    builder::Builder,
    context::Context,
    types::BasicTypeEnum,
    values::{BasicValueEnum, FunctionValue, PointerValue},
};

use crate::{expr::Expr, types::TypeInfo};

pub(crate) struct Lowering;

pub(crate) enum LoweredValue<'ctx> {
    Value(BasicValueEnum<'ctx>),
    Pointer {
        pointer: PointerValue<'ctx>,
        type_info: TypeInfo,
    },
}

mod binary;
mod control;
mod field;
mod constant;
mod materialize;
mod unary;
mod tuple;

impl<'ctx> Lowering {
    pub(crate) fn lower_expr(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        arguments: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        expr: &Expr,
    ) -> Result<LoweredValue<'ctx>, String> {
        match expr {
            Expr::Argument(index) => {
                let pointer = *arguments.get(*index)
                    .ok_or_else(|| format!("argument index {} out of bounds", index))?;
                let type_info = argument_types.get(*index).cloned()
                    .ok_or_else(|| format!("argument type index {} out of bounds", index))?;
                Ok(LoweredValue::Pointer { pointer, type_info })
            }
            Expr::Constant(value) => self.lower_constant(context, value),
            Expr::Field { object, name } => self.lower_field(
                context, builder, function, arguments, argument_types,
                expected_type, object, name,
            ),
            Expr::Tuple { elements } => self.lower_tuple(
                context, builder, function, arguments, argument_types,
                expected_type, elements,
            ),
            Expr::IfElse { condition, then_branch, else_branch } => self.lower_if_else(
                context, builder, function, arguments, argument_types,
                expected_type, condition, then_branch, else_branch,
            ),
            Expr::Add { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Add),
            Expr::Sub { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Sub),
            Expr::Mul { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Mul),
            Expr::Div { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Div),
            Expr::Rem { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Rem),
            Expr::Eq { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Eq),
            Expr::Ne { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Ne),
            Expr::Lt { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Lt),
            Expr::Le { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Le),
            Expr::Gt { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Gt),
            Expr::Ge { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Ge),
            Expr::And { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::And),
            Expr::Or { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Or),
            Expr::BitAnd { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::BitAnd),
            Expr::BitOr { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::BitOr),
            Expr::BitXor { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::BitXor),
            Expr::Shl { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Shl),
            Expr::Shr { lhs, rhs } => self.lower_binary(context, builder, function, arguments, argument_types, expected_type, lhs, rhs, crate::operators::BinaryOp::Shr),
            Expr::Not { operand } => self.lower_unary(context, builder, function, arguments, argument_types, expected_type, operand, crate::operators::UnaryOp::Not),
            Expr::Neg { operand } => self.lower_unary(context, builder, function, arguments, argument_types, expected_type, operand, crate::operators::UnaryOp::Neg),
        }
    }

    pub(crate) fn llvm_struct_type(
        context: &'ctx Context,
        type_info: &TypeInfo,
    ) -> Result<inkwell::types::StructType<'ctx>, String> {
        match crate::compiler::llvm_type(context, type_info)? {
            BasicTypeEnum::StructType(value) => Ok(value),
            _ => Err("expected struct LLVM type".to_string()),
        }
    }
}
