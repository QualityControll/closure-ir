use proc_macro2::TokenStream;

use quote::quote;

use syn::ExprPath;

use crate::parser::ClosureArgument;


// ============================================================
// Argument
// ============================================================

pub(crate) fn lower_path(
    path: &ExprPath,
    arguments: &[ClosureArgument],
) -> syn::Result<TokenStream> {
    if path.path.segments.len() != 1 {
        return Err(
            syn::Error::new_spanned(
                path,
                "only simple identifiers are supported",
            )
        );
    }

    let name =
        &path.path.segments[0].ident;

    let index =
        arguments
            .iter()
            .position(|argument| &argument.name == name)
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    path,
                    format!(
                        "unknown closure argument `{}`",
                        name
                    ),
                )
            })?;

    Ok(
        quote! {
            ::closure_llvm::Expr::Argument(#index)
        }
    )
}