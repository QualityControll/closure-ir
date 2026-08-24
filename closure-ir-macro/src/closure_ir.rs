use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use crate::lowering::lower_block;
use crate::parser::ClosureInput;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ClosureInput);
    match expand_closure_ir(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_closure_ir(input: ClosureInput) -> syn::Result<proc_macro2::TokenStream> {
    let ClosureInput { arguments, return_type, body } = input;
    let locals = Vec::new();
    let block = lower_block(&body.block, &arguments, &locals, Some(&return_type))?;

    let argument_type_infos = arguments.iter().map(|argument| {
        let ty = &argument.type_info;
        quote! { <#ty as ::closure_ir::CompileType>::type_info() }
    }).collect::<Vec<_>>();

    Ok(quote! {
        ::closure_ir::Closure {
            arguments: vec![#(#argument_type_infos),*],
            return_type: <#return_type as ::closure_ir::CompileType>::type_info(),
            body: #block,
        }
    })
}
