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

pub struct Compiler<'ctx> { pub(crate) context: &'ctx Context }

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self { Self { context } }

    /// Compile a closure through the type-erased backend implementation.
    ///
    /// The expensive MLIR construction path is deliberately non-generic so
    /// callers with many different argument/return types do not cause the
    /// backend implementation to be monomorphized repeatedly.
    pub fn compile_erased(&self, closure: &Closure) -> Result<(ExecutionEngine, String), String> {
        let (module, name) = self.build_module(closure, "compiled_closure", false)?;
        let engine = ExecutionEngine::new(&module, 0, &[], false, false);
        Ok((engine, name))
    }

    pub fn compile<Args, Ret>(&self, closure: &Closure) -> Result<CompiledClosure<'ctx, Args, Ret>, String>
    where Args: CompileType, Ret: CompileType + 'static {
        let (engine, name) = self.compile_erased(closure)?;
        Ok(CompiledClosure::new(engine, name))
    }

    pub fn compile_dynamic(&self, closure: &Closure) -> Result<DynamicCompiledClosure<'ctx>, String> {
        let (module, name) = self.build_module(closure, "compiled_dynamic_closure", true)?;
        let engine = ExecutionEngine::new(&module, 0, &[], false, false);
        Ok(DynamicCompiledClosure::new(engine, name, closure.arguments.clone(), closure.return_type.clone()))
    }

    fn build_module(&self, closure: &Closure, name: &str, dynamic: bool) -> Result<(Module<'ctx>, String), String> {
        let registry = DialectRegistry::new();
        register_all_dialects(&registry);
        self.context.append_dialect_registry(&registry);
        self.context.load_all_available_dialects();
        register_all_llvm_translations(self.context);
        let ir = MlirBuilder::new(name, closure, dynamic).build()?;
        let module = Module::parse(self.context, &ir).ok_or_else(|| format!("failed to parse generated MLIR:\n{}", ir))?;
        if !module.as_operation().verify() {
            return Err(format!("generated MLIR failed verification:\n{}", ir));
        }
        Ok((module, name.to_string()))
    }
}
