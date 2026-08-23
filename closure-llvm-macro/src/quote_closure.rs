use proc_macro::TokenStream;

use quote::quote;

use syn::parse_macro_input;

use crate::lowering::lower_block;
use crate::parser::ClosureInput;


// ============================================================
// quote_closure!
// ============================================================

pub(crate) fn expand(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(
            input as ClosureInput
        );

    match expand_quote_closure(input) {
        Ok(tokens) =>
            tokens.into(),

        Err(error) =>
            error
                .into_compile_error()
                .into(),
    }
}


// ============================================================
// Implementation
// ============================================================

fn expand_quote_closure(
    input: ClosureInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let ClosureInput {
        arguments,
        return_type,
        body,
    } = input;


    let expression =
        lower_block(
            &body.block,
            &arguments,
            None,
        )?;


    let argument_type_infos =
        arguments
            .iter()
            .map(|argument| {
                let ty =
                    &argument.type_info;

                quote! {
                    <#ty as
                        ::closure_llvm::CompileType>
                        ::type_info()
                }
            })
            .collect::<Vec<_>>();


    Ok(
        quote! {
            ::closure_llvm::Closure {
                arguments:
                    vec![
                        #(
                            #argument_type_infos
                        ),*
                    ],

                return_type:
                    <#return_type as
                        ::closure_llvm::CompileType>
                        ::type_info(),

                body:
                    #expression,
            }
        }
    )
}