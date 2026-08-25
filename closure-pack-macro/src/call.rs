use proc_macro::TokenStream;

use crate::parser::CallInput;
use quote::quote;
use syn::parse_macro_input;

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CallInput);
    let closure = input.closure;
    let values = input.values;

    quote! {
        unsafe {
            let mut args = (#(#values,)*);
            #closure.call(&mut args)
        }
    }
    .into()
}
