use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprAssign, Pat, Stmt, Type};
use crate::lowering::expression::{expression_type, lower_expr, LocalVariable};
use crate::parser::ClosureArgument;

pub(crate) fn lower_block(block: &syn::Block, arguments: &[ClosureArgument], locals: &[LocalVariable], expected_type: Option<&Type>) -> syn::Result<TokenStream> {
    lower_statements(&block.stmts, arguments, locals, expected_type)
}

fn lower_statements(statements: &[Stmt], arguments: &[ClosureArgument], locals: &[LocalVariable], expected_type: Option<&Type>) -> syn::Result<TokenStream> {
    let mut statement_tokens = Vec::new();
    let mut current_locals = locals.to_vec();
    let mut result = None;

    for (position, statement) in statements.iter().enumerate() {
        let is_last = position + 1 == statements.len();
        match statement {
            Stmt::Local(local) => {
                let (name, mutable, explicit_type) = match &local.pat {
                    Pat::Ident(pattern) => (pattern.ident.clone(), pattern.mutability.is_some(), None),
                    Pat::Type(pattern) => {
                        let explicit_type = (*pattern.ty).clone();
                        match &*pattern.pat {
                            Pat::Ident(pattern) => (pattern.ident.clone(), pattern.mutability.is_some(), Some(explicit_type)),
                            _ => return Err(syn::Error::new_spanned(&pattern.pat, "let bindings must use identifiers")),
                        }
                    }
                    _ => return Err(syn::Error::new_spanned(&local.pat, "let bindings must use identifiers")),
                };
                let initializer = local.init.as_ref().ok_or_else(|| syn::Error::new_spanned(local, "let bindings require an initializer"))?;
                let local_type = explicit_type.or_else(|| expression_type(&initializer.expr, arguments, &current_locals)).ok_or_else(|| syn::Error::new_spanned(&initializer.expr, "cannot infer local variable type"))?;
                let value = lower_expr(&initializer.expr, arguments, &current_locals, Some(&local_type))?;
                let index = current_locals.len();
                let type_info = quote! { <#local_type as ::closure_llvm::CompileType>::type_info() };
                statement_tokens.push(quote! {
                    ::closure_llvm::Statement::Let { local: #index, type_info: #type_info, value: #value, mutable: #mutable }
                });
                current_locals.push(LocalVariable { name, index, type_info: Some(local_type), mutable });
            }
            Stmt::Expr(expr, _) => {
                if let syn::Expr::While(while_expr) = expr {
                    let condition = lower_expr(&while_expr.cond, arguments, &current_locals, Some(&syn::parse_quote!(bool)))?;
                    let body = lower_block(&while_expr.body, arguments, &current_locals, None)?;
                    statement_tokens.push(quote! {
                        ::closure_llvm::Statement::While { condition: #condition, body: #body }
                    });
                    continue;
                }
                if let syn::Expr::Assign(assign) = expr {
                    let (index, expected) = assignment_target(assign, &current_locals)?;
                    let value = lower_expr(&assign.right, arguments, &current_locals, expected)?;
                    statement_tokens.push(quote! { ::closure_llvm::Statement::Assign { local: #index, value: #value } });
                    continue;
                }
                if !is_last {
                    return Err(syn::Error::new_spanned(expr, "only let bindings, assignments, and while loops may precede the final expression"));
                }
                result = Some(lower_expr(expr, arguments, &current_locals, expected_type)?);
            }
            other => return Err(syn::Error::new_spanned(other, "only let bindings, assignments, while loops, and a final expression are supported")),
        }
    }

    if result.is_none() && expected_type.is_some() {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "closure block must end with an expression"));
    }

    Ok(quote! {
        ::closure_llvm::Block { statements: vec![#(#statement_tokens),*], result: #result }
    })
}

fn assignment_target<'a>(assign: &ExprAssign, locals: &'a [LocalVariable]) -> syn::Result<(usize, Option<&'a Type>)> {
    let name = match &*assign.left {
        syn::Expr::Path(path) if path.path.segments.len() == 1 => &path.path.segments[0].ident,
        _ => return Err(syn::Error::new_spanned(&assign.left, "assignment targets must be local variables")),
    };
    let local = locals.iter().rev().find(|local| &local.name == name).ok_or_else(|| syn::Error::new_spanned(&assign.left, format!("unknown local variable `{}`", name)))?;
    if !local.mutable {
        return Err(syn::Error::new_spanned(&assign.left, format!("cannot assign to immutable variable `{}`", name)));
    }
    Ok((local.index, local.type_info.as_ref()))
}
