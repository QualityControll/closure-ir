use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, Ident, Pat, Type};

use crate::lowering::expression::{expression_type, LocalVariable};
use crate::parser::ClosureArgument;

#[derive(Clone)]
pub(crate) struct Capture {
    pub(crate) name: Ident,
    pub(crate) type_info: Option<Type>,
}

fn pat_ident(pat: &Pat) -> Option<&Ident> {
    match pat {
        Pat::Ident(p) => Some(&p.ident),
        Pat::Type(p) => pat_ident(&p.pat),
        _ => None,
    }
}

struct CaptureVisitor {
    names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for CaptureVisitor {
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.path.segments.len() == 1 {
            self.names
                .insert(node.path.segments[0].ident.to_string());
        }
        visit::visit_expr_path(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        for arg in &node.args {
            self.visit_expr(arg);
        }
    }
}

struct LocalCollector {
    names: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for LocalCollector {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(ident) = pat_ident(&node.pat) {
            self.names.insert(ident.to_string());
        }
        visit::visit_local(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        if let Some(ident) = pat_ident(&node.pat) {
            self.names.insert(ident.to_string());
        }
        visit::visit_expr_for_loop(self, node);
    }
}

fn infer_expr_block(
    block: &syn::Block,
    name: &Ident,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected: Option<&Type>,
) -> Option<Type> {
    let mut current = locals.to_vec();

    for stmt in &block.stmts {
        match stmt {
            syn::Stmt::Local(local) => {
                let ty = match &local.pat {
                    Pat::Type(p) => local
                        .init
                        .as_ref()
                        .and_then(|i| {
                            infer_in_expr(
                                &i.expr,
                                name,
                                arguments,
                                &current,
                                Some(&p.ty),
                            )
                        })
                        .or_else(|| Some((*p.ty).clone())),
                    _ => local.init.as_ref().and_then(|i| {
                        infer_in_expr(&i.expr, name, arguments, &current, None)
                    }),
                };

                if let Some(ident) = pat_ident(&local.pat) {
                    current.push(LocalVariable {
                        name: ident.clone(),
                        index: current.len(),
                        type_info: ty,
                        mutable: matches!(
                            &local.pat,
                            Pat::Ident(p) if p.mutability.is_some()
                        ),
                    });
                }
            }
            syn::Stmt::Expr(expr, _) => {
                if let Some(ty) = infer_in_expr(
                    expr,
                    name,
                    arguments,
                    &current,
                    expected,
                ) {
                    return Some(ty);
                }
            }
            _ => {}
        }
    }

    None
}

fn infer_binary(
    binary: &syn::ExprBinary,
    name: &Ident,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected: Option<&Type>,
) -> Option<Type> {
    // For arithmetic/bitwise operations, the result type is also the
    // operand type.  Propagate the expected result type down both sides.
    // This is important for expressions such as `-2.0 * pi / (n as f64)`,
    // where the capture itself has no independently known type.
    let is_comparison = matches!(
        binary.op,
        syn::BinOp::Eq(_)
            | syn::BinOp::Ne(_)
            | syn::BinOp::Lt(_)
            | syn::BinOp::Le(_)
            | syn::BinOp::Gt(_)
            | syn::BinOp::Ge(_)
    );

    if !is_comparison {
        if let Some(t) = expected {
            if let Some(found) = infer_in_expr(
                &binary.left,
                name,
                arguments,
                locals,
                Some(t),
            ) {
                return Some(found);
            }
            if let Some(found) = infer_in_expr(
                &binary.right,
                name,
                arguments,
                locals,
                Some(t),
            ) {
                return Some(found);
            }
        }
    }

    let left_type = expression_type(&binary.left, arguments, locals);
    let right_type = expression_type(&binary.right, arguments, locals);

    if let Some(t) = left_type.as_ref() {
        if let Some(found) = infer_in_expr(
            &binary.right,
            name,
            arguments,
            locals,
            Some(t),
        ) {
            return Some(found);
        }
    }

    if let Some(t) = right_type.as_ref() {
        if let Some(found) = infer_in_expr(
            &binary.left,
            name,
            arguments,
            locals,
            Some(t),
        ) {
            return Some(found);
        }
    }

    infer_in_expr(
        &binary.left,
        name,
        arguments,
        locals,
        expected,
    )
    .or_else(|| {
        infer_in_expr(
            &binary.right,
            name,
            arguments,
            locals,
            expected,
        )
    })
}

fn infer_in_expr(
    expr: &Expr,
    name: &Ident,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected: Option<&Type>,
) -> Option<Type> {
    match expr {
        Expr::Path(path) if path.path.segments.len() == 1 => {
            if path.path.segments[0].ident == *name {
                expected.cloned()
            } else {
                None
            }
        }
        Expr::Paren(paren) => infer_in_expr(
            &paren.expr,
            name,
            arguments,
            locals,
            expected,
        ),
        Expr::Binary(binary) => infer_binary(
            binary,
            name,
            arguments,
            locals,
            expected,
        ),
        Expr::Unary(unary) => infer_in_expr(
            &unary.expr,
            name,
            arguments,
            locals,
            expected,
        ),
        Expr::Cast(cast) => infer_in_expr(
            &cast.expr,
            name,
            arguments,
            locals,
            Some(&cast.ty),
        ),
        Expr::Index(index) => infer_in_expr(
            &index.expr,
            name,
            arguments,
            locals,
            None,
        )
        .or_else(|| {
            infer_in_expr(
                &index.index,
                name,
                arguments,
                locals,
                Some(&syn::parse_quote!(usize)),
            )
        }),
        Expr::Call(call) => call.args.iter().find_map(|arg| {
            infer_in_expr(
                arg,
                name,
                arguments,
                locals,
                expected,
            )
        }),
        Expr::If(if_expr) => infer_in_expr(
            &if_expr.cond,
            name,
            arguments,
            locals,
            Some(&syn::parse_quote!(bool)),
        )
        .or_else(|| {
            infer_expr_block(
                &if_expr.then_branch,
                name,
                arguments,
                locals,
                expected,
            )
        })
        .or_else(|| {
            if_expr
                .else_branch
                .as_ref()
                .and_then(|(_, expr)| {
                    infer_in_expr(
                        expr,
                        name,
                        arguments,
                        locals,
                        expected,
                    )
                })
        }),
        _ => None,
    }
}

pub(crate) fn discover(
    body: &syn::Block,
    arguments: &[ClosureArgument],
) -> Vec<Capture> {
    let mut refs = CaptureVisitor {
        names: BTreeSet::new(),
    };
    refs.visit_block(body);

    let mut locals = LocalCollector {
        names: BTreeSet::new(),
    };
    locals.visit_block(body);

    let bound: BTreeSet<String> = arguments
        .iter()
        .map(|argument| argument.name.to_string())
        .chain(locals.names)
        .collect();

    refs.names
        .into_iter()
        .filter(|name| name.as_str() != "self" && !bound.contains(name))
        .map(|name| Capture {
            name: Ident::new(&name, proc_macro2::Span::call_site()),
            type_info: None,
        })
        .collect()
}

pub(crate) fn infer_types(
    body: &syn::Block,
    arguments: &[ClosureArgument],
    captures: &mut [Capture],
    return_type: &Type,
) {
    let locals = Vec::new();
    for capture in captures {
        capture.type_info = infer_expr_block(
            body,
            &capture.name,
            arguments,
            &locals,
            Some(return_type),
        );
    }
}

pub(crate) fn is_simple_capture_path(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Path(path) if path.path.segments.len() == 1
    )
}
