use syn::{Expr as SynExpr, ExprBlock, Pat, ReturnType, Type};

pub(crate) struct ClosureInput { pub(crate) arguments: Vec<ClosureArgument>, pub(crate) return_type: Type, pub(crate) body: ExprBlock }
#[derive(Clone)]
pub(crate) struct ClosureArgument { pub(crate) name: syn::Ident, pub(crate) type_info: Type, pub(crate) capture: bool }
impl syn::parse::Parse for ClosureInput {
 fn parse(input:syn::parse::ParseStream<'_>)->syn::Result<Self>{let closure: syn::ExprClosure=input.parse()?;let arguments=closure.inputs.iter().map(|argument|match argument{Pat::Type(pat_type)=>{let name=match &*pat_type.pat{Pat::Ident(pat_ident)=>pat_ident.ident.clone(),_=>return Err(syn::Error::new_spanned(&pat_type.pat,"closure arguments must be identifiers"))};Ok(ClosureArgument{name,type_info:(*pat_type.ty).clone(),capture:false})}_=>Err(syn::Error::new_spanned(argument,"closure arguments must have explicit types"))}).collect::<syn::Result<Vec<_>>>()?;let return_type=match closure.output{ReturnType::Type(_,ty)=>(*ty).clone(),ReturnType::Default=>syn::parse_quote!{()}};let body=match *closure.body{SynExpr::Block(block)=>block,other=>return Err(syn::Error::new_spanned(other,"closure body must be a block"))};Ok(Self{arguments,return_type,body})}
}
pub(crate) struct CallInput { pub(crate) closure: SynExpr, pub(crate) values: Vec<SynExpr> }
impl syn::parse::Parse for CallInput { fn parse(input:syn::parse::ParseStream<'_>)->syn::Result<Self>{let closure=input.parse()?;let mut values=Vec::new();while !input.is_empty(){input.parse::<syn::Token![,]>()?;if input.is_empty(){break;}values.push(input.parse()?);}Ok(Self{closure,values})} }
