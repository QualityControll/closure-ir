use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, Ident, Pat, Type};
use crate::parser::ClosureArgument;

#[derive(Clone)]
pub(crate) struct Capture { pub(crate) name: Ident, pub(crate) type_info: Option<Type> }

struct CaptureVisitor { bound:BTreeSet<String>, names:BTreeSet<String> }
impl CaptureVisitor { fn new(arguments:&[ClosureArgument])->Self{let mut bound=BTreeSet::new();for a in arguments{bound.insert(a.name.to_string());}Self{bound,names:BTreeSet::new()}} }
impl<'ast> Visit<'ast> for CaptureVisitor {
 fn visit_expr_path(&mut self,node:&'ast syn::ExprPath){if node.path.segments.len()==1{let name=node.path.segments[0].ident.to_string();if !self.bound.contains(&name)&&name!="self"{self.names.insert(name);}}visit::visit_expr_path(self,node)}
 fn visit_expr_call(&mut self,node:&'ast ExprCall){for arg in &node.args{self.visit_expr(arg);}}
 fn visit_local(&mut self,node:&'ast syn::Local){if let Some(init)=&node.init{self.visit_expr(&init.expr);}if let Pat::Ident(p)=&node.pat{self.bound.insert(p.ident.to_string());}}
 fn visit_expr_for_loop(&mut self,node:&'ast syn::ExprForLoop){self.visit_expr(&node.expr);if let Pat::Ident(p)=&*node.pat{self.bound.insert(p.ident.to_string());}self.visit_block(&node.body);}
}
pub(crate) fn discover(body:&syn::Block,arguments:&[ClosureArgument])->Vec<Capture>{let mut v=CaptureVisitor::new(arguments);v.visit_block(body);v.names.into_iter().map(|name|Capture{name:Ident::new(&name,proc_macro2::Span::call_site()),type_info:None}).collect()}
pub(crate) fn is_simple_capture_path(expr:&Expr)->bool{matches!(expr,Expr::Path(path) if path.path.segments.len()==1)}
