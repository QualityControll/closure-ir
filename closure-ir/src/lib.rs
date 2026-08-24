mod compiler;
mod expr;
mod jit;
mod lowering;
mod operators;
mod statement_lowering;
mod types;
mod value;

pub use compiler::Compiler;
pub use expr::{Block, Closure, Expr, Intrinsic, Statement};
pub use jit::CompiledClosure;
pub use types::{CompileType, FieldInfo, TypeInfo};
pub use value::Value;

pub use closure_ir_macro::{
    call,
    closure_ir,
    compile_closure,
    CompileType,
};
