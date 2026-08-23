use proc_macro2::TokenStream;

use quote::quote;

use syn::{
    ExprBinary,
    Type,
};

use crate::parser::ClosureArgument;

use super::expression::{
    expression_type,
    lower_expr,
    LocalVariable,
};


// ============================================================
// Binary operations
// ============================================================

pub(crate) fn lower_binary(
    binary: &ExprBinary,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    let operand_type =
        expression_type(
            &binary.left,
            arguments,
            locals,
        )
        .or_else(|| {
            expression_type(
                &binary.right,
                arguments,
                locals,
            )
        });

    let operand_type =
        operand_type
            .as_ref()
            .or(expected_type);


    let lhs =
        lower_expr(
            &binary.left,
            arguments,
            locals,
            operand_type,
        )?;

    let rhs =
        lower_expr(
            &binary.right,
            arguments,
            locals,
            operand_type,
        )?;


    let operation =
        match binary.op {
            syn::BinOp::Add(_) =>
                quote! {
                    ::closure_llvm::Expr::Add
                },

            syn::BinOp::Sub(_) =>
                quote! {
                    ::closure_llvm::Expr::Sub
                },

            syn::BinOp::Mul(_) =>
                quote! {
                    ::closure_llvm::Expr::Mul
                },

            syn::BinOp::Div(_) =>
                quote! {
                    ::closure_llvm::Expr::Div
                },

            syn::BinOp::Rem(_) =>
                quote! {
                    ::closure_llvm::Expr::Rem
                },

            syn::BinOp::Eq(_) =>
                quote! {
                    ::closure_llvm::Expr::Eq
                },

            syn::BinOp::Ne(_) =>
                quote! {
                    ::closure_llvm::Expr::Ne
                },

            syn::BinOp::Lt(_) =>
                quote! {
                    ::closure_llvm::Expr::Lt
                },

            syn::BinOp::Le(_) =>
                quote! {
                    ::closure_llvm::Expr::Le
                },

            syn::BinOp::Gt(_) =>
                quote! {
                    ::closure_llvm::Expr::Gt
                },

            syn::BinOp::Ge(_) =>
                quote! {
                    ::closure_llvm::Expr::Ge
                },

            syn::BinOp::And(_) =>
                quote! {
                    ::closure_llvm::Expr::And
                },

            syn::BinOp::Or(_) =>
                quote! {
                    ::closure_llvm::Expr::Or
                },

            syn::BinOp::BitAnd(_) =>
                quote! {
                    ::closure_llvm::Expr::BitAnd
                },

            syn::BinOp::BitOr(_) =>
                quote! {
                    ::closure_llvm::Expr::BitOr
                },

            syn::BinOp::BitXor(_) =>
                quote! {
                    ::closure_llvm::Expr::BitXor
                },

            syn::BinOp::Shl(_) =>
                quote! {
                    ::closure_llvm::Expr::Shl
                },

            syn::BinOp::Shr(_) =>
                quote! {
                    ::closure_llvm::Expr::Shr
                },

            _ =>
                return Err(
                    syn::Error::new_spanned(
                        binary,
                        "unsupported binary operator",
                    )
                ),
        };


    Ok(
        quote! {
            #operation {
                lhs:
                    Box::new(#lhs),

                rhs:
                    Box::new(#rhs),
            }
        }
    )
}