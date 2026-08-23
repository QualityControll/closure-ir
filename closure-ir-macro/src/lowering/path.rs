use proc_macro2::TokenStream;
use quote::quote;
use syn::ExprPath;
use crate::parser::ClosureArgument;
use super::expression::LocalVariable;

pub(crate) fn lower_path(
    path: &ExprPath,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    _expected_type: Option<&syn::Type>,
) -> syn::Result<TokenStream> {
    if path.path.segments.len() != 1 {
        return Err(syn::Error::new_spanned(path, "only simple identifiers are supported"));
    }
    let name = &path.path.segments[0].ident;
    if let Some(index) = arguments.iter().position(|argument| &argument.name == name) {
        return Ok(quote! { ::closure_ir::Expr::Argument(#index) });
    }
    if let Some(local) = locals.iter().rev().find(|local| &local.name == name) {
        let index = arguments.len() + local.index;
        return Ok(quote! { ::closure_ir::Expr::Argument(#index) });
    }
    Err(syn::Error::new_spanned(path, format!("unknown closure argument or local variable `{}`", name)))
}
