use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use crate::lowering::lower_block;
use crate::parser::ClosureInput;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ClosureInput);
    match expand_compile_closure(input) { Ok(tokens) => tokens.into(), Err(error) => error.into_compile_error().into() }
}

fn expand_compile_closure(input: ClosureInput) -> syn::Result<proc_macro2::TokenStream> {
    let ClosureInput { arguments, return_type, body } = input;
    let locals = Vec::new();
    let is_unit = matches!(&return_type, syn::Type::Tuple(tuple) if tuple.elems.is_empty());
    let block = if is_unit { lower_block(&body.block, &arguments, &locals, None)? } else { lower_block(&body.block, &arguments, &locals, Some(&return_type))? };
    let argument_type_infos = arguments.iter().map(|argument| { let ty = &argument.type_info; quote! { <#ty as ::closure_pack::CompileType>::type_info() } }).collect::<Vec<_>>();
    let argument_types = arguments.iter().map(|argument| &argument.type_info).collect::<Vec<_>>();
    let tuple_type = if argument_types.is_empty() { quote! { () } } else { quote! { (#(#argument_types,)*) } };
    Ok(quote! {{
        let __closure = ::closure_pack::Closure { arguments: vec![#(#argument_type_infos),*], return_type: <#return_type as ::closure_pack::CompileType>::type_info(), body: #block };
        let __context: &'static ::closure_pack::melior::Context = Box::leak(Box::new(::closure_pack::melior::Context::new()));
        let __compiler = ::closure_pack::Compiler::new(__context);
        let (__engine, __function_name) = __compiler.compile_erased(&__closure).expect("failed to compile closure");
        ::closure_pack::CompiledClosure::<#tuple_type, #return_type>::from_erased(__engine, __function_name)
    }})
}
