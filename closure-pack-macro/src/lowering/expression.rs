use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr as SynExpr, Type};
use crate::parser::ClosureArgument;
use super::{binary, field, if_else, intrinsic, literal, path, structure, tuple, unary};

#[derive(Clone)]
pub(crate) struct LocalVariable {
    pub(crate) name: syn::Ident,
    pub(crate) index: usize,
    pub(crate) type_info: Option<Type>,
    pub(crate) mutable: bool,
}

pub(crate) fn lower_expr(expr: &SynExpr, arguments: &[ClosureArgument], locals: &[LocalVariable], expected_type: Option<&Type>) -> syn::Result<TokenStream> {
    match expr {
        SynExpr::Path(path) => path::lower_path(path, arguments, locals, expected_type),
        SynExpr::Lit(literal) => literal::lower_literal(literal, expected_type),
        SynExpr::Binary(binary) => binary::lower_binary(binary, arguments, locals, expected_type),
        SynExpr::Unary(unary) => unary::lower_unary(unary, arguments, locals, expected_type),
        SynExpr::If(if_expr) => if_else::lower_if(if_expr, arguments, locals, expected_type),
        SynExpr::Tuple(tuple) => tuple::lower_tuple(tuple, arguments, locals),
        SynExpr::Field(field) => field::lower_field(field, arguments, locals),
        SynExpr::Call(call) => intrinsic::lower_call(call, arguments, locals, expected_type),
        SynExpr::MethodCall(method) => {
            if method.method != "len" { return Err(syn::Error::new_spanned(method, "unsupported method")); }
            if !method.args.is_empty() { return Err(syn::Error::new_spanned(method, "len() takes no arguments")); }
            let sequence = lower_expr(&method.receiver, arguments, locals, None)?;
            Ok(quote!(::closure_pack::Expr::Len { sequence: Box::new(#sequence) }))
        }
        SynExpr::Struct(struct_expr) => structure::lower_struct(struct_expr, arguments, locals, expected_type),
        SynExpr::Cast(cast) => {
            let source_type = expression_type(&cast.expr, arguments, locals)
                .ok_or_else(|| syn::Error::new_spanned(&cast.expr, "cannot infer the source type of this cast"))?;
            let target_type = (*cast.ty).clone();
            let value = lower_expr(&cast.expr, arguments, locals, Some(&source_type))?;
            Ok(quote!(::closure_pack::Expr::Cast { expr: Box::new(#value), source_type: <#source_type as ::closure_pack::CompileType>::type_info(), target_type: <#target_type as ::closure_pack::CompileType>::type_info() }))
        }
        SynExpr::Index(index) => {
            let sequence = lower_expr(&index.expr, arguments, locals, None)?;
            let index_expr = lower_expr(&index.index, arguments, locals, Some(&syn::parse_quote!(usize)))?;
            Ok(quote!(::closure_pack::Expr::Index { sequence: Box::new(#sequence), index: Box::new(#index_expr) }))
        }
        SynExpr::Paren(paren) => lower_expr(&paren.expr, arguments, locals, expected_type),
        _ => Err(syn::Error::new_spanned(expr, "unsupported expression")),
    }
}

pub(crate) fn expression_type(expr: &SynExpr, arguments: &[ClosureArgument], locals: &[LocalVariable]) -> Option<Type> {
    match expr {
        SynExpr::Path(path) if path.path.segments.len() == 1 => {
            let name = &path.path.segments[0].ident;
            if let Some(argument) = arguments.iter().find(|a| &a.name == name) { return Some(argument.type_info.clone()); }
            locals.iter().rev().find(|l| &l.name == name).and_then(|l| l.type_info.clone())
        }
        SynExpr::Lit(literal) => match &literal.lit {
            syn::Lit::Bool(_) => Some(syn::parse_quote!(bool)),
            syn::Lit::Int(v) => match v.suffix() {
                "i8" => Some(syn::parse_quote!(i8)), "i16" => Some(syn::parse_quote!(i16)), "i32" | "" => Some(syn::parse_quote!(i32)), "i64" => Some(syn::parse_quote!(i64)), "i128" => Some(syn::parse_quote!(i128)),
                "u8" => Some(syn::parse_quote!(u8)), "u16" => Some(syn::parse_quote!(u16)), "u32" => Some(syn::parse_quote!(u32)), "u64" => Some(syn::parse_quote!(u64)), "u128" => Some(syn::parse_quote!(u128)), "usize" => Some(syn::parse_quote!(usize)), _ => None,
            },
            syn::Lit::Float(v) => match v.suffix() { "f32" => Some(syn::parse_quote!(f32)), "f64" | "" => Some(syn::parse_quote!(f64)), _ => None },
            _ => None,
        },
        SynExpr::Paren(p) => expression_type(&p.expr, arguments, locals),
        SynExpr::Cast(c) => Some((*c.ty).clone()),
        SynExpr::Binary(b) => {
            let left = expression_type(&b.left, arguments, locals);
            let right = expression_type(&b.right, arguments, locals);
            left.or(right)
        }
        SynExpr::Unary(u) => expression_type(&u.expr, arguments, locals),
        SynExpr::Field(field) => {
            let base_type = expression_type(&field.base, arguments, locals)?;
            // Field types are deliberately resolved from the user's type rather than
            // guessed from the field expression. Rust will type-check the final
            // generated closure, while this supplies the expected type to lowering.
            let _ = base_type;
            None
        }
        SynExpr::Call(c) => c.args.first().and_then(|a| expression_type(a, arguments, locals)),
        SynExpr::MethodCall(m) if m.method == "len" && m.args.is_empty() => Some(syn::parse_quote!(usize)),
        SynExpr::Index(index) => {
            let sequence_type = expression_type(&index.expr, arguments, locals)?;
            match sequence_type {
                Type::Array(array) => Some((*array.elem).clone()),
                Type::Reference(reference) => match &*reference.elem { Type::Slice(slice) => Some((*slice.elem).clone()), _ => None },
                _ => None,
            }
        }
        SynExpr::Struct(s) => Some(Type::Path(syn::TypePath { qself: None, path: s.path.clone() })),
        _ => None,
    }
}
