use proc_macro::TokenStream;

use quote::quote;

use std::collections::HashMap;

use syn::{
    parse_macro_input,
    BinOp,
    Data,
    DeriveInput,
    Expr,
    ExprBinary,
    ExprClosure,
    ExprField,
    ExprGroup,
    ExprLit,
    ExprParen,
    ExprPath,
    Fields,
    Lit,
    ReturnType,
};


#[proc_macro_derive(CompileType)]
pub fn derive_compile_type(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(input as DeriveInput);

    let name =
        &input.ident;

    let fields =
        match &input.data {
            Data::Struct(data) => {
                &data.fields
            }

            _ => {
                return syn::Error::new_spanned(
                    name,
                    "CompileType can only be derived for structs",
                )
                .to_compile_error()
                .into();
            }
        };

    let Fields::Named(fields) =
        fields
    else {
        return syn::Error::new_spanned(
            fields,
            "CompileType requires named fields",
        )
        .to_compile_error()
        .into();
    };

    let field_info =
        fields.named.iter().map(|field| {
            let field_name =
                field.ident.as_ref().unwrap();

            let field_type =
                &field.ty;

            quote! {
                ::closure_llvm::FieldInfo {
                    name: stringify!(#field_name),

                    type_info:
                        <#field_type as
                            ::closure_llvm::CompileType>
                            ::type_info,
                }
            }
        });

    let llvm_fields =
        fields.named.iter().map(|field| {
            let field_type =
                &field.ty;

            quote! {
                <#field_type as
                    ::closure_llvm::CompileType>
                    ::llvm_type(context)
            }
        });

    quote! {
        impl ::closure_llvm::CompileType
            for #name
        {
            fn type_info()
                -> ::closure_llvm::TypeInfo
            {
                ::closure_llvm::TypeInfo::Struct {
                    name: stringify!(#name),

                    fields: &[
                        #(#field_info),*
                    ],
                }
            }

            fn llvm_type<'ctx>(
                context:
                    &'ctx
                    ::inkwell::context::Context,
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
    .into()
}


// ============================================================
// compile_closure!
// ============================================================

#[proc_macro]
pub fn compile_closure(
    input: TokenStream,
) -> TokenStream {
    let closure =
        parse_macro_input!(
            input as ExprClosure
        );

    match compile_closure_impl(&closure) {
        Ok(tokens) =>
            tokens.into(),

        Err(error) =>
            error
                .to_compile_error()
                .into(),
    }
}


// ============================================================
// Closure compiler
// ============================================================

fn compile_closure_impl(
    closure: &ExprClosure,
) -> syn::Result<
    proc_macro2::TokenStream
> {
    let mut arguments =
        Vec::new();

    let mut environment =
        HashMap::new();

    for (index, input)
        in closure.inputs.iter().enumerate()
    {
        let syn::Pat::Type(pat_type) =
            input
        else {
            return Err(
                syn::Error::new_spanned(
                    input,
                    "closure arguments must have explicit types",
                )
            );
        };

        let syn::Pat::Ident(pattern) =
            &*pat_type.pat
        else {
            return Err(
                syn::Error::new_spanned(
                    &pat_type.pat,
                    "closure arguments must be identifiers",
                )
            );
        };

        environment.insert(
            pattern.ident.to_string(),
            index,
        );

        let ty =
            &pat_type.ty;

        arguments.push(
            quote! {
                <#ty as
                    ::closure_llvm::CompileType>
                    ::type_info()
            }
        );
    }

    let return_type =
        match &closure.output {
            ReturnType::Type(_, ty) => {
                quote! {
                    <#ty as
                        ::closure_llvm::CompileType>
                        ::type_info()
                }
            }

            ReturnType::Default => {
                return Err(
                    syn::Error::new_spanned(
                        &closure.output,
                        "closure must have an explicit return type",
                    )
                );
            }
        };

    let body =
        compile_expr(
            &closure.body,
            &environment,
        )?;

    Ok(
        quote! {
            ::closure_llvm::Closure {
                arguments: vec![
                    #(#arguments),*
                ],

                return_type: #return_type,

                body: #body,
            }
        }
    )
}


// ============================================================
// Expression lowering
// ============================================================

fn compile_expr(
    expr: &Expr,
    environment: &HashMap<String, usize>,
) -> syn::Result<proc_macro2::TokenStream> {
    eprintln!("compile_expr: {:?}", expr);

    match expr {
        // ----------------------------------------------------
        // Block
        // ----------------------------------------------------

        Expr::Block(expr_block) => {
            let statements = &expr_block.block.stmts;

            if statements.len() != 1 {
                return Err(
                    syn::Error::new_spanned(
                        &expr_block.block,
                        "only a single expression is supported",
                    )
                );
            }

            match &statements[0] {
                syn::Stmt::Expr(expr, _) => {
                    compile_expr(
                        expr,
                        environment,
                    )
                }

                statement => {
                    Err(
                        syn::Error::new_spanned(
                            statement,
                            "expected an expression",
                        )
                    )
                }
            }
        }


        // ----------------------------------------------------
        // Parentheses
        // ----------------------------------------------------

        Expr::Paren(expr_paren) => {
            compile_expr(
                &expr_paren.expr,
                environment,
            )
        }


        // ----------------------------------------------------
        // Group
        // ----------------------------------------------------

        Expr::Group(expr_group) => {
            compile_expr(
                &expr_group.expr,
                environment,
            )
        }


        // ----------------------------------------------------
        // Identifier
        // ----------------------------------------------------

        Expr::Path(expr_path) => {
            if expr_path.path.segments.len() != 1 {
                return Err(
                    syn::Error::new_spanned(
                        &expr_path.path,
                        "only simple identifiers are supported",
                    )
                );
            }

            let name =
                expr_path
                    .path
                    .segments
                    .first()
                    .unwrap()
                    .ident
                    .to_string();

            let index =
                environment
                    .get(&name)
                    .ok_or_else(|| {
                        syn::Error::new_spanned(
                            &expr_path.path,
                            format!(
                                "unknown closure argument `{}`",
                                name
                            ),
                        )
                    })?;

            Ok(
                quote! {
                    ::closure_llvm::Expr::Argument(#index)
                }
            )
        }


        // ----------------------------------------------------
        // Field access
        // ----------------------------------------------------

        Expr::Field(expr_field) => {
            let object =
                compile_expr(
                    &expr_field.base,
                    environment,
                )?;

            let field_name =
                match &expr_field.member {
                    syn::Member::Named(ident) => {
                        ident.to_string()
                    }

                    syn::Member::Unnamed(index) => {
                        index.index.to_string()
                    }
                };

            Ok(
                quote! {
                    ::closure_llvm::Expr::Field {
                        object: Box::new(#object),
                        name: #field_name.to_string(),
                    }
                }
            )
        }


        // ----------------------------------------------------
        // Binary expression
        // ----------------------------------------------------

        Expr::Binary(expr_binary) => {
            let lhs =
                compile_expr(
                    &expr_binary.left,
                    environment,
                )?;

            let rhs =
                compile_expr(
                    &expr_binary.right,
                    environment,
                )?;

            match &expr_binary.op {
                BinOp::Add(_) => {
                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Add {
                                lhs: Box::new(#lhs),
                                rhs: Box::new(#rhs),
                            }
                        }
                    )
                }

                BinOp::Sub(_) => {
                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Sub {
                                lhs: Box::new(#lhs),
                                rhs: Box::new(#rhs),
                            }
                        }
                    )
                }

                BinOp::Mul(_) => {
                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Mul {
                                lhs: Box::new(#lhs),
                                rhs: Box::new(#rhs),
                            }
                        }
                    )
                }

                BinOp::Div(_) => {
                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Div {
                                lhs: Box::new(#lhs),
                                rhs: Box::new(#rhs),
                            }
                        }
                    )
                }

                op => {
                    Err(
                        syn::Error::new_spanned(
                            op,
                            "unsupported binary operator",
                        )
                    )
                }
            }
        }


        // ----------------------------------------------------
        // Literals
        // ----------------------------------------------------

        Expr::Lit(expr_lit) => {
            match &expr_lit.lit {
                Lit::Int(value) => {
                    let value =
                        value.base10_parse::<i32>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::I32(#value)
                            )
                        }
                    )
                }

                Lit::Float(value) => {
                    let value =
                        value.base10_parse::<f64>()?;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::F64(#value)
                            )
                        }
                    )
                }

                Lit::Bool(value) => {
                    let value =
                        value.value;

                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Constant(
                                ::closure_llvm::Value::Bool(#value)
                            )
                        }
                    )
                }

                lit => {
                    Err(
                        syn::Error::new_spanned(
                            lit,
                            "unsupported literal",
                        )
                    )
                }
            }
        }


        // ----------------------------------------------------
        // Everything else
        // ----------------------------------------------------

        other => {
            Err(
                syn::Error::new_spanned(
                    other,
                    format!(
                        "unsupported expression: {:?}",
                        other
                    ),
                )
            )
        }
    }
}