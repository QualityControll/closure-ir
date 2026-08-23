use proc_macro2::TokenStream;

use quote::quote;

use syn::ExprTuple;

use crate::parser::ClosureArgument;

use super::expression::{
    lower_expr,
    LocalVariable,
};


// ============================================================
// Tuple
// ============================================================

pub(crate) fn lower_tuple(
    tuple: &ExprTuple,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
) -> syn::Result<TokenStream> {
    let elements =
        tuple
            .elems
            .iter()
            .map(|element| {
                lower_expr(
                    element,
                    arguments,
                    locals,
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