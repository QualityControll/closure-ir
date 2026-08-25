use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use crate::{captures, lowering::lower_block};
use crate::parser::{ClosureArgument, ClosureInput};

pub(crate) fn expand(input:TokenStream)->TokenStream{let input=parse_macro_input!(input as ClosureInput);match expand_compile_closure(input){Ok(tokens)=>tokens.into(),Err(error)=>error.into_compile_error().into()}}

fn expand_compile_closure(input:ClosureInput)->syn::Result<proc_macro2::TokenStream>{
 let ClosureInput{arguments,return_type,body}=input;
 let captures=captures::discover(&body.block,&arguments);
 let mut lowering_arguments=arguments.clone();
 for capture in &captures {
  let type_info=capture.type_info.clone().ok_or_else(||syn::Error::new(capture.name.span(),format!("cannot infer type of capture `{}`",capture.name)))?;
  lowering_arguments.push(ClosureArgument{name:capture.name.clone(),type_info,capture:true});
 }
 let locals=Vec::new();
 let is_unit=matches!(&return_type,syn::Type::Tuple(tuple) if tuple.elems.is_empty());
 let block=if is_unit{lower_block(&body.block,&lowering_arguments,&locals,None)?}else{lower_block(&body.block,&lowering_arguments,&locals,Some(&return_type))?};
 let argument_type_infos=arguments.iter().map(|argument|{let ty=&argument.type_info;quote!{<#ty as ::closure_pack::CompileType>::type_info()}}).collect::<Vec<_>>();
 let argument_types=arguments.iter().map(|argument|&argument.type_info).collect::<Vec<_>>();
 let tuple_type=if argument_types.is_empty(){quote!{()}}else{quote!{(#(#argument_types,)*)}};
 let capture_names=captures.iter().map(|capture|&capture.name).collect::<Vec<_>>();
 let capture_values=if capture_names.is_empty(){quote!{()}}else{quote!{(#(#capture_names,)*)}};
 let capture_type_infos=captures.iter().map(|capture|{
  let ty=capture.type_info.as_ref().ok_or_else(||syn::Error::new(capture.name.span(),format!("cannot infer type of capture `{}`",capture.name)))?;
  Ok(quote!{<#ty as ::closure_pack::CompileType>::type_info()})
 }).collect::<syn::Result<Vec<_>>>()?;
 Ok(quote!{{
   let __captures= #capture_values;
   let __closure=::closure_pack::Closure{captures:vec![#(#capture_type_infos),*],arguments:vec![#(#argument_type_infos),*],return_type:<#return_type as ::closure_pack::CompileType>::type_info(),body:#block};
   let __context:&'static ::closure_pack::melior::Context=Box::leak(Box::new(::closure_pack::melior::Context::new()));
   let __compiler=::closure_pack::Compiler::new(__context);
   __compiler.compile_captured::<#tuple_type,#return_type,_>(&__closure,__captures).expect("failed to compile closure")
 }})
}
