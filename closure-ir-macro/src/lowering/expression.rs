use proc_macro2::TokenStream;
use syn::{Expr as SynExpr, Type};
use crate::parser::ClosureArgument;
use super::{binary, field, if_else, intrinsic, literal, path, tuple, unary};

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
        SynExpr::Call(call) => intrinsic::lower_call(call, arguments, locals, expected_type),
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
        SynExpr::Lit(literal) => match &literal.lit {
            syn::Lit::Bool(_) => Some(syn::parse_quote!(bool)),
            syn::Lit::Int(value) => match value.suffix() {
                "i8" => Some(syn::parse_quote!(i8)),
                "i16" => Some(syn::parse_quote!(i16)),
                "i32" | "" => Some(syn::parse_quote!(i32)),
                "i64" => Some(syn::parse_quote!(i64)),
                "i128" => Some(syn::parse_quote!(i128)),
                "u8" => Some(syn::parse_quote!(u8)),
                "u16" => Some(syn::parse_quote!(u16)),
                "u32" => Some(syn::parse_quote!(u32)),
                "u64" => Some(syn::parse_quote!(u64)),
                "u128" => Some(syn::parse_quote!(u128)),
                _ => None,
            },
            syn::Lit::Float(value) => match value.suffix() {
                "f32" => Some(syn::parse_quote!(f32)),
                "f64" | "" => Some(syn::parse_quote!(f64)),
                _ => None,
            },
            _ => None,
        },
        SynExpr::Paren(paren) => expression_type(&paren.expr, arguments, locals),
        SynExpr::Binary(binary) => expression_type(&binary.left, arguments, locals)
            .or_else(|| expression_type(&binary.right, arguments, locals)),
        _ => None,
    }
}
