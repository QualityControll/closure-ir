use proc_macro::TokenStream;

use quote::quote;

use syn::{
    parse_macro_input,

    Data,
    DataStruct,
    DeriveInput,
    Fields,
};


// ============================================================
// Derive CompileType
// ============================================================

pub(crate) fn expand(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(
            input as DeriveInput
        );

    match expand_derive_closure_type(input) {
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

fn expand_derive_closure_type(
    input: DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let name =
        input.ident;

    let fields =
        match input.data {
            Data::Struct(DataStruct {
                fields,
                ..
            }) =>
                fields,

            _ =>
                return Err(
                    syn::Error::new_spanned(
                        name,
                        "CompileType can only be derived for structs",
                    )
                ),
        };


    // --------------------------------------------------------
    // TypeInfo fields
    // --------------------------------------------------------

    let field_infos =
        match &fields {
            Fields::Named(fields) => {
                fields
                    .named
                    .iter()
                    .map(|field| {
                        let field_name =
                            field
                                .ident
                                .as_ref()
                                .ok_or_else(|| {
                                    syn::Error::new_spanned(
                                        field,
                                        "expected named field",
                                    )
                                })?;

                        let ty =
                            &field.ty;

                        Ok(
                            quote! {
                                ::closure_ir::FieldInfo {
                                    name:
                                        stringify!(
                                            #field_name
                                        )
                                        .to_string(),

                                    type_info:
                                        <#ty as
                                            ::closure_ir::CompileType>
                                            ::type_info(),
                                }
                            }
                        )
                    })
                    .collect::<syn::Result<Vec<_>>>()?
            }

            Fields::Unnamed(fields) => {
                fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let ty =
                            &field.ty;

                        let index_string =
                            index.to_string();

                        Ok(
                            quote! {
                                ::closure_ir::FieldInfo {
                                    name:
                                        #index_string
                                            .to_string(),

                                    type_info:
                                        <#ty as
                                            ::closure_ir::CompileType>
                                            ::type_info(),
                                }
                            }
                        )
                    })
                    .collect::<syn::Result<Vec<_>>>()?
            }

            Fields::Unit =>
                Vec::new(),
        };


    // --------------------------------------------------------
    // LLVM fields
    // --------------------------------------------------------

    let llvm_fields =
        match &fields {
            Fields::Named(fields) => {
                fields
                    .named
                    .iter()
                    .map(|field| {
                        let ty =
                            &field.ty;

                        quote! {
                            <#ty as
                                ::closure_ir::CompileType>
                                ::llvm_type(context)
                        }
                    })
                    .collect::<Vec<_>>()
            }

            Fields::Unnamed(fields) => {
                fields
                    .unnamed
                    .iter()
                    .map(|field| {
                        let ty =
                            &field.ty;

                        quote! {
                            <#ty as
                                ::closure_ir::CompileType>
                                ::llvm_type(context)
                        }
                    })
                    .collect::<Vec<_>>()
            }

            Fields::Unit =>
                Vec::new(),
        };


    Ok(
        quote! {
            impl ::closure_ir::CompileType
                for #name
            {
                fn type_info()
                    -> ::closure_ir::TypeInfo
                {
                    ::closure_ir::TypeInfo::Struct {
                        name:
                            stringify!(
                                #name
                            )
                            .to_string(),

                        fields: vec![
                            #(#field_infos),*
                        ],
                    }
                }

                fn llvm_type<'ctx>(
                    context:
                        &'ctx ::inkwell::context::Context,
                )
                    -> ::inkwell::types::BasicTypeEnum<'ctx>
                {
                    context
                        .struct_type(
                            &[
                                #(#llvm_fields),*
                            ],
                            false,
                        )
                        .into()
                }
            }
        }
    )
}
