use proc_macro::TokenStream;

mod call;
mod compile_closure;
mod compile_type;
mod parser;
mod quote_closure;

mod lowering;


// ============================================================
// CompileType
// ============================================================

#[proc_macro_derive(CompileType)]
pub fn derive_closure_type(
    input: TokenStream,
) -> TokenStream {
    compile_type::expand(input)
}


// ============================================================
// quote_closure!
// ============================================================

#[proc_macro]
pub fn quote_closure(
    input: TokenStream,
) -> TokenStream {
    quote_closure::expand(input)
}


// ============================================================
// compile_closure!
// ============================================================

#[proc_macro]
pub fn compile_closure(
    input: TokenStream,
) -> TokenStream {
    compile_closure::expand(input)
}


// ============================================================
// call!
// ============================================================

#[proc_macro]
pub fn call(
    input: TokenStream,
) -> TokenStream {
    call::expand(input)
}