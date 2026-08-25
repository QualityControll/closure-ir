use proc_macro::TokenStream;

mod call;
mod captures;
mod closure_pack;
mod compile_closure;
mod compile_type;
mod lowering;
mod parser;

#[proc_macro_derive(CompileType)]
pub fn derive_closure_type(input: TokenStream) -> TokenStream {
    compile_type::expand(input)
}

#[proc_macro]
pub fn closure_pack(input: TokenStream) -> TokenStream {
    closure_pack::expand(input)
}

#[proc_macro]
pub fn compile_closure(input: TokenStream) -> TokenStream {
    compile_closure::expand(input)
}

#[proc_macro]
pub fn call(input: TokenStream) -> TokenStream {
    call::expand(input)
}
