use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprLit, Lit, Type};

macro_rules! int {
    ($v:expr, $t:ty, $variant:ident) => {{
        let parsed = $v.base10_parse::<$t>()?;
        Ok(quote!(::closure_pack::Expr::Constant(::closure_pack::Value::$variant(#parsed))))
    }}
}

macro_rules! float {
    ($v:expr, $t:ty, $variant:ident) => {{
        let parsed = $v.base10_parse::<$t>()?;
        Ok(quote!(::closure_pack::Expr::Constant(::closure_pack::Value::$variant(#parsed))))
    }}
}

pub(crate) fn lower_literal(
    literal: &ExprLit,
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    match &literal.lit {
        Lit::Bool(value) => {
            let value = value.value;
            Ok(quote!(::closure_pack::Expr::Constant(::closure_pack::Value::Bool(#value))))
        }
        Lit::Int(value) => {
            let suffix = value.suffix();
            match suffix {
                "" => {
                    if let Some(ty) = expected_type {
                        lower_integer_with_type(value, ty)
                    } else {
                        let parsed = value.base10_parse::<i32>()?;
                        Ok(
                            quote!(::closure_pack::Expr::Constant(::closure_pack::Value::I32(#parsed))),
                        )
                    }
                }
                "i8" => int!(value, i8, I8),
                "i16" => int!(value, i16, I16),
                "i32" => int!(value, i32, I32),
                "i64" => int!(value, i64, I64),
                "i128" => int!(value, i128, I128),
                "u8" => int!(value, u8, U8),
                "u16" => int!(value, u16, U16),
                "u32" => int!(value, u32, U32),
                "u64" => int!(value, u64, U64),
                "u128" => int!(value, u128, U128),
                "usize" => int!(value, usize, Usize),
                _ => Err(syn::Error::new_spanned(
                    value,
                    "unsupported integer literal",
                )),
            }
        }
        Lit::Float(value) => {
            let suffix = value.suffix();
            match suffix {
                "" => {
                    if let Some(ty) = expected_type {
                        lower_float_with_type(value, ty)
                    } else {
                        let parsed = value.base10_parse::<f64>()?;
                        Ok(
                            quote!(::closure_pack::Expr::Constant(::closure_pack::Value::F64(#parsed))),
                        )
                    }
                }
                "f32" => float!(value, f32, F32),
                "f64" => float!(value, f64, F64),
                _ => Err(syn::Error::new_spanned(
                    value,
                    "unsupported floating-point literal",
                )),
            }
        }
        _ => Err(syn::Error::new_spanned(literal, "unsupported literal")),
    }
}

fn lower_integer_with_type(value: &syn::LitInt, ty: &Type) -> syn::Result<TokenStream> {
    let type_name = match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    };

    match type_name.as_deref() {
        Some("i8") => int!(value, i8, I8),
        Some("i16") => int!(value, i16, I16),
        Some("i32") => int!(value, i32, I32),
        Some("i64") => int!(value, i64, I64),
        Some("i128") => int!(value, i128, I128),
        Some("u8") => int!(value, u8, U8),
        Some("u16") => int!(value, u16, U16),
        Some("u32") => int!(value, u32, U32),
        Some("u64") => int!(value, u64, U64),
        Some("u128") => int!(value, u128, U128),
        Some("usize") => int!(value, usize, Usize),
        _ => Err(syn::Error::new_spanned(
            ty,
            "cannot infer integer literal type from this expression",
        )),
    }
}

fn lower_float_with_type(value: &syn::LitFloat, ty: &Type) -> syn::Result<TokenStream> {
    let type_name = match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    };

    match type_name.as_deref() {
        Some("f32") => float!(value, f32, F32),
        Some("f64") => float!(value, f64, F64),
        _ => Err(syn::Error::new_spanned(
            ty,
            "cannot infer floating-point literal type from this expression",
        )),
    }
}
