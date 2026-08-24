use crate::expr::{Expr, Intrinsic};
use crate::types::{TypeInfo, FieldInfo};
use crate::value::Value;

#[derive(Debug, Clone, Copy)]
pub(crate) enum BinaryOp { Add, Sub, Mul, Div, Rem, Eq, Ne, Lt, Le, Gt, Ge, And, Or, BitAnd, BitOr, BitXor, Shl, Shr }
#[derive(Debug, Clone, Copy)]
pub(crate) enum UnaryOp { Not, Neg }

pub(crate) fn expression_type(argument_types: &[TypeInfo], expr: &Expr) -> Result<TypeInfo, String> {
    match expr {
        Expr::Argument(index) => argument_types.get(*index).cloned().ok_or_else(|| format!("argument index {} out of bounds", index)),
        Expr::Constant(value) => Ok(match value {
            Value::F32(_) => TypeInfo::F32, Value::F64(_) => TypeInfo::F64,
            Value::I8(_) => TypeInfo::I8, Value::I16(_) => TypeInfo::I16, Value::I32(_) => TypeInfo::I32, Value::I64(_) => TypeInfo::I64, Value::I128(_) => TypeInfo::I128,
            Value::U8(_) => TypeInfo::U8, Value::U16(_) => TypeInfo::U16, Value::U32(_) => TypeInfo::U32, Value::U64(_) => TypeInfo::U64, Value::U128(_) => TypeInfo::U128,
            Value::Bool(_) => TypeInfo::Bool,
        }),
        Expr::Field { object, name } => {
            let object_type = expression_type(argument_types, object)?;
            let fields = match object_type { TypeInfo::Struct { fields, .. } => fields, _ => return Err(format!("cannot access field `{}` on non-struct type", name)) };
            fields.into_iter().find(|field| field.name == *name).map(|field| field.type_info).ok_or_else(|| format!("field `{}` not found", name))
        }
        Expr::Tuple { elements } => {
            let fields = elements.iter().enumerate().map(|(index, element)| Ok(FieldInfo { name: index.to_string(), type_info: expression_type(argument_types, element)? })).collect::<Result<Vec<_>, String>>()?;
            Ok(TypeInfo::Struct { name: "tuple".to_string(), fields })
        }
        Expr::Intrinsic { intrinsic, arguments } => {
            let arity = match intrinsic { Intrinsic::Min | Intrinsic::Max | Intrinsic::Pow => 2, _ => 1 };
            if arguments.len() != arity { return Err(format!("intrinsic {:?} expects {} argument(s)", intrinsic, arity)); }
            let first_type = expression_type(argument_types, &arguments[0])?;
            if !first_type.is_numeric() { return Err(format!("intrinsic {:?} requires numeric arguments", intrinsic)); }
            for argument in &arguments[1..] {
                let argument_type = expression_type(argument_types, argument)?;
                if argument_type != first_type { return Err(format!("intrinsic {:?} arguments must have the same type", intrinsic)); }
            }
            match intrinsic {
                Intrinsic::Sqrt | Intrinsic::Sin | Intrinsic::Cos | Intrinsic::Tan |
                Intrinsic::Exp | Intrinsic::Log | Intrinsic::Floor | Intrinsic::Ceil |
                Intrinsic::Round | Intrinsic::Pow => {
                    if !first_type.is_float() { return Err(format!("intrinsic {:?} requires f32 or f64", intrinsic)); }
                }
                Intrinsic::Abs | Intrinsic::Min | Intrinsic::Max => {}
            }
            Ok(first_type)
        }
        Expr::Add { lhs, .. } | Expr::Sub { lhs, .. } | Expr::Mul { lhs, .. } | Expr::Div { lhs, .. } | Expr::Rem { lhs, .. } | Expr::BitAnd { lhs, .. } | Expr::BitOr { lhs, .. } | Expr::BitXor { lhs, .. } | Expr::Shl { lhs, .. } | Expr::Shr { lhs, .. } | Expr::Neg { operand: lhs } => expression_type(argument_types, lhs),
        Expr::Eq { .. } | Expr::Ne { .. } | Expr::Lt { .. } | Expr::Le { .. } | Expr::Gt { .. } | Expr::Ge { .. } | Expr::And { .. } | Expr::Or { .. } | Expr::Not { .. } => Ok(TypeInfo::Bool),
        Expr::IfElse { then_branch, else_branch, .. } => {
            let then_type = expression_type(argument_types, then_branch)?;
            let else_type = expression_type(argument_types, else_branch)?;
            if then_type != else_type { return Err("if/else branches must have the same type".to_string()); }
            Ok(then_type)
        }
    }
}

pub(crate) fn binary_operand_type(argument_types: &[TypeInfo], lhs: &Expr, rhs: &Expr, expected_type: &TypeInfo, operation: &BinaryOp) -> Result<TypeInfo, String> {
    let lhs_type = expression_type(argument_types, lhs)?;
    let rhs_type = expression_type(argument_types, rhs)?;
    match operation {
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => { if lhs_type != rhs_type { return Err("comparison operands must have the same type".to_string()); } Ok(lhs_type) }
        BinaryOp::And | BinaryOp::Or => { if !lhs_type.is_bool() || !rhs_type.is_bool() { return Err("logical &&/|| operands must be bool".to_string()); } Ok(TypeInfo::Bool) }
        _ => { if lhs_type != rhs_type { return Err(format!("binary operands must have the same type, got {:?} and {:?} (expected {:?})", lhs_type, rhs_type, expected_type)); } Ok(lhs_type) }
    }
}
