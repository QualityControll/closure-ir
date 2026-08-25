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

// melior::Context is intentionally !Send + !Sync because MLIR contexts are not
// thread-safe. closure-pack currently assumes that compilation is single-
// threaded, so we keep the context behind a raw pointer in the process-wide
// singleton rather than requiring Context itself to satisfy Sync.
//
// The context is leaked for the lifetime of the process. This avoids running
// its destructor during shutdown, after other MLIR objects may already have
// been torn down.
struct GlobalContext(*const Context);

// SAFETY: closure-pack currently requires all compiler/MLIR access to happen
// from a single thread. This wrapper must not be used to make the global
// compiler concurrently accessible from multiple threads.
unsafe impl Send for GlobalContext {}
unsafe impl Sync for GlobalContext {}

static GLOBAL_CONTEXT: OnceLock<GlobalContext> = OnceLock::new();

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
    /// is single-threaded, so callers must not use this global compiler from
    /// multiple threads concurrently.
    pub fn global() -> Compiler<'static> {
        let global = GLOBAL_CONTEXT.get_or_init(|| {
            let context = Box::new(Context::new());
            initialize_context(&context);
            GlobalContext(Box::leak(context) as *const Context)
        });

        // SAFETY: GlobalContext is initialized exactly once and the Context is
        // deliberately leaked, so the pointed-to Context remains alive for the
        // remainder of the process. The single-threaded usage restriction is
        // part of the contract of Compiler::global().
        let context = unsafe { &*global.0 };
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
