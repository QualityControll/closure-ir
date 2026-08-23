use proc_macro2::TokenStream;

use syn::{
    Stmt,
    Type,
};

use crate::lowering::expression::lower_expr;
use crate::parser::ClosureArgument;


// ============================================================
// Block
// ============================================================

pub(crate) fn lower_block(
    block: &syn::Block,
    arguments: &[ClosureArgument],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    if block.stmts.len() != 1 {
        return Err(
            syn::Error::new_spanned(
                block,
                "compiled closure blocks must contain exactly one expression",
            )
        );
    }

    match &block.stmts[0] {
        Stmt::Expr(expr, _) =>
            lower_expr(
                expr,
                arguments,
                expected_type,
            ),

        other =>
            Err(
                syn::Error::new_spanned(
                    other,
                    "only expressions are supported",
                )
            ),
    }
}