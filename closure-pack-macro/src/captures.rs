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
    bound: BTreeSet<String>,
    names: BTreeSet<String>,
}

impl CaptureVisitor {
    fn new(arguments: &[ClosureArgument]) -> Self {
        let mut bound = BTreeSet::new();
        for argument in arguments {
            bound.insert(argument.name.to_string());
        }
        Self {
            bound,
            names: BTreeSet::new(),
        }
    }
}

impl<'ast> Visit<'ast> for CaptureVisitor {
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.path.segments.len() == 1 {
            let name = node.path.segments[0].ident.to_string();
            if !self.bound.contains(&name) && name != "self" {
                self.names.insert(name);
            }
        }
        visit::visit_expr_path(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        for arg in &node.args {
            self.visit_expr(arg);
        }
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(init) = &node.init {
            self.visit_expr(&init.expr);
        }
        if let Some(ident) = pat_ident(&node.pat) {
            self.bound.insert(ident.to_string());
        }
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        self.visit_expr(&node.expr);
        if let Some(ident) = pat_ident(&node.pat) {
            self.bound.insert(ident.to_string());
        }
        self.visit_block(&node.body);
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
                    Pat::Type(p) => Some((*p.ty).clone()),
                    _ => local.init.as_ref().and_then(|i| {
                        infer_in_expr(&i.expr, name, arguments, &current, None)
                            .or_else(|| expression_type(&i.expr, arguments, &current))
                    }),
                };

                if let Some(ident) = pat_ident(&local.pat) {
                    current.push(LocalVariable {
                        name: ident.clone(),
                        index: current.len(),
                        type_info: ty.clone(),
                        mutable: matches!(&local.pat, Pat::Ident(p) if p.mutability.is_some()),
                    });
                }

                if let Some(init) = &local.init {
                    // The type of a local initializer comes from its annotation or
                    // from the initializer itself. Do not use the closure return
                    // type here: doing so can incorrectly infer an unrelated
                    // capture (for example `PI`) as the closure's return type.
                    if let Some(found) =
                        infer_in_expr(&init.expr, name, arguments, &current, ty.as_ref())
                    {
                        return Some(found);
                    }
                }
            }
            syn::Stmt::Expr(expr, _) => {
                if let Some(ty) = infer_in_expr(expr, name, arguments, &current, expected) {
                    return Some(ty);
                }
            }
            _ => {}
        }
    }

    // `expected` only describes the type of the block's result. If the capture
    // was not found anywhere in the block, returning it here would assign the
    // return type to an unrelated capture. The capture must be observed in an
    // expression before an expected type can be used to infer it.
    None
}

fn infer_binary(
    binary: &syn::ExprBinary,
    name: &Ident,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected: Option<&Type>,
) -> Option<Type> {
    let left_type = expression_type(&binary.left, arguments, locals);
    let right_type = expression_type(&binary.right, arguments, locals);

    if let Some(t) = left_type.as_ref() {
        if let Some(found) = infer_in_expr(&binary.right, name, arguments, locals, Some(t)) {
            return Some(found);
        }
    }
    if let Some(t) = right_type.as_ref() {
        if let Some(found) = infer_in_expr(&binary.left, name, arguments, locals, Some(t)) {
            return Some(found);
        }
    }

    infer_in_expr(&binary.left, name, arguments, locals, None)
        .or_else(|| infer_in_expr(&binary.right, name, arguments, locals, None))
        .or_else(|| {
            expected.and_then(|t| {
                infer_in_expr(&binary.left, name, arguments, locals, Some(t))
                    .or_else(|| infer_in_expr(&binary.right, name, arguments, locals, Some(t)))
            })
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
            let path_name = &path.path.segments[0].ident;
            if path_name != name {
                return None;
            }

            // A capture can be used in an expression without carrying an explicit
            // type annotation. Prefer the type imposed by the surrounding
            // expression, but also use any known argument/local type when the
            // capture name resolves to one of those bindings. This makes capture
            // inference independent of the compiler's general expression_type()
            // helper, which intentionally does not assign a type to captures.
            expected
                .cloned()
                .or_else(|| {
                    arguments
                        .iter()
                        .find(|argument| argument.name == *path_name)
                        .map(|argument| argument.type_info.clone())
                })
                .or_else(|| {
                    locals
                        .iter()
                        .rev()
                        .find(|local| local.name == *path_name)
                        .and_then(|local| local.type_info.clone())
                })
        }
        Expr::Paren(paren) => infer_in_expr(&paren.expr, name, arguments, locals, expected),
        Expr::Binary(binary) => infer_binary(binary, name, arguments, locals, expected),
        Expr::Unary(unary) => infer_in_expr(&unary.expr, name, arguments, locals, expected),
        Expr::Cast(cast) => infer_in_expr(&cast.expr, name, arguments, locals, Some(&cast.ty)),
        Expr::Index(index) => {
            infer_in_expr(&index.expr, name, arguments, locals, None).or_else(|| {
                infer_in_expr(
                    &index.index,
                    name,
                    arguments,
                    locals,
                    Some(&syn::parse_quote!(usize)),
                )
            })
        }
        Expr::Call(call) => call
            .args
            .iter()
            .find_map(|arg| infer_in_expr(arg, name, arguments, locals, expected)),
        Expr::Block(block) => infer_expr_block(&block.block, name, arguments, locals, expected),
        Expr::If(if_expr) => infer_in_expr(
            &if_expr.cond,
            name,
            arguments,
            locals,
            Some(&syn::parse_quote!(bool)),
        )
        .or_else(|| infer_expr_block(&if_expr.then_branch, name, arguments, locals, expected))
        .or_else(|| {
            if_expr
                .else_branch
                .as_ref()
                .and_then(|(_, expr)| infer_in_expr(expr, name, arguments, locals, expected))
        }),
        Expr::While(while_expr) => infer_in_expr(
            &while_expr.cond,
            name,
            arguments,
            locals,
            Some(&syn::parse_quote!(bool)),
        )
        .or_else(|| infer_expr_block(&while_expr.body, name, arguments, locals, expected)),
        Expr::ForLoop(for_expr) => infer_in_expr(&for_expr.expr, name, arguments, locals, None)
            .or_else(|| {
                let mut nested = locals.to_vec();
                if let Some(ident) = pat_ident(&for_expr.pat) {
                    nested.push(LocalVariable {
                        name: ident.clone(),
                        index: nested.len(),
                        type_info: None,
                        mutable: false,
                    });
                }
                infer_expr_block(&for_expr.body, name, arguments, &nested, expected)
            }),
        Expr::Loop(loop_expr) => {
            infer_expr_block(&loop_expr.body, name, arguments, locals, expected)
        }
        _ => None,
    }
}

pub(crate) fn discover(body: &syn::Block, arguments: &[ClosureArgument]) -> Vec<Capture> {
    let mut locals = LocalCollector {
        names: BTreeSet::new(),
    };
    locals.visit_block(body);

    let mut visitor = CaptureVisitor::new(arguments);
    visitor.bound.extend(locals.names);
    visitor.visit_block(body);

    visitor
        .names
        .into_iter()
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
        capture.type_info =
            infer_expr_block(body, &capture.name, arguments, &locals, Some(return_type));
    }
}

pub(crate) fn is_simple_capture_path(expr: &Expr) -> bool {
    matches!(expr, Expr::Path(path) if path.path.segments.len() == 1)
}
