use proc_macro::TokenStream;

use quote::quote;

use syn::{
    parse_macro_input,
    Expr,
    ExprBinary,
    ExprBlock,
    ExprClosure,
    ExprField,
    ExprLit,
    ExprPath,
    Fields,
    ItemStruct,
    Lit,
    Member,
    Pat,
    PatIdent,
    Type,
};


// ============================================================
// Type -> TypeInfo expression
// ============================================================

fn compile_type_expr(
    ty: &Type,
) -> proc_macro2::TokenStream {
    match ty {
        Type::Path(path) => {
            let ident = path
                .path
                .segments
                .last()
                .unwrap()
                .ident
                .clone();

            if ident == "f64" {
                quote! {
                    ::closure_llvm::TypeInfo::F64
                }
            } else if ident == "f32" {
                quote! {
                    ::closure_llvm::TypeInfo::F32
                }
            } else if ident == "i64" {
                quote! {
                    ::closure_llvm::TypeInfo::I64
                }
            } else if ident == "i32" {
                quote! {
                    ::closure_llvm::TypeInfo::I32
                }
            } else if ident == "i16" {
                quote! {
                    ::closure_llvm::TypeInfo::I16
                }
            } else if ident == "i8" {
                quote! {
                    ::closure_llvm::TypeInfo::I8
                }
            } else if ident == "u64" {
                quote! {
                    ::closure_llvm::TypeInfo::U64
                }
            } else if ident == "u32" {
                quote! {
                    ::closure_llvm::TypeInfo::U32
                }
            } else if ident == "u16" {
                quote! {
                    ::closure_llvm::TypeInfo::U16
                }
            } else if ident == "u8" {
                quote! {
                    ::closure_llvm::TypeInfo::U8
                }
            } else if ident == "bool" {
                quote! {
                    ::closure_llvm::TypeInfo::Bool
                }
            } else {
                quote! {
                    <#ty as ::closure_llvm::CompileType>::type_info()
                }
            }
        }

        _ => {
            quote! {
                <#ty as ::closure_llvm::CompileType>::type_info()
            }
        }
    }
}


// ============================================================
// Expression -> compiler IR
// ============================================================

fn expr_to_ir(
    expr: &Expr,
    argument: &syn::Ident,
) -> proc_macro2::TokenStream {
    match expr {

        // ----------------------------------------------------
        // Parenthesized expression
        // ----------------------------------------------------

        Expr::Paren(paren) => {
            expr_to_ir(
                &paren.expr,
                argument,
            )
        }


        // ----------------------------------------------------
        // Block expression
        // ----------------------------------------------------

        Expr::Block(
            ExprBlock {
                block,
                ..
            },
        ) => {
            if block.stmts.len() != 1 {
                panic!(
                    "closure body must contain exactly one expression"
                );
            }

            match &block.stmts[0] {
                syn::Stmt::Expr(
                    expr,
                    _,
                ) => {
                    expr_to_ir(
                        expr,
                        argument,
                    )
                }

                _ => {
                    panic!(
                        "closure body must contain exactly one expression"
                    );
                }
            }
        }


        // ----------------------------------------------------
        // Binary expression
        // ----------------------------------------------------

        Expr::Binary(
            ExprBinary {
                left,
                op,
                right,
                ..
            },
        ) => {
            let lhs =
                expr_to_ir(
                    left,
                    argument,
                );

            let rhs =
                expr_to_ir(
                    right,
                    argument,
                );

            if matches!(
                op,
                syn::BinOp::Sub(_)
            ) {
                quote! {
                    ::closure_llvm::Expr::Sub {
                        lhs: Box::new(#lhs),
                        rhs: Box::new(#rhs),
                    }
                }
            } else if matches!(
                op,
                syn::BinOp::Add(_)
            ) {
                quote! {
                    ::closure_llvm::Expr::Add {
                        lhs: Box::new(#lhs),
                        rhs: Box::new(#rhs),
                    }
                }
            } else if matches!(
                op,
                syn::BinOp::Mul(_)
            ) {
                quote! {
                    ::closure_llvm::Expr::Mul {
                        lhs: Box::new(#lhs),
                        rhs: Box::new(#rhs),
                    }
                }
            } else if matches!(
                op,
                syn::BinOp::Div(_)
            ) {
                quote! {
                    ::closure_llvm::Expr::Div {
                        lhs: Box::new(#lhs),
                        rhs: Box::new(#rhs),
                    }
                }
            } else {
                panic!(
                    "unsupported binary operator"
                );
            }
        }


        // ----------------------------------------------------
        // Field access
        // ----------------------------------------------------

        Expr::Field(
            ExprField {
                base,
                member,
                ..
            },
        ) => {
            let base_ir =
                expr_to_ir(
                    base,
                    argument,
                );

            let field_name =
                match member {
                    Member::Named(name) => {
                        name.to_string()
                    }

                    Member::Unnamed(index) => {
                        index.index.to_string()
                    }
                };

            quote! {
                ::closure_llvm::Expr::Field {
                    object: Box::new(#base_ir),
                    name: #field_name.to_string(),
                }
            }
        }


        // ----------------------------------------------------
        // Identifier / path
        // ----------------------------------------------------

        Expr::Path(
            ExprPath {
                path,
                ..
            },
        ) => {
            let ident =
                path.segments
                    .last()
                    .unwrap()
                    .ident
                    .clone();

            if ident == *argument {
                quote! {
                    ::closure_llvm::Expr::Argument(0)
                }
            } else {
                panic!(
                    "unsupported identifier: {}",
                    ident
                );
            }
        }


        // ----------------------------------------------------
        // f64 literal
        // ----------------------------------------------------

        Expr::Lit(
            ExprLit {
                lit: Lit::Float(value),
                ..
            },
        ) => {
            let value =
                value
                    .base10_parse::<f64>()
                    .unwrap();

            quote! {
                ::closure_llvm::Expr::Constant(
                    ::closure_llvm::Value::F64(
                        #value
                    )
                )
            }
        }


        // ----------------------------------------------------
        // Integer literal
        // ----------------------------------------------------

        Expr::Lit(
            ExprLit {
                lit: Lit::Int(value),
                ..
            },
        ) => {
            let value =
                value
                    .base10_parse::<i32>()
                    .unwrap();

            quote! {
                ::closure_llvm::Expr::Constant(
                    ::closure_llvm::Value::I32(
                        #value
                    )
                )
            }
        }


        // ----------------------------------------------------
        // Boolean literal
        // ----------------------------------------------------

        Expr::Lit(
            ExprLit {
                lit: Lit::Bool(value),
                ..
            },
        ) => {
            let value =
                value.value;

            quote! {
                ::closure_llvm::Expr::Constant(
                    ::closure_llvm::Value::Bool(
                        #value
                    )
                )
            }
        }


        // ----------------------------------------------------
        // Unsupported expression
        // ----------------------------------------------------

        other => {
            panic!(
                "unsupported expression: {}",
                quote!(#other)
            );
        }
    }
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


    // --------------------------------------------------------
    // Extract exactly one argument
    // --------------------------------------------------------

    let mut inputs =
        closure.inputs.iter();

    let first_input =
        inputs
            .next()
            .unwrap_or_else(|| {
                panic!(
                    "closure must have exactly one argument"
                )
            });

    if inputs.next().is_some() {
        panic!(
            "closure must have exactly one argument"
        );
    }


    // --------------------------------------------------------
    // Argument type
    // --------------------------------------------------------

    let argument_type =
        match first_input {
            Pat::Type(pat_type) => {
                (*pat_type.ty).clone()
            }

            _ => {
                panic!(
                    "closure argument must be explicitly typed"
                );
            }
        };


    // --------------------------------------------------------
    // Argument identifier
    // --------------------------------------------------------

    let argument_ident =
        match first_input {
            Pat::Type(pat_type) => {
                match &*pat_type.pat {
                    Pat::Ident(
                        PatIdent {
                            ident,
                            ..
                        },
                    ) => {
                        ident.clone()
                    }

                    _ => {
                        panic!(
                            "closure argument must be an identifier"
                        );
                    }
                }
            }

            _ => {
                unreachable!();
            }
        };


    // --------------------------------------------------------
    // Return type
    // --------------------------------------------------------

    let return_type =
        match &closure.output {
            syn::ReturnType::Type(
                _,
                ty,
            ) => {
                (**ty).clone()
            }

            syn::ReturnType::Default => {
                panic!(
                    "closure must specify a return type"
                );
            }
        };


    // --------------------------------------------------------
    // Type information
    // --------------------------------------------------------

    let argument_info =
        compile_type_expr(
            &argument_type,
        );

    let return_info =
        compile_type_expr(
            &return_type,
        );


    // --------------------------------------------------------
    // Convert closure body to compiler IR
    // --------------------------------------------------------

    let body =
        expr_to_ir(
            &closure.body,
            &argument_ident,
        );


    // --------------------------------------------------------
    // Generate compiled closure
    // --------------------------------------------------------
    //
    // IMPORTANT:
    //
    // CompiledClosure contains an ExecutionEngine<'ctx>.
    // Therefore the LLVM Context must outlive the returned
    // CompiledClosure.
    //
    // A local Context would be dropped at the end of this
    // generated block, producing:
    //
    //     error[E0716]: temporary value dropped while borrowed
    //
    // We intentionally leak the Context so that it has a
    // 'static lifetime.
    //
    // This is acceptable for the current JIT API because the
    // compiled closure is intended to remain usable for the
    // lifetime of the process.
    //
    // --------------------------------------------------------

    let expanded =
        quote! {
            {
                let context:
                    &'static ::inkwell::context::Context =
                    Box::leak(
                        Box::new(
                            ::inkwell::context::Context::create()
                        )
                    );


                let compiler =
                    ::closure_llvm::Compiler::new(
                        context,
                    );


                let closure =
                    ::closure_llvm::Closure {
                        arguments: vec![
                            #argument_info,
                        ],

                        return_type:
                            #return_info,

                        body:
                            #body,
                    };


                compiler
                    .compile::<
                        #argument_type,
                        #return_type,
                    >(
                        &closure,
                    )
                    .expect(
                        "failed to compile closure"
                    )
            }
        };


    expanded.into()
}


// ============================================================
// call!
// ============================================================
//
// Usage:
//
//     let result = call!(
//         compiled,
//         rectangle
//     );
//
// Expands approximately to:
//
//     unsafe {
//         compiled.call(&rectangle)
//     }
//
// ============================================================

#[proc_macro]
pub fn call(
    input: TokenStream,
) -> TokenStream {
    let args =
        syn::parse_macro_input!(
            input
            with syn::punctuated::Punctuated::<
                Expr,
                syn::Token![,]
            >::parse_terminated
        );


    if args.len() != 2 {
        panic!(
            "call! expects exactly two arguments: \
             call!(compiled, value)"
        );
    }


    let mut iter =
        args.into_iter();


    let compiled =
        iter.next().unwrap();

    let value =
        iter.next().unwrap();


    quote! {
        unsafe {
            (#compiled).call(
                &(#value)
            )
        }
    }
    .into()
}


// ============================================================
// #[derive(CompileType)]
// ============================================================

#[proc_macro_derive(CompileType)]
pub fn derive_compile_type(
    input: TokenStream,
) -> TokenStream {
    let item =
        parse_macro_input!(
            input as ItemStruct
        );


    let name =
        item.ident;


    // --------------------------------------------------------
    // Named fields only
    // --------------------------------------------------------

    let fields =
        match item.fields {
            Fields::Named(fields) => {
                fields.named
            }

            _ => {
                panic!(
                    "CompileType requires a struct with named fields"
                );
            }
        };


    // --------------------------------------------------------
    // Generate TypeInfo fields
    // --------------------------------------------------------

    let field_entries =
        fields
            .iter()
            .map(|field| {
                let field_name =
                    field
                        .ident
                        .as_ref()
                        .unwrap();

                let field_type =
                    &field.ty;

                let type_info =
                    compile_type_expr(
                        field_type,
                    );

                quote! {
                    ::closure_llvm::FieldInfo {
                        name:
                            stringify!(
                                #field_name
                            )
                            .to_string(),

                        type_info:
                            #type_info,
                    }
                }
            })
            .collect::<Vec<_>>();


    // --------------------------------------------------------
    // Generate LLVM field types
    // --------------------------------------------------------

    let field_types =
        fields
            .iter()
            .map(|field| {
                let ty =
                    &field.ty;

                quote! {
                    <#ty as ::closure_llvm::CompileType>
                        ::llvm_type(context)
                }
            })
            .collect::<Vec<_>>();


    // --------------------------------------------------------
    // Generate CompileType implementation
    // --------------------------------------------------------

    let expanded =
        quote! {
            impl ::closure_llvm::CompileType
                for #name
            {
                fn type_info()
                    -> ::closure_llvm::TypeInfo
                {
                    ::closure_llvm::TypeInfo::Struct {
                        name:
                            stringify!(
                                #name
                            )
                            .to_string(),

                        fields:
                            vec![
                                #(#field_entries),*
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
                                #(#field_types),*
                            ],
                            false,
                        )
                        .into()
                }
            }
        };


    expanded.into()
}