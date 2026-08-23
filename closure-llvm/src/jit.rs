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
    value::Value,
};


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
        Self { engine, function_name, _marker: PhantomData }
    }

    pub unsafe fn call(&self, value: &Args) -> Ret {
        jit_call::<Args, Ret>(&self.engine, &self.function_name, value)
    }
}


// ============================================================
// Dynamic compiled closure
// ============================================================

pub struct DynamicCompiledClosure<'ctx> {
    pub(crate) engine: ExecutionEngine<'ctx>,
    pub(crate) function_name: String,
    pub(crate) arguments: Vec<TypeInfo>,
    pub(crate) return_type: TypeInfo,
}

impl<'ctx> DynamicCompiledClosure<'ctx> {
    pub(crate) fn new(
        engine: ExecutionEngine<'ctx>,
        function_name: String,
        arguments: Vec<TypeInfo>,
        return_type: TypeInfo,
    ) -> Self {
        Self { engine, function_name, arguments, return_type }
    }

    pub unsafe fn call(&self, values: &[Value]) -> Result<Value, String> {
        if values.len() != self.arguments.len() {
            return Err(format!("expected {} arguments, got {}", self.arguments.len(), values.len()));
        }

        for (index, (value, expected_type)) in values.iter().zip(self.arguments.iter()).enumerate() {
            validate_value(index, value, expected_type)?;
        }

        type DynamicJitFn = unsafe extern "C" fn(*const *const u8, *mut u8);

        let function = self.engine.get_function::<DynamicJitFn>(&self.function_name)
            .map_err(|error| format!("failed to get dynamic JIT function: {:?}", error))?;

        let mut argument_storage = Vec::with_capacity(values.len());
        let mut argument_pointers = Vec::with_capacity(values.len());

        for value in values {
            let storage = value_to_bytes(value)?;
            argument_pointers.push(storage.as_ptr());
            argument_storage.push(storage);
        }

        let result_size = value_size(&self.return_type)?;
        let result_alignment = value_alignment(&self.return_type)?;
        let mut result = AlignedBuffer::new(result_size, result_alignment);

        function.call(argument_pointers.as_ptr(), result.as_mut_ptr());
        std::hint::black_box(&argument_storage);

        bytes_to_value(result.as_ptr(), &self.return_type)
    }
}


// ============================================================
// Dynamic value validation
// ============================================================

fn validate_value(index: usize, value: &Value, expected: &TypeInfo) -> Result<(), String> {
    let valid = match (value, expected) {
        (Value::Bool(_), TypeInfo::Bool) => true,
        (Value::I8(_), TypeInfo::I8) => true,
        (Value::I16(_), TypeInfo::I16) => true,
        (Value::I32(_), TypeInfo::I32) => true,
        (Value::I64(_), TypeInfo::I64) => true,
        (Value::I128(_), TypeInfo::I128) => true,
        (Value::U8(_), TypeInfo::U8) => true,
        (Value::U16(_), TypeInfo::U16) => true,
        (Value::U32(_), TypeInfo::U32) => true,
        (Value::U64(_), TypeInfo::U64) => true,
        (Value::U128(_), TypeInfo::U128) => true,
        (Value::F32(_), TypeInfo::F32) => true,
        (Value::F64(_), TypeInfo::F64) => true,
        _ => false,
    };

    if valid {
        Ok(())
    } else {
        Err(format!("argument {} has type {:?}, expected {:?}", index, value, expected))
    }
}


// ============================================================
// Value -> bytes
// ============================================================

fn value_to_bytes(value: &Value) -> Result<AlignedBuffer, String> {
    match value {
        Value::Bool(value) => {
            let mut buffer = AlignedBuffer::new(std::mem::size_of::<bool>(), std::mem::align_of::<bool>());
            unsafe { *(buffer.as_mut_ptr() as *mut bool) = *value; }
            Ok(buffer)
        }
        Value::I8(value) => scalar_to_buffer(*value),
        Value::I16(value) => scalar_to_buffer(*value),
        Value::I32(value) => scalar_to_buffer(*value),
        Value::I64(value) => scalar_to_buffer(*value),
        Value::I128(value) => scalar_to_buffer(*value),
        Value::U8(value) => scalar_to_buffer(*value),
        Value::U16(value) => scalar_to_buffer(*value),
        Value::U32(value) => scalar_to_buffer(*value),
        Value::U64(value) => scalar_to_buffer(*value),
        Value::U128(value) => scalar_to_buffer(*value),
        Value::F32(value) => scalar_to_buffer(*value),
        Value::F64(value) => scalar_to_buffer(*value),
        _ => Err(format!("dynamic invocation does not yet support value {:?}", value)),
    }
}

fn scalar_to_buffer<T: Copy>(value: T) -> Result<AlignedBuffer, String> {
    let mut buffer = AlignedBuffer::new(std::mem::size_of::<T>(), std::mem::align_of::<T>());
    unsafe { *(buffer.as_mut_ptr() as *mut T) = value; }
    Ok(buffer)
}


// ============================================================
// Result decoding
// ============================================================

fn bytes_to_value(pointer: *const u8, type_info: &TypeInfo) -> Result<Value, String> {
    unsafe {
        match type_info {
            TypeInfo::Bool => Ok(Value::Bool(*(pointer as *const bool))),
            TypeInfo::I8 => Ok(Value::I8(*(pointer as *const i8))),
            TypeInfo::I16 => Ok(Value::I16(*(pointer as *const i16))),
            TypeInfo::I32 => Ok(Value::I32(*(pointer as *const i32))),
            TypeInfo::I64 => Ok(Value::I64(*(pointer as *const i64))),
            TypeInfo::I128 => Ok(Value::I128(*(pointer as *const i128))),
            TypeInfo::U8 => Ok(Value::U8(*(pointer as *const u8))),
            TypeInfo::U16 => Ok(Value::U16(*(pointer as *const u16))),
            TypeInfo::U32 => Ok(Value::U32(*(pointer as *const u32))),
            TypeInfo::U64 => Ok(Value::U64(*(pointer as *const u64))),
            TypeInfo::U128 => Ok(Value::U128(*(pointer as *const u128))),
            TypeInfo::F32 => Ok(Value::F32(*(pointer as *const f32))),
            TypeInfo::F64 => Ok(Value::F64(*(pointer as *const f64))),
            _ => Err("dynamic result decoding for this type is not implemented yet".to_string()),
        }
    }
}


// ============================================================
// Size
// ============================================================

fn value_size(type_info: &TypeInfo) -> Result<usize, String> {
    match type_info {
        TypeInfo::Bool => Ok(std::mem::size_of::<bool>()),
        TypeInfo::I8 => Ok(std::mem::size_of::<i8>()),
        TypeInfo::I16 => Ok(std::mem::size_of::<i16>()),
        TypeInfo::I32 => Ok(std::mem::size_of::<i32>()),
        TypeInfo::I64 => Ok(std::mem::size_of::<i64>()),
        TypeInfo::I128 => Ok(std::mem::size_of::<i128>()),
        TypeInfo::U8 => Ok(std::mem::size_of::<u8>()),
        TypeInfo::U16 => Ok(std::mem::size_of::<u16>()),
        TypeInfo::U32 => Ok(std::mem::size_of::<u32>()),
        TypeInfo::U64 => Ok(std::mem::size_of::<u64>()),
        TypeInfo::U128 => Ok(std::mem::size_of::<u128>()),
        TypeInfo::F32 => Ok(std::mem::size_of::<f32>()),
        TypeInfo::F64 => Ok(std::mem::size_of::<f64>()),
        _ => Err("dynamic size calculation for this type is not implemented yet".to_string()),
    }
}


// ============================================================
// Alignment
// ============================================================

fn value_alignment(type_info: &TypeInfo) -> Result<usize, String> {
    match type_info {
        TypeInfo::Bool => Ok(std::mem::align_of::<bool>()),
        TypeInfo::I8 => Ok(std::mem::align_of::<i8>()),
        TypeInfo::I16 => Ok(std::mem::align_of::<i16>()),
        TypeInfo::I32 => Ok(std::mem::align_of::<i32>()),
        TypeInfo::I64 => Ok(std::mem::align_of::<i64>()),
        TypeInfo::I128 => Ok(std::mem::align_of::<i128>()),
        TypeInfo::U8 => Ok(std::mem::align_of::<u8>()),
        TypeInfo::U16 => Ok(std::mem::align_of::<u16>()),
        TypeInfo::U32 => Ok(std::mem::align_of::<u32>()),
        TypeInfo::U64 => Ok(std::mem::align_of::<u64>()),
        TypeInfo::U128 => Ok(std::mem::align_of::<u128>()),
        TypeInfo::F32 => Ok(std::mem::align_of::<f32>()),
        TypeInfo::F64 => Ok(std::mem::align_of::<f64>()),
        _ => Err("dynamic alignment calculation for this type is not implemented yet".to_string()),
    }
}


// ============================================================
// Aligned buffer
// ============================================================

struct AlignedBuffer {
    storage: Vec<u8>,
    alignment: usize,
}

impl AlignedBuffer {
    fn new(size: usize, alignment: usize) -> Self {
        let extra = alignment.saturating_sub(1);
        Self { storage: vec![0; size + extra], alignment }
    }

    fn as_ptr(&self) -> *const u8 { self.storage.as_ptr() }
    fn as_mut_ptr(&mut self) -> *mut u8 { self.storage.as_mut_ptr() }
}


// ============================================================
// Typed JIT invocation
// ============================================================

unsafe fn jit_call<Args, Ret>(engine: &ExecutionEngine<'_>, function_name: &str, value: &Args) -> Ret
where
    Args: CompileType,
    Ret: CompileType + 'static,
{
    type JitFn = unsafe extern "C" fn(*const u8, *mut u8);

    let function = engine.get_function::<JitFn>(function_name)
        .expect("failed to get JIT function");

    let mut result = MaybeUninit::<Ret>::uninit();
    function.call(value as *const Args as *const u8, result.as_mut_ptr() as *mut u8);
    result.assume_init()
}
