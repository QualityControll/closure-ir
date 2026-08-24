use crate::value::Value;
use super::{LoweredValue, Lowering};
use inkwell::{context::Context, types::BasicType, values::BasicValueEnum};

impl<'ctx> Lowering {
    pub(crate) fn lower_constant(
        &self,
        context: &'ctx Context,
        value: &Value,
    ) -> Result<LoweredValue<'ctx>, String> {
        let value = match value {
            Value::F32(v) => context.f32_type().const_float(*v as f64).into(),
            Value::F64(v) => context.f64_type().const_float(*v).into(),
            Value::I8(v) => context.i8_type().const_int(*v as i64 as u64, true).into(),
            Value::I16(v) => context.i16_type().const_int(*v as i64 as u64, true).into(),
            Value::I32(v) => context.i32_type().const_int(*v as i64 as u64, true).into(),
            Value::I64(v) => context.i64_type().const_int(*v as u64, true).into(),
            Value::I128(v) => context
                .i128_type()
                .const_int_arbitrary_precision(&[*v as u128 as u64, (*v as u128 >> 64) as u64])
                .into(),
            Value::U8(v) => context.i8_type().const_int(*v as u64, false).into(),
            Value::U16(v) => context.i16_type().const_int(*v as u64, false).into(),
            Value::U32(v) => context.i32_type().const_int(*v as u64, false).into(),
            Value::U64(v) => context.i64_type().const_int(*v, false).into(),
            Value::U128(v) => context
                .i128_type()
                .const_int_arbitrary_precision(&[*v as u64, (*v >> 64) as u64])
                .into(),
            Value::Bool(v) => context
                .bool_type()
                .const_int(if *v { 1 } else { 0 }, false)
                .into(),
            Value::Array(values) => {
                let elems = values
                    .iter()
                    .map(|v| {
                        self.lower_constant(context, v).and_then(|v| match v {
                            LoweredValue::Value(v) => Ok(v),
                            _ => Err("array constant element was not a value".into()),
                        })
                    })
                    .collect::<Result<Vec<BasicValueEnum<'ctx>>, String>>()?;

                let first = elems
                    .first()
                    .ok_or("empty array constants are not supported")?;

                match first {
                    BasicValueEnum::Int(_) => {
                        let values = elems
                            .iter()
                            .map(|v| v.into_int_value())
                            .collect::<Vec<_>>();
                        first.get_type().into_int_type().const_array(&values).into()
                    }
                    BasicValueEnum::Float(_) => {
                        let values = elems
                            .iter()
                            .map(|v| v.into_float_value())
                            .collect::<Vec<_>>();
                        first.get_type().into_float_type().const_array(&values).into()
                    }
                    BasicValueEnum::Array(_) => {
                        let values = elems
                            .iter()
                            .map(|v| v.into_array_value())
                            .collect::<Vec<_>>();
                        first.get_type().into_array_type().const_array(&values).into()
                    }
                    BasicValueEnum::Struct(_) => {
                        let values = elems
                            .iter()
                            .map(|v| v.into_struct_value())
                            .collect::<Vec<_>>();
                        first.get_type().into_struct_type().const_array(&values).into()
                    }
                    BasicValueEnum::Pointer(_) => {
                        let values = elems
                            .iter()
                            .map(|v| v.into_pointer_value())
                            .collect::<Vec<_>>();
                        first.get_type().into_pointer_type().const_array(&values).into()
                    }
                    BasicValueEnum::Vector(_) => {
                        let values = elems
                            .iter()
                            .map(|v| v.into_vector_value())
                            .collect::<Vec<_>>();
                        first.get_type().into_vector_type().const_array(&values).into()
                    }
                    _ => return Err("unsupported array constant element type".into()),
                }
            }
        };

        Ok(LoweredValue::Value(value))
    }
}
