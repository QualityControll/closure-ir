use proc_macro2::TokenStream;
use syn::{Expr as SynExpr, Type};
use crate::parser::ClosureArgument;
use super::{binary, field, if_else, literal, path, tuple, unary};

#[derive(Clone)]
pub(crate) struct LocalVariable {
    pub(crate) name: syn::Ident,
    pub(crate) index: usize,
    pub(crate) type_info: Option<Type>,
    pub(crate) mutable: bool,
}

pub(crate) fn lower_expr(
    expr: &SynExpr,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    match expr {
        SynExpr::Path(path) => path::lower_path(path, arguments, locals, expected_type),
        SynExpr::Lit(literal) => literal::lower_literal(literal, expected_type),
        SynExpr::Binary(binary) => binary::lower_binary(binary, arguments, locals, expected_type),
        SynExpr::Unary(unary) => unary::lower_unary(unary, arguments, locals, expected_type),
        SynExpr::If(if_expr) => if_else::lower_if(if_expr, arguments, locals, expected_type),
        SynExpr::Tuple(tuple) => tuple::lower_tuple(tuple, arguments, locals),
        SynExpr::Field(field) => field::lower_field(field, arguments, locals),
        SynExpr::Paren(paren) => lower_expr(&paren.expr, arguments, locals, expected_type),
        _ => Err(syn::Error::new_spanned(expr, "unsupported expression")),
    }
}

pub(crate) fn expression_type(
    expr: &SynExpr,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
) -> Option<Type> {
    match expr {
        SynExpr::Path(path) if path.path.segments.len() == 1 => {
            let name = &path.path.segments[0].ident;
            if let Some(argument) = arguments.iter().find(|argument| &argument.name == name) {
                return Some(argument.type_info.clone());
            }
            locals.iter().rev().find(|local| &local.name == name).and_then(|local| local.type_info.clone())
        }
        SynExpr::Paren(paren) => expression_type(&paren.expr, arguments, locals),
        SynExpr::Binary(binary) => expression_type(&binary.left, arguments, locals)
            .or_else(|| expression_type(&binary.right, arguments, locals)),
        _ => None,
    }
}
