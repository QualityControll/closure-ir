mod compiler;
mod expr;
mod jit;
mod lowering;
mod mlir_lowering;
mod mlir_types;
mod operators;
mod statement_lowering;
mod types;
mod value;

pub use compiler::Compiler;
pub use expr::{Block, Closure, Expr, Intrinsic, Statement};
pub use jit::CompiledClosure;
pub use types::{CompileType, FieldInfo, TypeInfo};
pub use value::Value;

pub use closure_ir_macro::{call, closure_ir, compile_closure, CompileType};

/// Returns the CompileType metadata for the return type of a zero-argument closure.
pub fn type_info_of<T, F>(_: F) -> TypeInfo
where
    T: CompileType,
    F: FnOnce() -> T,
{
    T::type_info()
}

/// Returns the CompileType metadata for the return type of a one-argument closure.
/// The argument is never invoked; it only supplies Rust with the closure's input type.
pub fn type_info_of1<A, T, F>(_: F) -> TypeInfo
where
    A: CompileType,
    T: CompileType,
    F: FnOnce(A) -> T,
{
    T::type_info()
}
