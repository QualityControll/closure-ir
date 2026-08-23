use std::marker::PhantomData;
use std::mem::MaybeUninit;

use inkwell::{
    execution_engine::ExecutionEngine,
};

use crate::{
    types::{
        CompileType,
        TypeInfo,
    },
};


// ============================================================
// Typed compiled closure
// ============================================================

pub struct CompiledClosure<'ctx, Args, Ret> {
    pub(crate) engine:
        ExecutionEngine<'ctx>,

    pub(crate) function_name:
        String,

    pub(crate) _marker:
        PhantomData<fn(Args) -> Ret>,
}


impl<'ctx, Args, Ret>
    CompiledClosure<'ctx, Args, Ret>
where
    Args: CompileType,
    Ret: CompileType + 'static,
{
    pub(crate) fn new(
        engine:
            ExecutionEngine<'ctx>,

        function_name:
            String,
    ) -> Self {
        Self {
            engine,
            function_name,

            _marker:
                PhantomData,
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
// Dynamic compiled closure
//
// This is used when a Closure has been deserialized.
//
// Unlike CompiledClosure<Args, Ret>, this type does not need
// Rust generic Args/Ret types.
//
// The Closure itself contains:
//
//     arguments: Vec<TypeInfo>
//     return_type: TypeInfo
//     body: Expr
//
// Those are sufficient for LLVM compilation.
//
// Runtime invocation will be added separately once we define
// the runtime Value -> memory ABI.
// ============================================================
#[allow(dead_code)]
pub struct DynamicCompiledClosure<'ctx> {
    pub(crate) engine:
        ExecutionEngine<'ctx>,

    pub(crate) function_name:
        String,

    pub(crate) arguments:
        Vec<TypeInfo>,

    pub(crate) return_type:
        TypeInfo,
}


impl<'ctx>
    DynamicCompiledClosure<'ctx>
{
    pub(crate) fn new(
        engine:
            ExecutionEngine<'ctx>,

        function_name:
            String,

        arguments:
            Vec<TypeInfo>,

        return_type:
            TypeInfo,
    ) -> Self {
        Self {
            engine,
            function_name,
            arguments,
            return_type,
        }
    }


    // ========================================================
    // Accessors
    // ========================================================

    pub fn arguments(
        &self,
    ) -> &[TypeInfo] {
        &self.arguments
    }


    pub fn return_type(
        &self,
    ) -> &TypeInfo {
        &self.return_type
    }


    // ========================================================
    // Runtime invocation
    //
    // Not implemented yet.
    //
    // The LLVM function has the ABI:
    //
    //     void compiled_closure(
    //         ptr args,
    //         ptr result
    //     )
    //
    // Before implementing this method, we need to define the
    // stable runtime memory representation for Value.
    // ========================================================

    pub unsafe fn call(
        &self,
        _arguments: &[crate::value::Value],
    ) -> Result<
        crate::value::Value,
        String,
    > {
        Err(
            "dynamic JIT invocation is not implemented yet"
                .to_string()
        )
    }
}


// ============================================================
// JIT invocation
//
// IMPORTANT:
//
// We intentionally do NOT return Ret directly from the JIT
// function.
//
// Returning an LLVM struct directly and asking Rust to interpret
// that return value as an arbitrary Rust tuple/struct can result
// in an ABI mismatch.
//
// Instead:
//
//     LLVM:
//         fn(args_ptr, result_ptr) -> void
//
//     Rust:
//         allocate MaybeUninit<Ret>
//         pass pointer to LLVM
//         read result
//
// This makes aggregate return values such as:
//
//     (i32, i32)
//     (i32, i64)
//     (f32, f64, i32)
//
// work without relying on aggregate return ABI compatibility.
// ============================================================

unsafe fn jit_call<Args, Ret>(
    engine:
        &ExecutionEngine<'_>,

    function_name:
        &str,

    value:
        &Args,
) -> Ret
where
    Args: CompileType,
    Ret: CompileType + 'static,
{
    type JitFn =
        unsafe extern "C" fn(
            *const u8,
            *mut u8,
        );


    let function =
        engine
            .get_function::<JitFn>(
                function_name,
            )
            .expect(
                "failed to get JIT function"
            );


    let mut result =
        MaybeUninit::<Ret>::uninit();


    function.call(
        value as *const Args
            as *const u8,

        result.as_mut_ptr()
            as *mut u8,
    );


    result.assume_init()
}