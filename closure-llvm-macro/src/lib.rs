use proc_macro::TokenStream;

use quote::quote;

use syn::{
    parse_macro_input,

    Data,
    DataStruct,
    DeriveInput,

    Expr as SynExpr,
    ExprBinary,
    ExprBlock,
    ExprField,
    ExprIf,
    ExprLit,
    ExprPath,

    Fields,
    Lit,
    Pat,
    ReturnType,
    Stmt,
    Type,
};


// ============================================================
// Derive CompileType
// ============================================================

#[proc_macro_derive(CompileType)]
pub fn derive_closure_type(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(
            input as DeriveInput
        );

    match expand_derive_closure_type(input) {
        Ok(tokens) =>
            tokens.into(),

        Err(error) =>
            error
                .into_compile_error()
                .into(),
    }
}


// ============================================================
// compile_closure!
// ============================================================

#[proc_macro]
pub fn compile_closure(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(
            input as ClosureInput
        );

    match expand_compile_closure(input) {
        Ok(tokens) =>
            tokens.into(),

        Err(error) =>
            error
                .into_compile_error()
                .into(),
    }
}


// ============================================================
// call!
// ============================================================

#[proc_macro]
pub fn call(
    input: TokenStream,
) -> TokenStream {
    let input =
        parse_macro_input!(
            input as CallInput
        );

    let closure =
        input.closure;

    let values =
        input.values;

    quote! {
        unsafe {
            #closure.call(
                &(#(#values,)*)
            )
        }
    }
    .into()
}


// ============================================================
// Derive CompileType
// ============================================================

fn expand_derive_closure_type(
    input: DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let name =
        input.ident;

    let fields =
        match input.data {
            Data::Struct(DataStruct {
                fields,
                ..
            }) =>
                fields,

            _ =>
                return Err(
                    syn::Error::new_spanned(
                        name,
                        "CompileType can only be derived for structs",
                    )
                ),
        };


    // --------------------------------------------------------
    // TypeInfo fields
    // --------------------------------------------------------

    let field_infos =
        match &fields {
            Fields::Named(fields) => {
                fields
                    .named
                    .iter()
                    .map(|field| {
                        let field_name =
                            field
                                .ident
                                .as_ref()
                                .ok_or_else(|| {
                                    syn::Error::new_spanned(
                                        field,
                                        "expected named field",
                                    )
                                })?;

                        let ty =
                            &field.ty;

                        Ok(
                            quote! {
                                ::closure_llvm::FieldInfo {
                                    name:
                                        stringify!(
                                            #field_name
                                        )
                                        .to_string(),

                                    type_info:
                                        <#ty as
                                            ::closure_llvm::CompileType>
                                            ::type_info(),
                                }
                            }
                        )
                    })
                    .collect::<syn::Result<Vec<_>>>()?
            }

            Fields::Unnamed(fields) => {
                fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        let ty =
                            &field.ty;

                        let index_string =
                            index.to_string();

                        Ok(
                            quote! {
                                ::closure_llvm::FieldInfo {
                                    name:
                                        #index_string
                                            .to_string(),

                                    type_info:
                                        <#ty as
                                            ::closure_llvm::CompileType>
                                            ::type_info(),
                                }
                            }
                        )
                    })
                    .collect::<syn::Result<Vec<_>>>()?
            }

            Fields::Unit =>
                Vec::new(),
        };


    // --------------------------------------------------------
    // LLVM fields
    // --------------------------------------------------------

    let llvm_fields =
        match &fields {
            Fields::Named(fields) => {
                fields
                    .named
                    .iter()
                    .map(|field| {
                        let ty =
                            &field.ty;

                        quote! {
                            <#ty as
                                ::closure_llvm::CompileType>
                                ::llvm_type(context)
                        }
                    })
                    .collect::<Vec<_>>()
            }

            Fields::Unnamed(fields) => {
                fields
                    .unnamed
                    .iter()
                    .map(|field| {
                        let ty =
                            &field.ty;

                        quote! {
                            <#ty as
                                ::closure_llvm::CompileType>
                                ::llvm_type(context)
                        }
                    })
                    .collect::<Vec<_>>()
            }

            Fields::Unit =>
                Vec::new(),
        };


    Ok(
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

                        fields: vec![
                            #(#field_infos),*
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
// Closure input
// ============================================================

struct ClosureInput {
    arguments: Vec<ClosureArgument>,
    return_type: Type,
    body: ExprBlock,
}


struct ClosureArgument {
    name: syn::Ident,
    type_info: Type,
}


impl syn::parse::Parse for ClosureInput {
    fn parse(
        input:
            syn::parse::ParseStream<'_>,
    ) -> syn::Result<Self> {
        let closure:
            syn::ExprClosure =
            input.parse()?;


        // ----------------------------------------------------
        // Arguments
        // ----------------------------------------------------

        let arguments =
            closure
                .inputs
                .iter()
                .map(|argument| {
                    match argument {
                        Pat::Type(pat_type) => {
                            let name =
                                match &*pat_type.pat {
                                    Pat::Ident(pat_ident) =>
                                        pat_ident
                                            .ident
                                            .clone(),

                                    _ =>
                                        return Err(
                                            syn::Error::new_spanned(
                                                &pat_type.pat,
                                                "closure arguments must be identifiers",
                                            )
                                        ),
                                };

                            Ok(
                                ClosureArgument {
                                    name,

                                    type_info:
                                        (*pat_type.ty)
                                            .clone(),
                                }
                            )
                        }

                        _ =>
                            Err(
                                syn::Error::new_spanned(
                                    argument,
                                    "closure arguments must have explicit types",
                                )
                            ),
                    }
                })
                .collect::<syn::Result<Vec<_>>>()?;


        // ----------------------------------------------------
        // Return type
        // ----------------------------------------------------

        let return_type =
            match closure.output {
                ReturnType::Type(_, ty) =>
                    (*ty).clone(),

                ReturnType::Default =>
                    return Err(
                        syn::Error::new_spanned(
                            &closure,
                            "closure must have an explicit return type",
                        )
                    ),
            };


        // ----------------------------------------------------
        // Body
        // ----------------------------------------------------

        let body =
            match *closure.body {
                SynExpr::Block(block) =>
                    block,

                other =>
                    return Err(
                        syn::Error::new_spanned(
                            other,
                            "closure body must be a block",
                        )
                    ),
            };


        Ok(Self {
            arguments,
            return_type,
            body,
        })
    }
}


// ============================================================
// call! input
// ============================================================

struct CallInput {
    closure: SynExpr,
    values: Vec<SynExpr>,
}


impl syn::parse::Parse for CallInput {
    fn parse(
        input:
            syn::parse::ParseStream<'_>,
    ) -> syn::Result<Self> {
        let closure =
            input.parse()?;

        let mut values =
            Vec::new();

        while !input.is_empty() {
            input.parse::<syn::Token![,]>()?;

            if input.is_empty() {
                break;
            }

            values.push(
                input.parse()?
            );
        }

        Ok(Self {
            closure,
            values,
        })
    }
}


// ============================================================
// Expand compile_closure!
// ============================================================

fn expand_compile_closure(
    input: ClosureInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let ClosureInput {
        arguments,
        return_type,
        body,
    } = input;

    let expression =
        lower_block(
            &body.block,
            &arguments,
            None,
        )?;


    let argument_type_infos =
        arguments
            .iter()
            .map(|argument| {
                let ty =
                    &argument.type_info;

                quote! {
                    <#ty as
                        ::closure_llvm::CompileType>
                        ::type_info()
                }
            })
            .collect::<Vec<_>>();


    let argument_types =
        arguments
            .iter()
            .map(|argument| &argument.type_info)
            .collect::<Vec<_>>();


    let tuple_type =
        if argument_types.is_empty() {
            quote! {
                ()
            }
        } else {
            quote! {
                (
                    #(
                        #argument_types,
                    )*
                )
            }
        };


    Ok(
        quote! {
            {
                let __closure =
                    ::closure_llvm::Closure {
                        arguments:
                            vec![
                                #(
                                    #argument_type_infos
                                ),*
                            ],

                        return_type:
                            <#return_type as
                                ::closure_llvm::CompileType>
                                ::type_info(),

                        body:
                            #expression,
                    };


                let __context:
                    &'static ::inkwell::context::Context =
                    Box::leak(
                        Box::new(
                            ::inkwell::context::Context::create()
                        )
                    );


                let __compiler =
                    ::closure_llvm::Compiler::new(
                        __context
                    );


                __compiler
                    .compile::<
                        #tuple_type,
                        #return_type,
                    >(
                        &__closure
                    )
                    .expect(
                        "failed to compile closure"
                    )
            }
        }
    )
}


// ============================================================
// Block
// ============================================================

fn lower_block(
    block: &syn::Block,
    arguments: &[ClosureArgument],
    expected_type: Option<&Type>,
) -> syn::Result<proc_macro2::TokenStream> {
    if block.stmts.len() != 1 {
        return Err(
            syn::Error::new_spanned(
                block,
                "compiled closure blocks must contain exactly one expression",
            )
        );
    }

    match &block.stmts[0] {
        Stmt::Expr(expr, _) =>
            lower_expr(
                expr,
                arguments,
                expected_type,
            ),

        other =>
            Err(
                syn::Error::new_spanned(
                    other,
                    "only expressions are supported",
                )
            ),
    }
}


// ============================================================
// Expression
// ============================================================

fn lower_expr(
    expr: &SynExpr,
    arguments: &[ClosureArgument],
    expected_type: Option<&Type>,
) -> syn::Result<proc_macro2::TokenStream> {
    match expr {
        SynExpr::Path(path) =>
            lower_path(
                path,
                arguments,
            ),

        SynExpr::Lit(literal) =>
            lower_literal(
                literal,
                expected_type,
            ),

        SynExpr::Binary(binary) =>
            lower_binary(
                binary,
                arguments,
                expected_type,
            ),

        SynExpr::Unary(unary) => {
            let operand =
                lower_expr(
                    &unary.expr,
                    arguments,
                    expected_type,
                )?;

            let expression =
                match unary.op {
                    syn::UnOp::Not(_) =>
                        quote! {
                            ::closure_llvm::Expr::Not
                        },

                    syn::UnOp::Neg(_) =>
                        quote! {
                            ::closure_llvm::Expr::Neg
                        },

                    _ =>
                        return Err(
                            syn::Error::new_spanned(
                                unary,
                                "unsupported unary operator",
                            )
                        ),
                };

            Ok(
                quote! {
                    #expression {
                        operand:
                            Box::new(#operand),
                    }
                }
            )
        }

        SynExpr::If(if_expr) =>
            lower_if(
                if_expr,
                arguments,
                expected_type,
            ),

        SynExpr::Field(field) =>
            lower_field(
                field,
                arguments,
            ),

        SynExpr::Paren(paren) =>
            lower_expr(
                &paren.expr,
                arguments,
                expected_type,
            ),

        _ =>
            Err(
                syn::Error::new_spanned(
                    expr,
                    "unsupported expression",
                )
            ),
    }
}


// ============================================================
// Argument
// ============================================================

fn lower_path(
    path: &ExprPath,
    arguments: &[ClosureArgument],
) -> syn::Result<proc_macro2::TokenStream> {
    if path.path.segments.len() != 1 {
        return Err(
            syn::Error::new_spanned(
                path,
                "only simple identifiers are supported",
            )
        );
    }

    let name =
        &path.path.segments[0].ident;

    let index =
        arguments
            .iter()
            .position(|argument| &argument.name == name)
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    path,
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


// ============================================================
// Literals
// ============================================================

fn lower_literal(
    literal: &ExprLit,
    expected_type: Option<&Type>,
) -> syn::Result<proc_macro2::TokenStream> {
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
                    // ------------------------------------------------
                    // Unsuffixed integer:
                    //
                    // Use the surrounding expression's expected type
                    // when available. This gives us Rust-like
                    // contextual typing for expressions such as:
                    //
                    //     a + 5
                    //
                    // where a: i8.
                    // ------------------------------------------------

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
) -> syn::Result<proc_macro2::TokenStream> {
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
) -> syn::Result<proc_macro2::TokenStream> {
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


// ============================================================
// Binary operations
// ============================================================

fn lower_binary(
    binary: &ExprBinary,
    arguments: &[ClosureArgument],
    expected_type: Option<&Type>,
) -> syn::Result<proc_macro2::TokenStream> {
    // --------------------------------------------------------
    // Determine the operand type.
    //
    // For:
    //
    //     a + 5
    //
    // `a` tells us that `5` should have the same type as `a`.
    //
    // We first look at the left operand, then the right operand,
    // and finally fall back to the expected result type.
    // --------------------------------------------------------

    let operand_type =
        expression_type(
            &binary.left,
            arguments,
        )
        .or_else(|| {
            expression_type(
                &binary.right,
                arguments,
            )
        })
        .or(expected_type);


    let lhs =
        lower_expr(
            &binary.left,
            arguments,
            operand_type,
        )?;

    let rhs =
        lower_expr(
            &binary.right,
            arguments,
            operand_type,
        )?;


    let operation =
        match binary.op {
            syn::BinOp::Add(_) =>
                quote! {
                    ::closure_llvm::Expr::Add
                },

            syn::BinOp::Sub(_) =>
                quote! {
                    ::closure_llvm::Expr::Sub
                },

            syn::BinOp::Mul(_) =>
                quote! {
                    ::closure_llvm::Expr::Mul
                },

            syn::BinOp::Div(_) =>
                quote! {
                    ::closure_llvm::Expr::Div
                },

            syn::BinOp::Rem(_) =>
                quote! {
                    ::closure_llvm::Expr::Rem
                },

            syn::BinOp::Eq(_) =>
                quote! {
                    ::closure_llvm::Expr::Eq
                },

            syn::BinOp::Ne(_) =>
                quote! {
                    ::closure_llvm::Expr::Ne
                },

            syn::BinOp::Lt(_) =>
                quote! {
                    ::closure_llvm::Expr::Lt
                },

            syn::BinOp::Le(_) =>
                quote! {
                    ::closure_llvm::Expr::Le
                },

            syn::BinOp::Gt(_) =>
                quote! {
                    ::closure_llvm::Expr::Gt
                },

            syn::BinOp::Ge(_) =>
                quote! {
                    ::closure_llvm::Expr::Ge
                },

            syn::BinOp::And(_) =>
                quote! {
                    ::closure_llvm::Expr::And
                },

            syn::BinOp::Or(_) =>
                quote! {
                    ::closure_llvm::Expr::Or
                },

            syn::BinOp::BitAnd(_) =>
                quote! {
                    ::closure_llvm::Expr::BitAnd
                },

            syn::BinOp::BitOr(_) =>
                quote! {
                    ::closure_llvm::Expr::BitOr
                },

            syn::BinOp::BitXor(_) =>
                quote! {
                    ::closure_llvm::Expr::BitXor
                },

            syn::BinOp::Shl(_) =>
                quote! {
                    ::closure_llvm::Expr::Shl
                },

            syn::BinOp::Shr(_) =>
                quote! {
                    ::closure_llvm::Expr::Shr
                },

            _ =>
                return Err(
                    syn::Error::new_spanned(
                        binary,
                        "unsupported binary operator",
                    )
                ),
        };


    Ok(
        quote! {
            #operation {
                lhs:
                    Box::new(#lhs),

                rhs:
                    Box::new(#rhs),
            }
        }
    )
}


// ============================================================
// Determine expression type
// ============================================================

fn expression_type<'a>(
    expr: &SynExpr,
    arguments: &'a [ClosureArgument],
) -> Option<&'a Type> {
    match expr {
        SynExpr::Path(path) => {
            if path.path.segments.len() != 1 {
                return None;
            }

            let name =
                &path.path.segments[0].ident;

            arguments
                .iter()
                .find(|argument| {
                    &argument.name == name
                })
                .map(|argument| {
                    &argument.type_info
                })
        }

        SynExpr::Paren(paren) =>
            expression_type(
                &paren.expr,
                arguments,
            ),

        SynExpr::Binary(binary) =>
            expression_type(
                &binary.left,
                arguments,
            )
            .or_else(|| {
                expression_type(
                    &binary.right,
                    arguments,
                )
            }),

        _ =>
            None,
    }
}


// ============================================================
// If / else
// ============================================================

fn lower_if(
    if_expr: &ExprIf,
    arguments: &[ClosureArgument],
    expected_type: Option<&Type>,
) -> syn::Result<proc_macro2::TokenStream> {
    let condition =
        lower_expr(
            &if_expr.cond,
            arguments,
            Some(&Type::Verbatim(
                quote! {
                    bool
                }
            )),
        )?;


    let then_branch =
        lower_block(
            &if_expr.then_branch,
            arguments,
            expected_type,
        )?;


    let else_branch =
        if_expr
            .else_branch
            .as_ref()
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    if_expr,
                    "if expressions require an else branch",
                )
            })?;


    let else_branch =
        match &*else_branch.1 {
            SynExpr::Block(block) =>
                lower_block(
                    &block.block,
                    arguments,
                    expected_type,
                )?,

            SynExpr::If(nested) =>
                lower_if(
                    nested,
                    arguments,
                    expected_type,
                )?,

            other =>
                return Err(
                    syn::Error::new_spanned(
                        other,
                        "else must contain a block or if expression",
                    )
                ),
        };


    Ok(
        quote! {
            ::closure_llvm::Expr::IfElse {
                condition:
                    Box::new(#condition),

                then_branch:
                    Box::new(#then_branch),

                else_branch:
                    Box::new(#else_branch),
            }
        }
    )
}


// ============================================================
// Field access
// ============================================================

fn lower_field(
    field: &ExprField,
    arguments: &[ClosureArgument],
) -> syn::Result<proc_macro2::TokenStream> {
    let object =
        lower_expr(
            &field.base,
            arguments,
            None,
        )?;


    let name =
        match &field.member {
            syn::Member::Named(name) =>
                name.to_string(),

            syn::Member::Unnamed(index) =>
                index.index.to_string(),
        };


    Ok(
        quote! {
            ::closure_llvm::Expr::Field {
                object:
                    Box::new(#object),

                name:
                    #name.to_string(),
            }
        }
    )
}