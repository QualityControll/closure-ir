use std::collections::BTreeSet;
use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, Ident, Pat, Type};
use crate::parser::ClosureArgument;
use crate::lowering::expression::{expression_type, LocalVariable};

#[derive(Clone)]
pub(crate) struct Capture { pub(crate) name: Ident, pub(crate) type_info: Option<Type> }

fn pat_ident(pat: &Pat) -> Option<&Ident> {
    match pat {
        Pat::Ident(p) => Some(&p.ident),
        Pat::Type(p) => pat_ident(&p.pat),
        _ => None,
    }
}

struct CaptureVisitor { bound:BTreeSet<String>, names:BTreeSet<String> }
impl CaptureVisitor { fn new(arguments:&[ClosureArgument])->Self{let mut bound=BTreeSet::new();for a in arguments{bound.insert(a.name.to_string());}Self{bound,names:BTreeSet::new()}} }
impl<'ast> Visit<'ast> for CaptureVisitor {
 fn visit_expr_path(&mut self,node:&'ast syn::ExprPath){if node.path.segments.len()==1{let name=node.path.segments[0].ident.to_string();if !self.bound.contains(&name)&&name!="self"{self.names.insert(name);}}visit::visit_expr_path(self,node)}
 fn visit_expr_call(&mut self,node:&'ast ExprCall){for arg in &node.args{self.visit_expr(arg);}}
 fn visit_local(&mut self,node:&'ast syn::Local){if let Some(init)=&node.init{self.visit_expr(&init.expr);}if let Some(ident)=pat_ident(&node.pat){self.bound.insert(ident.to_string());}}
 fn visit_expr_for_loop(&mut self,node:&'ast syn::ExprForLoop){self.visit_expr(&node.expr);if let Some(ident)=pat_ident(&node.pat){self.bound.insert(ident.to_string());}self.visit_block(&node.body);}
}
struct LocalCollector{names:BTreeSet<String>}
impl<'ast> Visit<'ast> for LocalCollector{fn visit_local(&mut self,node:&'ast syn::Local){if let Some(ident)=pat_ident(&node.pat){self.names.insert(ident.to_string());}visit::visit_local(self,node)}fn visit_expr_for_loop(&mut self,node:&'ast syn::ExprForLoop){if let Some(ident)=pat_ident(&node.pat){self.names.insert(ident.to_string());}visit::visit_expr_for_loop(self,node)}}

fn infer_expr_block(block:&syn::Block,name:&Ident,arguments:&[ClosureArgument],locals:&[LocalVariable],expected:Option<&Type>)->Option<Type>{
 let mut current=locals.to_vec();
 for stmt in &block.stmts{match stmt{
  syn::Stmt::Local(local)=>{
   let ty=match &local.pat{
    Pat::Type(p)=>local.init.as_ref().and_then(|i|infer_in_expr(&i.expr,name,arguments,&current,Some(&*p.ty))).or_else(||Some((*p.ty).clone())),
    _=>local.init.as_ref().and_then(|i|infer_in_expr(&i.expr,name,arguments,&current,None)),
   };
   if let Some(ident)=pat_ident(&local.pat){current.push(LocalVariable{name:ident.clone(),index:current.len(),type_info:ty,mutable:matches!(&local.pat,Pat::Ident(p) if p.mutability.is_some())});}
  }
  syn::Stmt::Expr(e,_)=>if let Some(t)=infer_in_expr(e,name,arguments,&current,expected){return Some(t)},
  _=>{}
 }}
 expected.filter(|_| false).cloned()
}

fn infer_in_expr(expr:&Expr,name:&Ident,arguments:&[ClosureArgument],locals:&[LocalVariable],expected:Option<&Type>)->Option<Type>{match expr{
 Expr::Path(path) if path.path.segments.len()==1=>if path.path.segments[0].ident==*name{expected.cloned()}else{None},
 Expr::Paren(p)=>infer_in_expr(&p.expr,name,arguments,locals,expected),
 Expr::Binary(b)=>{let lt=expression_type(&b.left,arguments,locals);let rt=expression_type(&b.right,arguments,locals);if let Some(t)=lt.as_ref(){if let Some(found)=infer_in_expr(&b.right,name,arguments,locals,Some(t)){return Some(found);}}if let Some(t)=rt.as_ref(){if let Some(found)=infer_in_expr(&b.left,name,arguments,locals,Some(t)){return Some(found);}}infer_in_expr(&b.left,name,arguments,locals,expected).or_else(||infer_in_expr(&b.right,name,arguments,locals,expected))},
 Expr::Unary(u)=>infer_in_expr(&u.expr,name,arguments,locals,expected),
 Expr::Cast(c)=>infer_in_expr(&c.expr,name,arguments,locals,Some(&*c.ty)),
 Expr::Index(i)=>infer_in_expr(&i.expr,name,arguments,locals,None).or_else(||infer_in_expr(&i.index,name,arguments,locals,Some(&syn::parse_quote!(usize)))),
 Expr::Call(c)=>c.args.iter().find_map(|a|infer_in_expr(a,name,arguments,locals,expected)),
 Expr::If(i)=>infer_in_expr(&i.cond,name,arguments,locals,Some(&syn::parse_quote!(bool))).or_else(||infer_expr_block(&i.then_branch,name,arguments,locals,expected)).or_else(||i.else_branch.as_ref().and_then(|(_,e)|infer_in_expr(e,name,arguments,locals,expected))),
 _=>None,
}}

pub(crate) fn discover(body:&syn::Block,arguments:&[ClosureArgument])->Vec<Capture>{let mut locals=LocalCollector{names:BTreeSet::new()};locals.visit_block(body);let mut v=CaptureVisitor::new(arguments);v.bound.extend(locals.names);v.visit_block(body);v.names.into_iter().map(|name|Capture{name:Ident::new(&name,proc_macro2::Span::call_site()),type_info:None}).collect()}
pub(crate) fn infer_types(body:&syn::Block,arguments:&[ClosureArgument],captures:&mut [Capture],return_type:&Type){let locals=Vec::new();for capture in captures{capture.type_info=infer_expr_block(body,&capture.name,arguments,&locals,Some(return_type));}}
pub(crate) fn is_simple_capture_path(expr:&Expr)->bool{matches!(expr,Expr::Path(path) if path.path.segments.len()==1)}
