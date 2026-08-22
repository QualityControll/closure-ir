use std::marker::PhantomData;

use inkwell::{
    execution_engine::ExecutionEngine,
};

use crate::types::CompileType;


// ============================================================
// Compiled closure
// ============================================================

pub struct CompiledClosure<'ctx, Args, Ret> {
    pub(crate) engine: ExecutionEngine<'ctx>,
    pub(crate) function_name: String,

    pub(crate) _marker: PhantomData<fn(Args) -> Ret>,
}

impl<'ctx, Args, Ret> CompiledClosure<'ctx, Args, Ret>
where
    Args: CompileType,
    Ret: CompileType + 'static,
{
    pub(crate) fn new(
        engine: ExecutionEngine<'ctx>,
        function_name: String,
    ) -> Self {
        Self {
            engine,
            function_name,
            _marker: PhantomData,
        }
    }

    pub unsafe fn call(
        &self,
        value: &Args,
    ) -> Ret {
        jit_call::<Args, Ret>(
            &self.engine,
            &self.function_name,
            value,
        )
    }
}


// ============================================================
// JIT invocation
// ============================================================

unsafe fn jit_call<Args, Ret>(
    engine: &ExecutionEngine<'_>,
    function_name: &str,
    value: &Args,
) -> Ret
where
    Args: CompileType,
    Ret: CompileType + 'static,
{
    type JitFn<Ret> =
        unsafe extern "C" fn(*const u8) -> Ret;

    let function =
        engine
            .get_function::<JitFn<Ret>>(function_name)
            .expect("failed to get JIT function");

    function.call(
        value as *const Args as *const u8
    )
}