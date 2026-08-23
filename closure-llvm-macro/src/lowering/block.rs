use proc_macro2::TokenStream;

use syn::{
    ExprAssign,
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
            let (name, mutable, explicit_type) =
                match &local.pat {
                    Pat::Ident(pattern) =>
                        (
                            pattern.ident.clone(),
                            pattern.mutability.is_some(),
                            None,
                        ),

                    Pat::Type(pattern) =>
                        match &*pattern.pat {
                            Pat::Ident(pattern) =>
                                (
                                    pattern.ident.clone(),
                                    pattern.mutability.is_some(),
                                    Some((*pattern.ty).clone()),
                                ),

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
                explicit_type
                    .or_else(|| {
                        expression_type(
                            &initializer.expr,
                            arguments,
                            locals,
                        )
                    });

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
                    mutable,
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
            if statements.len() == 1 {
                return lower_expr(
                    expr,
                    arguments,
                    locals,
                    expected_type,
                );
            }

            if let syn::Expr::Assign(assign) = expr {
                let next_locals =
                    lower_assignment(
                        assign,
                        arguments,
                        locals,
                    )?;

                return lower_stmts(
                    &statements[1..],
                    arguments,
                    &next_locals,
                    expected_type,
                );
            }

            Err(
                syn::Error::new_spanned(
                    &statements[1],
                    "only let bindings and assignments may precede the final expression",
                )
            )
        }

        other =>
            Err(
                syn::Error::new_spanned(
                    other,
                    "only let bindings, assignments, and a final expression are supported",
                )
            ),
    }
}


fn lower_assignment(
    assign: &ExprAssign,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
) -> syn::Result<Vec<LocalVariable>> {
    let name =
        match &*assign.left {
            syn::Expr::Path(path)
                if path.path.segments.len() == 1 =>
            {
                path.path.segments[0].ident.clone()
            }

            _ =>
                return Err(
                    syn::Error::new_spanned(
                        &assign.left,
                        "assignment targets must be local variables",
                    )
                ),
        };

    let index =
        locals
            .iter()
            .rposition(|local| &local.name == &name)
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    &assign.left,
                    format!(
                        "unknown local variable `{}`",
                        name
                    ),
                )
            })?;

    if !locals[index].mutable {
        return Err(
            syn::Error::new_spanned(
                &assign.left,
                format!(
                    "cannot assign to immutable variable `{}`",
                    name
                ),
            )
        );
    }

    let expected_type =
        locals[index].type_info.as_ref();

    let value =
        lower_expr(
            &assign.right,
            arguments,
            locals,
            expected_type,
        )?;

    let mut next_locals =
        locals.to_vec();

    next_locals[index].value = value;

    Ok(next_locals)
}
