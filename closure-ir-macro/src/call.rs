use proc_macro::TokenStream;

use quote::quote;

use syn::parse_macro_input;

use crate::parser::CallInput;


// ============================================================
// call!
// ============================================================

pub(crate) fn expand(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(
            input as CallInput
        );

    let closure =
        input.closure;

    let values =
        input.values;

    quote! {
        unsafe {
            #closure.call(
                &(#(#values,)*)
            )
        }
    }
    .into()
}