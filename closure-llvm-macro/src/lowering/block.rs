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
                if let syn::Expr::ForLoop(for_expr) = expr {
                    let name = match &for_expr.pat {
                        Pat::Ident(pattern) => pattern.ident.clone(),
                        _ => return Err(syn::Error::new_spanned(&for_expr.pat, "for loop bindings must use identifiers")),
                    };
                    let range = match &*for_expr.expr {
                        syn::Expr::Range(range) => range,
                        _ => return Err(syn::Error::new_spanned(&for_expr.expr, "for loops currently require a range expression")),
                    };
                    let start = range.start.as_ref().ok_or_else(|| syn::Error::new_spanned(range, "for ranges require a start value"))?;
                    let end = range.end.as_ref().ok_or_else(|| syn::Error::new_spanned(range, "for ranges require an end value"))?;
                    let local_type = expression_type(start, arguments, &current_locals)
                        .or_else(|| expression_type(end, arguments, &current_locals))
                        .ok_or_else(|| syn::Error::new_spanned(range, "cannot infer for-loop range type"))?;
                    let start_expr = lower_expr(start, arguments, &current_locals, Some(&local_type))?;
                    let end_expr = lower_expr(end, arguments, &current_locals, Some(&local_type))?;
                    let local_index = current_locals.len();
                    let type_info = quote! { <#local_type as ::closure_llvm::CompileType>::type_info() };
                    let mut body_locals = current_locals.clone();
                    body_locals.push(LocalVariable { name, index: local_index, type_info: Some(local_type.clone()), mutable: false });
                    let body = lower_block(&for_expr.body, arguments, &body_locals, None)?;
                    let inclusive = range.limits == syn::RangeLimits::Closed(syn::token::DotDotEq::default());
                    statement_tokens.push(quote! {
                        ::closure_llvm::Statement::For {
                            local: #local_index,
                            type_info: #type_info,
                            start: #start_expr,
                            end: #end_expr,
                            inclusive: #inclusive,
                            body: #body,
                        }
                    });
                    current_locals.push(LocalVariable { name: syn::Ident::new("__for_unused", proc_macro2::Span::call_site()), index: local_index, type_info: Some(local_type), mutable: false });
                    continue;
                }
                if let syn::Expr::Assign(assign) = expr {
                    let (index, expected) = assignment_target(assign, &current_locals)?;
                    let value = lower_expr(&assign.right, arguments, &current_locals, expected)?;
                    statement_tokens.push(quote! { ::closure_llvm::Statement::Assign { local: #index, value: #value } });
                    continue;
                }
                if !is_last {
                    return Err(syn::Error::new_spanned(expr, "only let bindings, assignments, while loops, for loops, and a final expression are supported"));
                }
                result = Some(lower_expr(expr, arguments, &current_locals, expected_type)?);
            }
            other => return Err(syn::Error::new_spanned(other, "only let bindings, assignments, while loops, for loops, and a final expression are supported")),
        }
    }

    if result.is_none() && expected_type.is_some() {
        return Err(syn::Error::new(proc_macro2::Span::call_site(), "closure block must end with an expression"));
    }

    let result_tokens = match result {
        Some(result) => quote! { Some(#result) },
        None => quote! { None },
    };

    Ok(quote! {
        ::closure_llvm::Block { statements: vec![#(#statement_tokens),*], result: #result_tokens }
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
