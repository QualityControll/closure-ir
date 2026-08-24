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
    let intrinsic = match name.as_str() {
        "abs" => "Abs",
        "min" => "Min",
        "max" => "Max",
        "sqrt" => "Sqrt",
        "pow" => "Pow",
        _ => return Err(syn::Error::new_spanned(function, "unsupported intrinsic function")),
    };
    let required_args = match intrinsic {
        "Abs" | "Sqrt" => 1,
        "Min" | "Max" | "Pow" => 2,
        _ => unreachable!(),
    };
    if call.args.len() != required_args {
        return Err(syn::Error::new_spanned(call, format!("{} expects {} argument(s)", name, required_args)));
    }
    let lowered = call.args.iter().map(|arg| lower_expr(arg, arguments, locals, expected_type)).collect::<syn::Result<Vec<_>>>()?;
    let intrinsic = syn::Ident::new(intrinsic, function.path.segments.last().unwrap().ident.span());
    Ok(quote! {
        ::closure_ir::Expr::Intrinsic {
            intrinsic: ::closure_ir::Intrinsic::#intrinsic,
            arguments: vec![#(#lowered),*],
        }
    })
}

fn intrinsic_name(path: &ExprPath) -> syn::Result<String> {
    if path.path.segments.len() != 1 {
        return Err(syn::Error::new_spanned(path, "intrinsic names must be unqualified"));
    }
    Ok(path.path.segments[0].ident.to_string())
}
