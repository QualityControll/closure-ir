use crate::expr::{Expr, Intrinsic};
use crate::types::{FieldInfo, TypeInfo};
use crate::value::Value;

#[derive(Debug, Clone, Copy)]
pub(crate) enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum UnaryOp {
    Not,
    Neg,
}

pub(crate) fn expression_type(
    argument_types: &[TypeInfo],
    expr: &Expr,
) -> Result<TypeInfo, String> {
    expression_type_with_captures(&[], argument_types, expr)
}

pub(crate) fn expression_type_with_captures(
    capture_types: &[TypeInfo],
    argument_types: &[TypeInfo],
    expr: &Expr,
) -> Result<TypeInfo, String> {
    match expr {
        Expr::Argument(index) => argument_types
            .get(*index)
            .cloned()
            .ok_or_else(|| format!("argument index {} out of bounds", index)),
        Expr::Capture(index) => capture_types
            .get(*index)
            .cloned()
            .ok_or_else(|| format!("capture index {} out of bounds", index)),
        Expr::Constant(value) => Ok(match value {
            Value::F32(_) => TypeInfo::F32,
            Value::F64(_) => TypeInfo::F64,
            Value::I8(_) => TypeInfo::I8,
            Value::I16(_) => TypeInfo::I16,
            Value::I32(_) => TypeInfo::I32,
            Value::I64(_) => TypeInfo::I64,
            Value::I128(_) => TypeInfo::I128,
            Value::U8(_) => TypeInfo::U8,
            Value::U16(_) => TypeInfo::U16,
            Value::U32(_) => TypeInfo::U32,
            Value::U64(_) => TypeInfo::U64,
            Value::U128(_) => TypeInfo::U128,
            Value::Usize(_) => TypeInfo::Usize,
            Value::Bool(_) => TypeInfo::Bool,
            Value::Array(values) => {
                let first = values
                    .first()
                    .ok_or("empty array constants are not supported")?;
                TypeInfo::Array {
                    element: Box::new(expression_type_with_captures(
                        capture_types,
                        argument_types,
                        &Expr::Constant(first.clone()),
                    )?),
                    length: values.len(),
                }
            }
        }),
        Expr::Cast {
            expr,
            source_type,
            target_type,
        } => {
            let actual = expression_type_with_captures(capture_types, argument_types, expr)?;
            if actual != *source_type {
                return Err(format!(
                    "cast source type mismatch: expression has type {:?}, cast declares {:?}",
                    actual, source_type
                ));
            }
            Ok(target_type.clone())
        }
        Expr::Index { sequence, index } => {
            let st = expression_type_with_captures(capture_types, argument_types, sequence)?;
            let it = expression_type_with_captures(capture_types, argument_types, index)?;
            if it != TypeInfo::Usize {
                return Err("sequence index must be usize".into());
            }
            st.indexed_element_type()
                .cloned()
                .ok_or_else(|| "cannot index non-indexable type".into())
        }
        Expr::Len { sequence } => {
            let st = expression_type_with_captures(capture_types, argument_types, sequence)?;
            if !st.is_indexable() {
                return Err("len() requires an indexable sequence".into());
            }
            Ok(TypeInfo::Usize)
        }
        Expr::Field { object, name } => {
            let ot = expression_type_with_captures(capture_types, argument_types, object)?;
            let fields = match ot {
                TypeInfo::Struct { fields, .. } => fields,
                _ => return Err(format!("cannot access field `{}` on non-struct type", name)),
            };
            fields
                .into_iter()
                .find(|f| f.name == *name)
                .map(|f| f.type_info)
                .ok_or_else(|| format!("field `{}` not found", name))
        }
        Expr::Struct { type_info, fields } => {
            let expected = match type_info {
                TypeInfo::Struct { fields, .. } => fields,
                _ => return Err("struct literal requires a struct type".into()),
            };
            if fields.len() != expected.len() {
                return Err(format!(
                    "struct literal has {} fields but expected {}",
                    fields.len(),
                    expected.len()
                ));
            }
            for (name, value) in fields {
                let ef = expected
                    .iter()
                    .find(|f| f.name == *name)
                    .ok_or_else(|| format!("field `{}` not found in struct", name))?;
                let actual = expression_type_with_captures(capture_types, argument_types, value)?;
                if actual != ef.type_info {
                    return Err(format!(
                        "field `{}` has type {:?}, expected {:?}",
                        name, actual, ef.type_info
                    ));
                }
            }
            Ok(type_info.clone())
        }
        Expr::Tuple { elements } => {
            let fields = elements
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    Ok(FieldInfo {
                        name: i.to_string(),
                        type_info: expression_type_with_captures(capture_types, argument_types, e)?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(TypeInfo::Struct {
                name: "tuple".to_string(),
                fields,
            })
        }
        Expr::Intrinsic {
            intrinsic,
            arguments,
        } => {
            let arity = match intrinsic {
                Intrinsic::Min | Intrinsic::Max | Intrinsic::Pow => 2,
                _ => 1,
            };
            if arguments.len() != arity {
                return Err(format!(
                    "{:?} expects {} argument(s), got {}",
                    intrinsic,
                    arity,
                    arguments.len()
                ));
            }
            let first =
                expression_type_with_captures(capture_types, argument_types, &arguments[0])?;
            if !matches!(first, TypeInfo::F32 | TypeInfo::F64) {
                return Err(format!("{:?} requires floating-point arguments", intrinsic));
            }
            for a in &arguments[1..] {
                if expression_type_with_captures(capture_types, argument_types, a)? != first {
                    return Err(format!("{:?} arguments must have the same type", intrinsic));
                }
            }
            Ok(first)
        }
        Expr::ExternalCall {
            arguments,
            return_type,
            ..
        } => {
            for _ in arguments {}
            Ok(return_type.clone())
        }
        Expr::Add { lhs, .. }
        | Expr::Sub { lhs, .. }
        | Expr::Mul { lhs, .. }
        | Expr::Div { lhs, .. }
        | Expr::Rem { lhs, .. }
        | Expr::BitAnd { lhs, .. }
        | Expr::BitOr { lhs, .. }
        | Expr::BitXor { lhs, .. }
        | Expr::Shl { lhs, .. }
        | Expr::Shr { lhs, .. }
        | Expr::Neg { operand: lhs } => {
            expression_type_with_captures(capture_types, argument_types, lhs)
        }
        Expr::Eq { .. }
        | Expr::Ne { .. }
        | Expr::Lt { .. }
        | Expr::Le { .. }
        | Expr::Gt { .. }
        | Expr::Ge { .. }
        | Expr::And { .. }
        | Expr::Or { .. }
        | Expr::Not { .. } => Ok(TypeInfo::Bool),
        Expr::IfElse {
            then_branch,
            else_branch,
            ..
        } => {
            let t = expression_type_with_captures(capture_types, argument_types, then_branch)?;
            let e = expression_type_with_captures(capture_types, argument_types, else_branch)?;
            if t != e {
                return Err("if/else branches must have the same type".into());
            }
            Ok(t)
        }
    }
}

pub(crate) fn binary_operand_type(
    argument_types: &[TypeInfo],
    lhs: &Expr,
    rhs: &Expr,
    expected_type: &TypeInfo,
    operation: &BinaryOp,
) -> Result<TypeInfo, String> {
    let lt = expression_type(argument_types, lhs)?;
    let rt = expression_type(argument_types, rhs)?;
    match operation {
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            if lt != rt {
                return Err("comparison operands must have the same type".into());
            }
            Ok(lt)
        }
        BinaryOp::And | BinaryOp::Or => {
            if !lt.is_bool() || !rt.is_bool() {
                return Err("logical &&/|| operands must be bool".into());
            }
            Ok(TypeInfo::Bool)
        }
        _ => {
            if lt != rt {
                return Err(format!(
                    "binary operands must have the same type, got {:?} and {:?} (expected {:?})",
                    lt, rt, expected_type
                ));
            }
            Ok(lt)
        }
    }
}
