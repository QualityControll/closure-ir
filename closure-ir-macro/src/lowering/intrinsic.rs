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
    let path = match &*call.func {
        syn::Expr::Path(path) => path,
        other => return Err(syn::Error::new_spanned(other, "function calls currently require an intrinsic name")),
    };
    let name = intrinsic_name(path)?;
    let (variant, arity) = match name.as_str() {
        "sqrt" => (quote!(::closure_ir::Intrinsic::Sqrt), 1),
        "sin" => (quote!(::closure_ir::Intrinsic::Sin), 1),
        "cos" => (quote!(::closure_ir::Intrinsic::Cos), 1),
        "tan" => (quote!(::closure_ir::Intrinsic::Tan), 1),
        "exp" => (quote!(::closure_ir::Intrinsic::Exp), 1),
        "log" => (quote!(::closure_ir::Intrinsic::Log), 1),
        "abs" => (quote!(::closure_ir::Intrinsic::Abs), 1),
        "min" => (quote!(::closure_ir::Intrinsic::Min), 2),
        "max" => (quote!(::closure_ir::Intrinsic::Max), 2),
        "floor" => (quote!(::closure_ir::Intrinsic::Floor), 1),
        "ceil" => (quote!(::closure_ir::Intrinsic::Ceil), 1),
        "round" => (quote!(::closure_ir::Intrinsic::Round), 1),
        "pow" => (quote!(::closure_ir::Intrinsic::Pow), 2),
        _ => return Err(syn::Error::new_spanned(path, "unsupported intrinsic function")),
    };
    if call.args.len() != arity {
        return Err(syn::Error::new_spanned(call, format!("{} expects {} argument(s)", name, arity)));
    }
    let args = call.args.iter()
        .map(|arg| lower_expr(arg, arguments, locals, expected_type))
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        ::closure_ir::Expr::Intrinsic {
            intrinsic: #variant,
            arguments: vec![#(#args),*],
        }
    })
}

fn intrinsic_name(path: &ExprPath) -> syn::Result<String> {
    if path.path.segments.len() != 1 {
        return Err(syn::Error::new_spanned(path, "intrinsic names must be unqualified"));
    }
    Ok(path.path.segments[0].ident.to_string())
}
