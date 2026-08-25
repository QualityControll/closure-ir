use super::expression::{lower_expr, LocalVariable};
use crate::parser::ClosureArgument;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprStruct, Member};

pub(crate) fn lower_struct(
    structure: &ExprStruct,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&syn::Type>,
) -> syn::Result<TokenStream> {
    let type_info = syn::Type::Path(syn::TypePath {
        qself: None,
        path: structure.path.clone(),
    });
    if let Some(expected) = expected_type {
        if quote!(#expected).to_string() != quote!(#type_info).to_string() {
            return Err(syn::Error::new_spanned(
                &structure.path,
                "struct literal type does not match expected type",
            ));
        }
    }

    let mut fields = Vec::new();
    for field in &structure.fields {
        let name = match &field.member {
            Member::Named(name) => name.clone(),
            Member::Unnamed(member) => {
                return Err(syn::Error::new_spanned(
                    member,
                    "struct literals must use named fields",
                ))
            }
        };
        let value = lower_expr(&field.expr, arguments, locals, None)?;
        fields.push(quote! { (stringify!(#name).to_string(), #value) });
    }

    Ok(quote! {
        ::closure_pack::Expr::Struct {
            type_info: <#type_info as ::closure_pack::CompileType>::type_info(),
            fields: vec![#(#fields),*],
        }
    })
}
