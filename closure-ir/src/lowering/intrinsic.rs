use inkwell::{builder::Builder, context::Context, intrinsics::Intrinsic as LlvmIntrinsic, values::{BasicValueEnum, FloatValue, FunctionValue, PointerValue}};
use crate::{expr::{Expr, Intrinsic}, types::TypeInfo};
use super::{LoweredValue, Lowering};

impl Lowering {
    pub(crate) fn lower_intrinsic<'ctx>(&self, context: &'ctx Context, builder: &Builder<'ctx>, function: FunctionValue<'ctx>, argument_pointers: &[PointerValue<'ctx>], argument_types: &[TypeInfo], expected_type: &TypeInfo, intrinsic: Intrinsic, arguments: &[Expr]) -> Result<LoweredValue<'ctx>, String> {
        let arity = match intrinsic { Intrinsic::Min | Intrinsic::Max | Intrinsic::Pow => 2, _ => 1 };
        if arguments.len() != arity { return Err(format!("{:?} expects {} argument(s), got {}", intrinsic, arity, arguments.len())); }
        if !matches!(expected_type, TypeInfo::F32 | TypeInfo::F64) { return Err(format!("{:?} requires an f32 or f64 result type", intrinsic)); }
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments { values.push(self.materialize_value(context, builder, self.lower_expr(context, builder, function, argument_pointers, argument_types, expected_type, argument)?)?); }
        let float_values = values.into_iter().map(|value| match value { BasicValueEnum::FloatValue(value) => Ok(value), _ => Err(format!("{:?} requires floating-point arguments", intrinsic)) }).collect::<Result<Vec<FloatValue<'ctx>>, String>>()?;
        if matches!(intrinsic, Intrinsic::Tan) { let sin = self.build_unary_intrinsic(builder, expected_type, "llvm.sin", float_values[0])?; let cos = self.build_unary_intrinsic(builder, expected_type, "llvm.cos", float_values[0])?; return Ok(LoweredValue::Value(builder.build_float_div(sin, cos, "tan").map_err(|e| format!("failed to build tan: {:?}", e))?.into())); }
        let name = match intrinsic { Intrinsic::Sqrt => "llvm.sqrt", Intrinsic::Abs => "llvm.fabs", Intrinsic::Min => "llvm.minnum", Intrinsic::Max => "llvm.maxnum", Intrinsic::Floor => "llvm.floor", Intrinsic::Ceil => "llvm.ceil", Intrinsic::Round => "llvm.round", Intrinsic::Sin => "llvm.sin", Intrinsic::Cos => "llvm.cos", Intrinsic::Exp => "llvm.exp", Intrinsic::Log => "llvm.log", Intrinsic::Pow => "llvm.pow", Intrinsic::Tan => unreachable!() };
        let result = if arguments.len() == 1 { self.build_unary_intrinsic(builder, expected_type, name, float_values[0])? } else { self.build_binary_intrinsic(builder, expected_type, name, float_values[0], float_values[1])? };
        Ok(LoweredValue::Value(result.into()))
    }

    fn build_unary_intrinsic<'ctx>(&self, builder: &Builder<'ctx>, expected_type: &TypeInfo, name: &str, value: FloatValue<'ctx>) -> Result<FloatValue<'ctx>, String> {
        let intrinsic = LlvmIntrinsic::find(name).ok_or_else(|| format!("LLVM intrinsic {} not found", name))?;
        let function = intrinsic.get_declaration(self.module(), &[value.get_type().into()]).ok_or_else(|| format!("failed to declare LLVM intrinsic {}", name))?;
        let call = builder.build_call(function, &[value.into()], "intrinsic").map_err(|e| format!("failed to build {}: {:?}", name, e))?;
        call.try_as_basic_value().basic().map(|v| v.into_float_value()).ok_or_else(|| format!("{} did not return a floating-point value", name))
    }

    fn build_binary_intrinsic<'ctx>(&self, builder: &Builder<'ctx>, expected_type: &TypeInfo, name: &str, lhs: FloatValue<'ctx>, rhs: FloatValue<'ctx>) -> Result<FloatValue<'ctx>, String> {
        let intrinsic = LlvmIntrinsic::find(name).ok_or_else(|| format!("LLVM intrinsic {} not found", name))?;
        let function = intrinsic.get_declaration(self.module(), &[lhs.get_type().into()]).ok_or_else(|| format!("failed to declare LLVM intrinsic {}", name))?;
        let call = builder.build_call(function, &[lhs.into(), rhs.into()], "intrinsic").map_err(|e| format!("failed to build {}: {:?}", name, e))?;
        call.try_as_basic_value().basic().map(|v| v.into_float_value()).ok_or_else(|| format!("{} did not return a floating-point value", name))
    }
}
