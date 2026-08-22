use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;

use syn::{
    parse_macro_input,
    Expr,
    ExprBinary,
    ExprClosure,
    ExprLit,
    ExprPath,
    Lit,
    Pat,
    PatIdent,
    PatType,
    Type as SynType,
};


// ============================================================
// #[derive(CompileType)]
// ============================================================

#[proc_macro_derive(CompileType)]
pub fn derive_compile_type(
    input: TokenStream,
) -> TokenStream {

    let input =
        parse_macro_input!(
            input as syn::DeriveInput
        );

    match generate_compile_type(input) {

        Ok(tokens) => {
            tokens.into()
        }

        Err(error) => {
            error
                .to_compile_error()
                .into()
        }
    }
}


fn generate_compile_type(
    input: syn::DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {

    let name =
        &input.ident;


    let syn::Data::Struct(data) =
        &input.data
    else {
        return Err(
            syn::Error::new_spanned(
                input,
                "CompileType only supports structs",
            )
        );
    };


    let syn::Fields::Named(fields) =
        &data.fields
    else {
        return Err(
            syn::Error::new_spanned(
                &data.fields,
                "CompileType requires named fields",
            )
        );
    };


    let static_name =
        syn::Ident::new(
            &format!(
                "__{}_FIELDS",
                name
            ),
            name.span(),
        );


    let mut field_info =
        Vec::new();

    let mut llvm_fields =
        Vec::new();


    for field in &fields.named {

        let field_name =
            field
                .ident
                .as_ref()
                .unwrap();


        let field_name_string =
            field_name.to_string();


        let field_type =
            &field.ty;


        // ----------------------------------------------------
        // IMPORTANT:
        //
        // Store the function pointer.
        //
        // Do NOT call type_info() here.
        // ----------------------------------------------------

        field_info.push(
            quote! {
                ::closure_llvm::FieldInfo {
                    name: #field_name_string,

                    type_info:
                        <#field_type as
                            ::closure_llvm::CompileType>
                            ::type_info,
                }
            }
        );


        llvm_fields.push(
            quote! {
                <#field_type as
                    ::closure_llvm::CompileType>
                    ::llvm_type(context)
            }
        );
    }


    let field_count =
        field_info.len();


    Ok(
        quote! {

            // ------------------------------------------------
            // Static field metadata
            // ------------------------------------------------

            static #static_name:
                [::closure_llvm::FieldInfo; #field_count]
                = [
                    #(#field_info),*
                ];


            // ------------------------------------------------
            // CompileType implementation
            // ------------------------------------------------

            impl ::closure_llvm::CompileType for #name {

                fn type_info()
                    -> ::closure_llvm::TypeInfo
                {
                    ::closure_llvm::TypeInfo::Struct {
                        name: stringify!(#name),

                        fields: &#static_name,
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
    )
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


    match compile_closure_impl(closure) {

        Ok(tokens) => {
            tokens.into()
        }

        Err(error) => {
            error
                .to_compile_error()
                .into()
        }
    }
}


// ============================================================
// Closure compiler
// ============================================================

fn compile_closure_impl(
    closure: ExprClosure,
) -> syn::Result<proc_macro2::TokenStream> {

    let mut arguments =
        Vec::new();

    let mut variables =
        HashMap::<String, usize>::new();


    // --------------------------------------------------------
    // Parameters
    // --------------------------------------------------------

    for (index, input)
        in closure.inputs.iter().enumerate()
    {

        let Pat::Type(
            PatType {
                pat,
                ty,
                ..
            }
        ) = input
        else {
            return Err(
                syn::Error::new_spanned(
                    input,
                    "closure parameters must have explicit types",
                )
            );
        };


        let Pat::Ident(
            PatIdent {
                ident,
                ..
            }
        ) = pat.as_ref()
        else {
            return Err(
                syn::Error::new_spanned(
                    pat,
                    "only identifier parameters are supported",
                )
            );
        };


        variables.insert(
            ident.to_string(),
            index,
        );


        arguments.push(
            compile_type(ty)?
        );
    }


    // --------------------------------------------------------
    // Return type
    // --------------------------------------------------------

    let return_type =
        match &closure.output {

            syn::ReturnType::Type(
                _,
                ty,
            ) => {
                compile_type(ty)?
            }

            syn::ReturnType::Default => {
                return Err(
                    syn::Error::new_spanned(
                        &closure.output,
                        "closure requires an explicit return type",
                    )
                );
            }
        };


    // --------------------------------------------------------
    // Body
    // --------------------------------------------------------

    let body =
        compile_block(
            &closure.body,
            &variables,
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
// Closure block handling
// ============================================================

fn compile_block(
    expr: &Expr,
    variables: &HashMap<String, usize>,
) -> syn::Result<
    proc_macro2::TokenStream
> {

    match expr {

        Expr::Block(block) => {

            let statements =
                &block.block.stmts;


            if statements.len() != 1 {
                return Err(
                    syn::Error::new_spanned(
                        &block.block,
                        "only a single expression is supported in a closure body",
                    )
                );
            }


            match &statements[0] {

                syn::Stmt::Expr(
                    expr,
                    _,
                ) => {
                    compile_expr(
                        expr,
                        variables,
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


        _ => {
            compile_expr(
                expr,
                variables,
            )
        }
    }
}


// ============================================================
// Type lowering
// ============================================================

fn compile_type(
    ty: &SynType,
) -> syn::Result<
    proc_macro2::TokenStream
> {

    let SynType::Path(path) =
        ty
    else {
        return Err(
            syn::Error::new_spanned(
                ty,
                "only simple types are supported",
            )
        );
    };


    let ident =
        path.path
            .get_ident()
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    ty,
                    "expected simple type",
                )
            })?;


    match ident.to_string().as_str() {

        "i32" => {
            Ok(
                quote! {
                    ::closure_llvm::TypeInfo::I32
                }
            )
        }


        "i64" => {
            Ok(
                quote! {
                    ::closure_llvm::TypeInfo::I64
                }
            )
        }


        "f32" => {
            Ok(
                quote! {
                    ::closure_llvm::TypeInfo::F32
                }
            )
        }


        "f64" => {
            Ok(
                quote! {
                    ::closure_llvm::TypeInfo::F64
                }
            )
        }


        "bool" => {
            Ok(
                quote! {
                    ::closure_llvm::TypeInfo::Bool
                }
            )
        }


        _ => {
            Ok(
                quote! {
                    <#ty as
                        ::closure_llvm::CompileType>
                        ::type_info()
                }
            )
        }
    }
}


// ============================================================
// Expression lowering
// ============================================================

fn compile_expr(
    expr: &Expr,
    variables: &HashMap<String, usize>,
) -> syn::Result<
    proc_macro2::TokenStream
> {

    match expr {

        Expr::Binary(binary) => {
            compile_binary(
                binary,
                variables,
            )
        }


        Expr::Path(path) => {
            compile_path(
                path,
                variables,
            )
        }


        Expr::Lit(lit) => {
            compile_literal(lit)
        }


        Expr::Paren(paren) => {
            compile_expr(
                &paren.expr,
                variables,
            )
        }


        Expr::Unary(unary) => {

            let value =
                compile_expr(
                    &unary.expr,
                    variables,
                )?;


            match &unary.op {

                syn::UnOp::Neg(_) => {
                    Ok(
                        quote! {
                            ::closure_llvm::Expr::Neg(
                                ::std::boxed::Box::new(
                                    #value
                                )
                            )
                        }
                    )
                }


                _ => {
                    Err(
                        syn::Error::new_spanned(
                            &unary.op,
                            "unsupported unary operator",
                        )
                    )
                }
            }
        }


        Expr::Field(field) => {

            let object =
                compile_expr(
                    &field.base,
                    variables,
                )?;


            let member =
                match &field.member {

                    syn::Member::Named(name) => {
                        name.to_string()
                    }


                    syn::Member::Unnamed(index) => {
                        return Err(
                            syn::Error::new_spanned(
                                index,
                                "tuple fields are not supported yet",
                            )
                        );
                    }
                };


            Ok(
                quote! {
                    ::closure_llvm::Expr::Field {
                        object:
                            ::std::boxed::Box::new(
                                #object
                            ),

                        name: #member,
                    }
                }
            )
        }


        _ => {
            Err(
                syn::Error::new_spanned(
                    expr,
                    "unsupported expression",
                )
            )
        }
    }
}


// ============================================================
// Binary expressions
// ============================================================

fn compile_binary(
    binary: &ExprBinary,
    variables: &HashMap<String, usize>,
) -> syn::Result<
    proc_macro2::TokenStream
> {

    let lhs =
        compile_expr(
            &binary.left,
            variables,
        )?;


    let rhs =
        compile_expr(
            &binary.right,
            variables,
        )?;


    match &binary.op {

        syn::BinOp::Add(_) => {
            Ok(
                quote! {
                    ::closure_llvm::Expr::Add(
                        ::std::boxed::Box::new(
                            #lhs
                        ),

                        ::std::boxed::Box::new(
                            #rhs
                        ),
                    )
                }
            )
        }


        syn::BinOp::Sub(_) => {
            Ok(
                quote! {
                    ::closure_llvm::Expr::Sub(
                        ::std::boxed::Box::new(
                            #lhs
                        ),

                        ::std::boxed::Box::new(
                            #rhs
                        ),
                    )
                }
            )
        }


        syn::BinOp::Mul(_) => {
            Ok(
                quote! {
                    ::closure_llvm::Expr::Mul(
                        ::std::boxed::Box::new(
                            #lhs
                        ),

                        ::std::boxed::Box::new(
                            #rhs
                        ),
                    )
                }
            )
        }


        syn::BinOp::Div(_) => {
            Ok(
                quote! {
                    ::closure_llvm::Expr::Div(
                        ::std::boxed::Box::new(
                            #lhs
                        ),

                        ::std::boxed::Box::new(
                            #rhs
                        ),
                    )
                }
            )
        }


        _ => {
            Err(
                syn::Error::new_spanned(
                    &binary.op,
                    "unsupported binary operator",
                )
            )
        }
    }
}


// ============================================================
// Variable references
// ============================================================

fn compile_path(
    path: &ExprPath,
    variables: &HashMap<String, usize>,
) -> syn::Result<
    proc_macro2::TokenStream
> {

    let ident =
        path.path
            .get_ident()
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    path,
                    "expected identifier",
                )
            })?;


    let index =
        variables
            .get(
                &ident.to_string()
            )
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    path,
                    format!(
                        "unknown variable `{}`",
                        ident
                    )
                )
            })?;


    Ok(
        quote! {
            ::closure_llvm::Expr::Argument(
                #index
            )
        }
    )
}


// ============================================================
// Literals
// ============================================================

fn compile_literal(
    literal: &ExprLit,
) -> syn::Result<
    proc_macro2::TokenStream
> {

    match &literal.lit {

        Lit::Int(value) => {

            let value:
                i32 =
                value.base10_parse()?;


            Ok(
                quote! {
                    ::closure_llvm::Expr::Constant(
                        ::closure_llvm::Value::I32(
                            #value
                        )
                    )
                }
            )
        }


        _ => {
            Err(
                syn::Error::new_spanned(
                    literal,
                    "unsupported literal",
                )
            )
        }
    }
}