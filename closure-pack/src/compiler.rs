use std::sync::OnceLock;

use melior::{Context, ExecutionEngine};
use melior::dialect::DialectRegistry;
use melior::ir::Module;
use melior::ir::operation::OperationLike;
use melior::utility::{register_all_dialects, register_all_llvm_translations};
use crate::{expr::Closure, jit::{CompiledClosure, DynamicCompiledClosure}, types::CompileType};

mod addresses;
mod builder;
mod expr_lowering;
mod statements;

pub(crate) use builder::MlirBuilder;

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum RefKind { Value, Address }

#[derive(Clone)]
pub(crate) struct Ref {
    pub(crate) name: String,
    pub(crate) ty: crate::types::TypeInfo,
    pub(crate) kind: RefKind,
}

static GLOBAL_CONTEXT: OnceLock<Context> = OnceLock::new();

fn initialize_context(context: &Context) {
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();
    register_all_llvm_translations(context);
}

pub struct Compiler<'ctx> { pub(crate) context: &'ctx Context }

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        initialize_context(context);
        Self { context }
    }

    /// Returns a compiler backed by the process-wide MLIR context.
    ///
    /// The context is initialized lazily on the first call and then reused for
    /// all subsequent compilations. Closure-pack currently assumes compilation
    /// is single-threaded, so callers should not use this global compiler from
    /// multiple threads concurrently.
    pub fn global() -> Compiler<'static> {
        let context = GLOBAL_CONTEXT.get_or_init(|| {
            let context = Context::new();
            initialize_context(&context);
            context
        });
        Compiler { context }
    }

    pub fn compile<Args, Ret>(&self, closure: &Closure) -> Result<CompiledClosure<'ctx, Args, Ret>, String>
    where Args: CompileType, Ret: CompileType + 'static {
        let (module, name) = self.build_module(closure, "compiled_closure", false)?;
        let engine = ExecutionEngine::new(&module, 0, &[], false, false);
        Ok(CompiledClosure::new(engine, name))
    }

    pub fn compile_dynamic(&self, closure: &Closure) -> Result<DynamicCompiledClosure<'ctx>, String> {
        let (module, name) = self.build_module(closure, "compiled_dynamic_closure", true)?;
        let engine = ExecutionEngine::new(&module, 0, &[], false, false);
        Ok(DynamicCompiledClosure::new(engine, name, closure.arguments.clone(), closure.return_type.clone()))
    }

    fn build_module(&self, closure: &Closure, name: &str, dynamic: bool) -> Result<(Module<'ctx>, String), String> {
        let ir = MlirBuilder::new(name, closure, dynamic).build()?;
        let module = Module::parse(self.context, &ir).ok_or_else(|| format!("failed to parse generated MLIR:\n{}", ir))?;
        if !module.as_operation().verify() {
            return Err(format!("generated MLIR failed verification:\n{}", ir));
        }
        Ok((module, name.to_string()))
    }
}
