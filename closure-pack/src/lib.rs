mod compiler;
mod expr;
mod ffi;
mod jit;
mod operators;
mod types;
mod value;

pub use compiler::Compiler;
pub use expr::{Block, Closure, Expr, Intrinsic, Statement};
pub use ffi::ExternalFunction;
pub use jit::CompiledClosure;
pub use melior;
pub use types::{CompileType, FieldInfo, TypeInfo};
pub use value::Value;

pub use closure_pack_macro::{call, closure_pack, compile_closure, CompileType};

pub fn type_info_of<T, F>(_: F) -> TypeInfo
where
    T: CompileType,
    F: FnOnce() -> T,
{
    T::type_info()
}
pub fn type_info_of1<A, T, F>(_: F) -> TypeInfo
where
    A: CompileType,
    T: CompileType,
    F: FnOnce(A) -> T,
{
    T::type_info()
}
