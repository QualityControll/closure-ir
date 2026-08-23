use proc_macro2::TokenStream;

use quote::quote;

use syn::ExprTuple;

use crate::parser::ClosureArgument;

use super::expression::lower_expr;


// ============================================================
// Tuple
// ============================================================

pub(crate) fn lower_tuple(
    tuple: &ExprTuple,
    arguments: &[ClosureArgument],
) -> syn::Result<TokenStream> {
    let elements =
        tuple
            .elems
            .iter()
            .map(|element| {
                lower_expr(
                    element,
                    arguments,
                    None,
                )
            })
            .collect::<syn::Result<Vec<_>>>()?;


    Ok(
        quote! {
            ::closure_llvm::Expr::Tuple {
                elements: vec![
                    #(
                        #elements
                    ),*
                ],
            }
        }
    )
}