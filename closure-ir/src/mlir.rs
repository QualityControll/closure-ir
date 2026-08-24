//! MLIR-backed closure representation and compilation support.
//!
//! This module is the replacement boundary for the former custom Expr/Statement
//! IR.  The frontend will construct a Melior `Module` directly; the module is
//! retained by the compiled closure so that its MLIR can be inspected or
//! serialized before native lowering.

use melior::{
    dialect::DialectRegistry,
    ir::Module,
    Context,
};

/// An MLIR computation produced by the closure frontend.
pub struct Closure<'ctx> {
    pub(crate) context: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) function_name: String,
}

impl<'ctx> Closure<'ctx> {
    pub(crate) fn new(context: &'ctx Context, module: Module<'ctx>, function_name: String) -> Self {
        Self { context, module, function_name }
    }

    pub fn module(&self) -> &Module<'ctx> {
        &self.module
    }

    pub fn function_name(&self) -> &str {
        &self.function_name
    }
}

pub(crate) fn new_context() -> &'static Context {
    let registry = DialectRegistry::new();
    melior::utility::register_all_dialects(&registry);
    let context = Box::leak(Box::new(Context::new()));
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();
    context
}
