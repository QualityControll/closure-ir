use std::alloc::{alloc, dealloc, Layout};
use std::mem;
use std::ptr;

use crate::expr::Closure;
use crate::value::Value;

// ============================================================
// Value -> bytes
// ============================================================

fn value_to_bytes(
    value: &Value,
) -> Result<AlignedBuffer, String> {
    match value {
        Value::Bool(value) => {
            let mut buffer =
                AlignedBuffer::new(
                    std::mem::size_of::<bool>(),
                    std::mem::align_of::<bool>(),
                );

            unsafe {
                *(buffer.as_mut_ptr() as *mut bool) = *value;
            }

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
    }
}

// ============================================================
// Scalar buffer
// ============================================================

fn scalar_to_buffer<T: Copy>(value: T) -> AlignedBuffer {
    let size = mem::size_of::<T>();
    let align = mem::align_of::<T>();

    let mut buffer = AlignedBuffer::new(size, align);

    unsafe {
        ptr::copy_nonoverlapping(
            &value as *const T as *const u8,
            buffer.as_mut_ptr(),
            size,
        );
    }

    buffer
}

// ============================================================
// Aligned buffer
// ============================================================

struct AlignedBuffer {
    ptr: *mut u8,
    layout: Layout,
}

impl AlignedBuffer {
    fn new(size: usize, align: usize) -> Self {
        let layout = Layout::from_size_align(size.max(1), align)
            .expect("invalid aligned buffer layout");

        let ptr = unsafe { alloc(layout) };

        assert!(!ptr.is_null(), "failed to allocate aligned buffer");

        Self { ptr, layout }
    }

    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr, self.layout);
        }
    }
}
