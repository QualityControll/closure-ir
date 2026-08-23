use proc_macro2::TokenStream;

use quote::quote;

use syn::{
    Expr as SynExpr,
    ExprIf,
    Type,
};

use crate::parser::ClosureArgument;

use super::{
    block::lower_block,
    expression::{
        lower_expr,
        LocalVariable,
    },
};


// ============================================================
// If / else
// ============================================================

pub(crate) fn lower_if(
    if_expr: &ExprIf,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    let condition =
        lower_expr(
            &if_expr.cond,
            arguments,
            locals,
            Some(
                &Type::Verbatim(
                    quote! {
                        bool
                    }
                )
            ),
        )?;


    let then_branch =
        lower_block(
            &if_expr.then_branch,
            arguments,
            locals,
            expected_type,
        )?;


    let else_branch =
        if_expr
            .else_branch
            .as_ref()
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    if_expr,
                    "if expressions require an else branch",
                )
            })?;


    let else_branch =
        match &*else_branch.1 {
            SynExpr::Block(block) =>
                lower_block(
                    &block.block,
                    arguments,
                    locals,
                    expected_type,
                )?,

            SynExpr::If(nested) =>
                lower_if(
                    nested,
                    arguments,
                    locals,
                    expected_type,
                )?,

            other =>
                return Err(
                    syn::Error::new_spanned(
                        other,
                        "else must contain a block or if expression",
                    )
                ),
        };


    Ok(
        quote! {
            ::closure_llvm::Expr::IfElse {
                condition:
                    Box::new(#condition),

                then_branch:
                    Box::new(#then_branch),

                else_branch:
                    Box::new(#else_branch),
            }
        }
    )
}