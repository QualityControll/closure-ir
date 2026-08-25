use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use crate::captures;
use crate::lowering::lower_block;
use crate::parser::ClosureInput;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ClosureInput);
    match expand_closure_pack(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_closure_pack(input: ClosureInput) -> syn::Result<proc_macro2::TokenStream> {
    let ClosureInput { arguments, return_type, body } = input;
    let mut discovered = captures::discover(&body.block, &arguments);
    captures::infer_types(&body.block, &arguments, &mut discovered, &return_type);
    let mut lowering_arguments = arguments.clone();
    for capture in &discovered {
        let type_info = capture.type_info.clone().ok_or_else(|| syn::Error::new(capture.name.span(), format!("cannot infer type of capture `{}`", capture.name)))?;
        lowering_arguments.push(crate::parser::ClosureArgument { name: capture.name.clone(), type_info, capture: true });
    }
    let locals = Vec::new();
    let block = lower_block(&body.block, &lowering_arguments, &locals, Some(&return_type))?;
    let argument_type_infos = arguments.iter().map(|argument| { let ty=&argument.type_info; quote! { <#ty as ::closure_pack::CompileType>::type_info() } }).collect::<Vec<_>>();
    let capture_type_infos = discovered.iter().map(|capture| { let ty=capture.type_info.as_ref().ok_or_else(|| syn::Error::new(capture.name.span(), format!("cannot infer type of capture `{}`", capture.name)))?; Ok(quote! { <#ty as ::closure_pack::CompileType>::type_info() }) }).collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        ::closure_pack::Closure {
            captures: vec![#(#capture_type_infos),*],
            arguments: vec![#(#argument_type_infos),*],
            return_type: <#return_type as ::closure_pack::CompileType>::type_info(),
            body: #block,
        }
    })
}
