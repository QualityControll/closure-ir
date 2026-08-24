use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprCall, ExprPath, Type};

use crate::parser::ClosureArgument;
use super::expression::{expression_type, lower_expr, LocalVariable};

pub(crate) fn lower_call(
    call: &ExprCall,
    arguments: &[ClosureArgument],
    locals: &[LocalVariable],
    expected_type: Option<&Type>,
) -> syn::Result<TokenStream> {
    let function = match &*call.func {
        syn::Expr::Path(path) => path,
        other => return Err(syn::Error::new_spanned(other, "function calls currently require an intrinsic name")),
    };
    let name = intrinsic_name(function)?;
    let required_args = match name.as_str() {
        "abs" | "sqrt" => 1,
        "min" | "max" | "pow" => 2,
        _ => return Err(syn::Error::new_spanned(function, "unsupported intrinsic function")),
    };
    if call.args.len() != required_args {
        return Err(syn::Error::new_spanned(call, format!("{} expects {} argument(s)", name, required_args)));
    }
    let lowered = call.args.iter().map(|arg| lower_expr(arg, arguments, locals, expected_type)).collect::<syn::Result<Vec<_>>>()?;

    match name.as_str() {
        "abs" => {
            let arg = &lowered[0];
            let arg_type = expression_type(&call.args[0], arguments, locals).ok_or_else(|| syn::Error::new_spanned(&call.args[0], "cannot determine abs argument type"))?;
            let zero = zero_value(&arg_type, &call.args[0])?;
            Ok(quote! {
                ::closure_ir::Expr::IfElse {
                    condition: Box::new(::closure_ir::Expr::Lt {
                        lhs: Box::new(#arg),
                        rhs: Box::new(#zero),
                    }),
                    then_branch: Box::new(::closure_ir::Expr::Neg { operand: Box::new(#arg) }),
                    else_branch: Box::new(#arg),
                }
            })
        }
        "min" | "max" => {
            let lhs = &lowered[0];
            let rhs = &lowered[1];
            let condition = if name == "min" {
                quote! { ::closure_ir::Expr::Lt { lhs: Box::new(#lhs), rhs: Box::new(#rhs) } }
            } else {
                quote! { ::closure_ir::Expr::Gt { lhs: Box::new(#lhs), rhs: Box::new(#rhs) } }
            };
            Ok(quote! {
                ::closure_ir::Expr::IfElse {
                    condition: Box::new(#condition),
                    then_branch: Box::new(#lhs),
                    else_branch: Box::new(#rhs),
                }
            })
        }
        "sqrt" => {
            let arg = &lowered[0];
            let arg_type = expression_type(&call.args[0], arguments, locals).ok_or_else(|| syn::Error::new_spanned(&call.args[0], "cannot determine sqrt argument type"))?;
            match numeric_type_name(&arg_type).as_deref() {
                Some("f32") => Ok(quote! { ::closure_ir::Expr::SqrtF32 { operand: Box::new(#arg) } }),
                Some("f64") => Ok(quote! { ::closure_ir::Expr::SqrtF64 { operand: Box::new(#arg) } }),
                _ => Err(syn::Error::new_spanned(&call.args[0], "sqrt requires f32 or f64")),
            }
        }
        "pow" => {
            let lhs = &lowered[0];
            let rhs = &lowered[1];
            let lhs_type = expression_type(&call.args[0], arguments, locals).ok_or_else(|| syn::Error::new_spanned(&call.args[0], "cannot determine pow base type"))?;
            match numeric_type_name(&lhs_type).as_deref() {
                Some("f32") => Ok(quote! { ::closure_ir::Expr::PowF32 { lhs: Box::new(#lhs), rhs: Box::new(#rhs) } }),
                Some("f64") => Ok(quote! { ::closure_ir::Expr::PowF64 { lhs: Box::new(#lhs), rhs: Box::new(#rhs) } }),
                _ => Err(syn::Error::new_spanned(&call.args[0], "pow requires an f32 or f64 base")),
            }
        }
        _ => unreachable!(),
    }
}

fn intrinsic_name(path: &ExprPath) -> syn::Result<String> {
    if path.path.segments.len() != 1 {
        return Err(syn::Error::new_spanned(path, "intrinsic names must be unqualified"));
    }
    Ok(path.path.segments[0].ident.to_string())
}

fn numeric_type_name(ty: &Type) -> Option<String> {
    let rendered = quote!(#ty).to_string().replace(' ', "");
    match rendered.as_str() {
        "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "i128" | "u8" | "u16" | "u32" | "u64" | "u128" => Some(rendered),
        _ => None,
    }
}

fn zero_value(ty: &Type, span: &syn::Expr) -> syn::Result<TokenStream> {
    let ty = numeric_type_name(ty).unwrap_or_default();
    let value = match ty.as_str() {
        "f32" => quote! { ::closure_ir::Value::F32(0.0) },
        "f64" => quote! { ::closure_ir::Value::F64(0.0) },
        "i8" => quote! { ::closure_ir::Value::I8(0) },
        "i16" => quote! { ::closure_ir::Value::I16(0) },
        "i32" => quote! { ::closure_ir::Value::I32(0) },
        "i64" => quote! { ::closure_ir::Value::I64(0) },
        "i128" => quote! { ::closure_ir::Value::I128(0) },
        "u8" => quote! { ::closure_ir::Value::U8(0) },
        "u16" => quote! { ::closure_ir::Value::U16(0) },
        "u32" => quote! { ::closure_ir::Value::U32(0) },
        "u64" => quote! { ::closure_ir::Value::U64(0) },
        "u128" => quote! { ::closure_ir::Value::U128(0) },
        _ => return Err(syn::Error::new_spanned(span, "abs requires a supported numeric type")),
    };
    Ok(quote! { ::closure_ir::Expr::Constant(#value) })
}
