use proc_macro2::TokenStream;
use quote::quote;
use syn::{ExprCall, ExprPath, Type};

use crate::parser::ClosureArgument;
use super::expression::{lower_expr, LocalVariable};

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
        "abs" => Ok(quote! {
            ::closure_ir::Expr::IfElse {
                condition: Box::new(::closure_ir::Expr::Lt {
                    lhs: Box::new(#lowered[0]),
                    rhs: Box::new(::closure_ir::Expr::Constant(::closure_ir::Value::I32(0))),
                }),
                then_branch: Box::new(::closure_ir::Expr::Neg { operand: Box::new(#lowered[0]) }),
                else_branch: Box::new(#lowered[0]),
            }
        }),
        "min" | "max" => {
            let lhs = &lowered[0];
            let rhs = &lowered[1];
            let condition = if name == "min" { quote! { ::closure_ir::Expr::Lt { lhs: Box::new(#lhs), rhs: Box::new(#rhs) } } } else { quote! { ::closure_ir::Expr::Gt { lhs: Box::new(#lhs), rhs: Box::new(#rhs) } } };
            Ok(quote! {
                ::closure_ir::Expr::IfElse {
                    condition: Box::new(#condition),
                    then_branch: Box::new(#lhs),
                    else_branch: Box::new(#rhs),
                }
            })
        }
        "sqrt" | "pow" => Err(syn::Error::new_spanned(function, "sqrt and pow are reserved for a future LLVM intrinsic implementation")),
        _ => unreachable!(),
    }
}

fn intrinsic_name(path: &ExprPath) -> syn::Result<String> {
    if path.path.segments.len() != 1 {
        return Err(syn::Error::new_spanned(path, "intrinsic names must be unqualified"));
    }
    Ok(path.path.segments[0].ident.to_string())
}
