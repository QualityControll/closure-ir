use proc_macro2::TokenStream;

use quote::quote;

use syn::{
    ExprLit,
    Lit,
    Type,
};


// ============================================================
// Literals
// ============================================================

pub(crate) fn lower_literal(
    literal: &ExprLit,
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    match &literal.lit {
        Lit::Bool(value) => {
            let value =
                value.value;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::Bool(
                            #value
                        )
                    )
                }
            )
        }

        Lit::Int(value) => {
            let suffix =
                value.suffix();

            match suffix {
                "" => {
                    if let Some(ty) = expected_type {
                        lower_integer_with_type(
                            value,
                            ty,
                        )
                    } else {
                        let parsed =
                            value.base10_parse::<i32>()?;

                        Ok(
                            quote! {
                                ::closure_llvm::Expr::Constant(
                                    ::closure_llvm::Value::I32(
                                        #parsed
                                    )
                                )
                            }
                        )
                    }
                }

                "i8" => {
                    let parsed =
                        value.base10_parse::<i8>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::I8(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "i16" => {
                    let parsed =
                        value.base10_parse::<i16>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::I16(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "i32" => {
                    let parsed =
                        value.base10_parse::<i32>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::I32(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "i64" => {
                    let parsed =
                        value.base10_parse::<i64>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::I64(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "i128" => {
                    let parsed =
                        value.base10_parse::<i128>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::I128(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "u8" => {
                    let parsed =
                        value.base10_parse::<u8>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::U8(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "u16" => {
                    let parsed =
                        value.base10_parse::<u16>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::U16(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "u32" => {
                    let parsed =
                        value.base10_parse::<u32>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::U32(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "u64" => {
                    let parsed =
                        value.base10_parse::<u64>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::U64(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "u128" => {
                    let parsed =
                        value.base10_parse::<u128>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::U128(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                _ =>
                    Err(
                        syn::Error::new_spanned(
                            value,
                            "unsupported integer literal",
                        )
                    ),
            }
        }

        Lit::Float(value) => {
            let suffix =
                value.suffix();

            match suffix {
                "" => {
                    if let Some(ty) = expected_type {
                        lower_float_with_type(
                            value,
                            ty,
                        )
                    } else {
                        let parsed =
                            value.base10_parse::<f64>()?;

                        Ok(
                            quote! {
                                ::closure_llvm::Expr::Constant(
                                    ::closure_llvm::Value::F64(
                                        #parsed
                                    )
                                )
                            }
                        )
                    }
                }

                "f32" => {
                    let parsed =
                        value.base10_parse::<f32>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::F32(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                "f64" => {
                    let parsed =
                        value.base10_parse::<f64>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::F64(
                                    #parsed
                                )
                            )
                        }
                    )
                }

                _ =>
                    Err(
                        syn::Error::new_spanned(
                            value,
                            "unsupported floating-point literal",
                        )
                    ),
            }
        }

        _ =>
            Err(
                syn::Error::new_spanned(
                    literal,
                    "unsupported literal",
                )
            ),
    }
}


// ============================================================
// Contextual integer literal
// ============================================================

fn lower_integer_with_type(
    value: &syn::LitInt,
    ty: &Type,
) -> syn::Result<TokenStream> {
    let type_name =
        match ty {
            Type::Path(type_path) =>
                type_path
                    .path
                    .segments
                    .last()
                    .map(|segment| {
                        segment.ident.to_string()
                    }),

            _ =>
                None,
        };

    match type_name.as_deref() {
        Some("i8") => {
            let parsed =
                value.base10_parse::<i8>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::I8(#parsed)
                    )
                }
            )
        }

        Some("i16") => {
            let parsed =
                value.base10_parse::<i16>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::I16(#parsed)
                    )
                }
            )
        }

        Some("i32") => {
            let parsed =
                value.base10_parse::<i32>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::I32(#parsed)
                    )
                }
            )
        }

        Some("i64") => {
            let parsed =
                value.base10_parse::<i64>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::I64(#parsed)
                    )
                }
            )
        }

        Some("i128") => {
            let parsed =
                value.base10_parse::<i128>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::I128(#parsed)
                    )
                }
            )
        }

        Some("u8") => {
            let parsed =
                value.base10_parse::<u8>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::U8(#parsed)
                    )
                }
            )
        }

        Some("u16") => {
            let parsed =
                value.base10_parse::<u16>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::U16(#parsed)
                    )
                }
            )
        }

        Some("u32") => {
            let parsed =
                value.base10_parse::<u32>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::U32(#parsed)
                    )
                }
            )
        }

        Some("u64") => {
            let parsed =
                value.base10_parse::<u64>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::U64(#parsed)
                    )
                }
            )
        }

        Some("u128") => {
            let parsed =
                value.base10_parse::<u128>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::U128(#parsed)
                    )
                }
            )
        }

        _ =>
            Err(
                syn::Error::new_spanned(
                    ty,
                    "cannot infer integer literal type from this expression",
                )
            ),
    }
}


// ============================================================
// Contextual floating-point literal
// ============================================================

fn lower_float_with_type(
    value: &syn::LitFloat,
    ty: &Type,
) -> syn::Result<TokenStream> {
    let type_name =
        match ty {
            Type::Path(type_path) =>
                type_path
                    .path
                    .segments
                    .last()
                    .map(|segment| {
                        segment.ident.to_string()
                    }),

            _ =>
                None,
        };

    match type_name.as_deref() {
        Some("f32") => {
            let parsed =
                value.base10_parse::<f32>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::F32(#parsed)
                    )
                }
            )
        }

        Some("f64") => {
            let parsed =
                value.base10_parse::<f64>()?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::F64(#parsed)
                    )
                }
            )
        }

        _ =>
            Err(
                syn::Error::new_spanned(
                    ty,
                    "cannot infer floating-point literal type from this expression",
                )
            ),
    }
}