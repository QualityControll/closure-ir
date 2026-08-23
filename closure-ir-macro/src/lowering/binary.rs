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
                    ::closure_ir::Expr::Add
                },

            syn::BinOp::Sub(_) =>
                quote! {
                    ::closure_ir::Expr::Sub
                },

            syn::BinOp::Mul(_) =>
                quote! {
                    ::closure_ir::Expr::Mul
                },

            syn::BinOp::Div(_) =>
                quote! {
                    ::closure_ir::Expr::Div
                },

            syn::BinOp::Rem(_) =>
                quote! {
                    ::closure_ir::Expr::Rem
                },

            syn::BinOp::Eq(_) =>
                quote! {
                    ::closure_ir::Expr::Eq
                },

            syn::BinOp::Ne(_) =>
                quote! {
                    ::closure_ir::Expr::Ne
                },

            syn::BinOp::Lt(_) =>
                quote! {
                    ::closure_ir::Expr::Lt
                },

            syn::BinOp::Le(_) =>
                quote! {
                    ::closure_ir::Expr::Le
                },

            syn::BinOp::Gt(_) =>
                quote! {
                    ::closure_ir::Expr::Gt
                },

            syn::BinOp::Ge(_) =>
                quote! {
                    ::closure_ir::Expr::Ge
                },

            syn::BinOp::And(_) =>
                quote! {
                    ::closure_ir::Expr::And
                },

            syn::BinOp::Or(_) =>
                quote! {
                    ::closure_ir::Expr::Or
                },

            syn::BinOp::BitAnd(_) =>
                quote! {
                    ::closure_ir::Expr::BitAnd
                },

            syn::BinOp::BitOr(_) =>
                quote! {
                    ::closure_ir::Expr::BitOr
                },

            syn::BinOp::BitXor(_) =>
                quote! {
                    ::closure_ir::Expr::BitXor
                },

            syn::BinOp::Shl(_) =>
                quote! {
                    ::closure_ir::Expr::Shl
                },

            syn::BinOp::Shr(_) =>
                quote! {
                    ::closure_ir::Expr::Shr
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