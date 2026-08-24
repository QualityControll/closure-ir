use crate::expr::Expr;
use crate::types::TypeInfo;

/// Infer the type of an expression from argument/local types and expression structure.
/// This intentionally lives independently of LLVM lowering so local declarations can
/// be typed before their values are lowered.
pub(crate) fn infer_expr_type(types: &[TypeInfo], expr: &Expr) -> Result<TypeInfo, String> {
    match expr {
        Expr::Argument(index) => types.get(*index).cloned().ok_or_else(|| format!("argument index {} out of bounds", index)),
        Expr::Constant(value) => Ok(value.type_info()),
        Expr::Cast { target_type, .. } => Ok(target_type.clone()),
        Expr::Index { sequence, .. } => match infer_expr_type(types, sequence)? {
            TypeInfo::Slice(element) => Ok(*element),
            TypeInfo::Array(element, _) => Ok(*element),
            other => Err(format!("cannot index value of type {:?}", other)),
        },
        Expr::Field { expr, field } => {
            let ty = infer_expr_type(types, expr)?;
            match ty {
                TypeInfo::Struct { fields, .. } => fields.get(*field).map(|(_, ty)| ty.clone()).ok_or_else(|| format!("struct field index {} out of bounds", field)),
                other => Err(format!("cannot access field {} on {:?}", field, other)),
            }
        }
        Expr::Unary { op, expr } => {
            let ty = infer_expr_type(types, expr)?;
            match op {
                crate::expr::UnaryOp::Not => Ok(TypeInfo::Bool),
                _ => Ok(ty),
            }
        }
        Expr::Binary { op, left, right } => {
            let left_ty = infer_expr_type(types, left)?;
            let right_ty = infer_expr_type(types, right)?;
            use crate::expr::BinaryOp;
            match op {
                BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge | BinaryOp::And | BinaryOp::Or => Ok(TypeInfo::Bool),
                _ if left_ty == right_ty => Ok(left_ty),
                _ => Err(format!("binary operands have incompatible types: {:?} and {:?}", left_ty, right_ty)),
            }
        }
        Expr::IfElse { then_branch, else_branch, .. } => {
            let then_ty = infer_expr_type(types, then_branch)?;
            let else_ty = infer_expr_type(types, else_branch)?;
            if then_ty != else_ty { return Err("if/else branches must have the same type".to_string()); }
            Ok(then_ty)
        }
        Expr::Intrinsic { intrinsic, .. } => Ok(intrinsic.return_type()),
        Expr::Tuple(elements) => Ok(TypeInfo::Tuple(elements.iter().map(|e| infer_expr_type(types, e)).collect::<Result<Vec<_>, _>>()?)),
        Expr::Struct { fields, type_info, .. } => Ok(type_info.clone()),
    }
}
