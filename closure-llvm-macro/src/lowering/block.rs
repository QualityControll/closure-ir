use proc_macro2::TokenStream;

use syn::{
    Pat,
    Stmt,
    Type,
};

use crate::lowering::expression::{
    expression_type,
    lower_expr,
    LocalVariable,
};
use crate::parser::ClosureArgument;


// ============================================================
// Block
// ============================================================

pub(crate) fn lower_block(
    block: &syn::Block,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    lower_stmts(
        &block.stmts,
        arguments,
        locals,
        expected_type,
    )
}


fn lower_stmts(
    statements: &[Stmt],
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    let statement =
        statements
            .first()
            .ok_or_else(|| {
                syn::Error::new(
                    proc_macro2::Span::call_site(),
                    "compiled closure blocks cannot be empty",
                )
            })?;

    match statement {
        Stmt::Local(local) => {
            let name =
                match &local.pat {
                    Pat::Ident(pattern) =>
                        pattern.ident.clone(),

                    Pat::Type(pattern) =>
                        match &*pattern.pat {
                            Pat::Ident(pattern) =>
                                pattern.ident.clone(),

                            _ =>
                                return Err(
                                    syn::Error::new_spanned(
                                        &pattern.pat,
                                        "let bindings must use identifiers",
                                    )
                                ),
                        },

                    _ =>
                        return Err(
                            syn::Error::new_spanned(
                                &local.pat,
                                "let bindings must use identifiers",
                            )
                        ),
                };

            let initializer =
                local
                    .init
                    .as_ref()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(
                            local,
                            "let bindings require an initializer",
                        )
                    })?;

            let local_type =
                match &local.pat {
                    Pat::Type(pattern) =>
                        Some((*pattern.ty).clone()),

                    _ =>
                        expression_type(
                            &initializer.expr,
                            arguments,
                            locals,
                        ),
                };

            let value =
                lower_expr(
                    &initializer.expr,
                    arguments,
                    locals,
                    local_type.as_ref(),
                )?;

            let mut next_locals =
                locals.to_vec();

            next_locals.push(
                LocalVariable {
                    name,
                    value,
                    type_info: local_type,
                }
            );

            lower_stmts(
                &statements[1..],
                arguments,
                &next_locals,
                expected_type,
            )
        }

        Stmt::Expr(expr, _) => {
            if statements.len() != 1 {
                return Err(
                    syn::Error::new_spanned(
                        &statements[1],
                        "only let bindings may precede the final expression",
                    )
                );
            }

            lower_expr(
                expr,
                arguments,
                locals,
                expected_type,
            )
        }

        other =>
            Err(
                syn::Error::new_spanned(
                    other,
                    "only let bindings and a final expression are supported",
                )
            ),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    fn lower(source: TokenStream) -> syn::Result<TokenStream> {
        let block = syn::parse2::<syn::Block>(source)?;

        lower_block(
            &block,
            &[],
            &[],
            None,
        )
    }

    #[test]
    fn lowers_let_binding() {
        let result =
            lower(
                quote! {
                    {
                        let x = 10;
                        x
                    }
                }
            )
            .unwrap();

        assert_eq!(
            result.to_string(),
            quote! {
                ::closure_llvm::Expr::Constant(
                    ::closure_llvm::Value::I32(10)
                )
            }
            .to_string()
        );
    }

    #[test]
    fn lowers_multiple_let_bindings() {
        let result =
            lower(
                quote! {
                    {
                        let x = 10;
                        let y = x + 2;
                        y
                    }
                }
            )
            .unwrap();

        assert_eq!(
            result.to_string(),
            quote! {
                ::closure_llvm::Expr::Add {
                    lhs: Box::new(
                        ::closure_llvm::Expr::Constant(
                            ::closure_llvm::Value::I32(10)
                        )
                    ),
                    rhs: Box::new(
                        ::closure_llvm::Expr::Constant(
                            ::closure_llvm::Value::I32(2)
                        )
                    ),
                }
            }
            .to_string()
        );
    }

    #[test]
    fn rejects_let_without_initializer() {
        let error =
            lower(
                quote! {
                    {
                        let x;
                        x
                    }
                }
            )
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("let bindings require an initializer")
        );
    }
}
