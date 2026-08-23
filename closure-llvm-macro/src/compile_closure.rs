use proc_macro::TokenStream;

use quote::quote;

use syn::parse_macro_input;

use crate::lowering::lower_block;
use crate::parser::ClosureInput;


// ============================================================
// compile_closure!
// ============================================================

pub(crate) fn expand(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(
            input as ClosureInput
        );

    match expand_compile_closure(input) {
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

fn expand_compile_closure(
    input: ClosureInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let ClosureInput {
        arguments,
        return_type,
        body,
    } = input;


    let locals =
        Vec::new();

    let expression =
        lower_block(
            &body.block,
            &arguments,
            &locals,
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


    let argument_types =
        arguments
            .iter()
            .map(|argument| &argument.type_info)
            .collect::<Vec<_>>();


    let tuple_type =
        if argument_types.is_empty() {
            quote! {
                ()
            }
        } else {
            quote! {
                (
                    #(
                        #argument_types,
                    )*
                )
            }
        };


    Ok(
        quote! {
            {
                let __closure =
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
                            ::closure_llvm::Block::expression(
                                #expression
                            ),
                    };


                let __context:
                    &'static ::inkwell::context::Context =
                    Box::leak(
                        Box::new(
                            ::inkwell::context::Context::create()
                        )
                    );


                let __compiler =
                    ::closure_llvm::Compiler::new(
                        __context
                    );


                __compiler
                    .compile::<
                        #tuple_type,
                        #return_type,
                    >(
                        &__closure
                    )
                    .expect(
                        "failed to compile closure"
                    )
            }
        }
    )
}