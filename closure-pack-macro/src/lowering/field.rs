use proc_macro2::TokenStream;

use quote::quote;

use syn::ExprField;

use crate::parser::ClosureArgument;

use super::expression::{lower_expr, LocalVariable};

// ============================================================
// Field access
// ============================================================

pub(crate) fn lower_field(
    field: &ExprField,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
) -> syn::Result<TokenStream> {
    let object = lower_expr(&field.base, arguments, locals, None)?;

    let name = match &field.member {
        syn::Member::Named(name) => name.to_string(),

        syn::Member::Unnamed(index) => index.index.to_string(),
    };

    Ok(quote! {
        ::closure_pack::Expr::Field {
            object:
                Box::new(#object),

            name:
                #name.to_string(),
        }
    })
}
