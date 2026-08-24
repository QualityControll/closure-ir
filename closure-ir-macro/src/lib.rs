use proc_macro::TokenStream;

mod call;
mod compile_closure;
mod compile_type;
mod parser;
mod closure_ir;

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
// closure_ir!
// ============================================================

#[proc_macro]
pub fn closure_ir(
    input: TokenStream,
) -> TokenStream {
    closure_ir::expand(input)
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