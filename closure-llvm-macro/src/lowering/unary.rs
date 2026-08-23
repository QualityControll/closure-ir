use proc_macro2::TokenStream;

use quote::quote;

use syn::{
    ExprUnary,
    Type,
};

use crate::parser::ClosureArgument;

use super::expression::{
    lower_expr,
    LocalVariable,
};


// ============================================================
// Unary operations
// ============================================================

pub(crate) fn lower_unary(
    unary: &ExprUnary,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    let operand =
        lower_expr(
            &unary.expr,
            arguments,
            locals,
            expected_type,
        )?;


    let expression =
        match unary.op {
            syn::UnOp::Not(_) =>
                quote! {
                    ::closure_llvm::Expr::Not
                },

            syn::UnOp::Neg(_) =>
                quote! {
                    ::closure_llvm::Expr::Neg
                },

            _ =>
                return Err(
                    syn::Error::new_spanned(
                        unary,
                        "unsupported unary operator",
                    )
                ),
        };


    Ok(
        quote! {
            #expression {
                operand:
                    Box::new(#operand),
            }
        }
    )
}