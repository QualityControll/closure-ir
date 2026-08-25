use proc_macro2::TokenStream;
use quote::quote;
use syn::ExprPath;
use crate::parser::ClosureArgument;
use super::expression::LocalVariable;

pub(crate) fn lower_path(path:&ExprPath,arguments:&[ClosureArgument],locals:&[LocalVariable],_expected_type:Option<&syn::Type>)->syn::Result<TokenStream>{
 if path.path.segments.len()!=1{return Err(syn::Error::new_spanned(path,"only simple identifiers are supported"));}
 let name=&path.path.segments[0].ident;
 if let Some(argument)=arguments.iter().find(|argument|&argument.name==name){
  if argument.capture { let index=arguments.iter().filter(|a|a.capture).position(|a|&a.name==name).unwrap(); return Ok(quote!{::closure_pack::Expr::Capture(#index)}); }
  let index=arguments.iter().filter(|a|!a.capture).position(|a|&a.name==name).unwrap();
  return Ok(quote!{::closure_pack::Expr::Argument(#index)});
 }
 if let Some(local)=locals.iter().rev().find(|local|&local.name==name){let index=arguments.iter().filter(|a|!a.capture).count()+local.index;return Ok(quote!{::closure_pack::Expr::Argument(#index)});}
 Err(syn::Error::new_spanned(path,format!("unknown closure argument, capture, or local variable `{}`",name)))
}
