use inkwell::{builder::Builder, context::Context, values::{BasicValueEnum, FloatValue, FunctionValue, PointerValue}};
use crate::{expr::{Expr, Intrinsic}, types::TypeInfo};
use super::{LoweredValue, Lowering};

impl<'ctx> Lowering<'ctx> {
    pub(crate) fn lower_intrinsic(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        function: FunctionValue<'ctx>,
        argument_pointers: &[PointerValue<'ctx>],
        argument_types: &[TypeInfo],
        expected_type: &TypeInfo,
        intrinsic: Intrinsic,
        arguments: &[Expr],
    ) -> Result<LoweredValue<'ctx>, String> {
        let expected_arguments = match intrinsic {
            Intrinsic::Min | Intrinsic::Max | Intrinsic::Pow => 2,
            _ => 1,
        };
        if arguments.len() != expected_arguments {
            return Err(format!("{:?} expects {} argument(s), got {}", intrinsic, expected_arguments, arguments.len()));
        }

        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let value = self.lower_expr(
                context, builder, function, argument_pointers, argument_types,
                expected_type, argument,
            )?;
            values.push(self.materialize_value(context, builder, value)?);
        }

        let float_values = values.into_iter().map(|value| match value {
            BasicValueEnum::FloatValue(value) => Ok(value),
            _ => Err(format!("{:?} requires floating-point arguments", intrinsic)),
        }).collect::<Result<Vec<FloatValue<'ctx>>, String>>()?;

        let result_type = match expected_type {
            TypeInfo::F32 => context.f32_type(),
            TypeInfo::F64 => context.f64_type(),
            _ => return Err(format!("{:?} requires an f32 or f64 result type", intrinsic)),
        };

        if matches!(intrinsic, Intrinsic::Min | Intrinsic::Max) {
            let predicate = match intrinsic {
                Intrinsic::Min => inkwell::FloatPredicate::OLT,
                Intrinsic::Max => inkwell::FloatPredicate::OGT,
                _ => unreachable!(),
            };
            let compare = builder.build_float_compare(predicate, float_values[0], float_values[1], "intrinsic_cmp")
                .map_err(|error| format!("failed to build {:?} comparison: {:?}", intrinsic, error))?;
            let result = builder.build_select(compare, float_values[0], float_values[1], "intrinsic_minmax")
                .map_err(|error| format!("failed to build {:?}: {:?}", intrinsic, error))?;
            return Ok(LoweredValue::Value(result));
        }

        let intrinsic_name = match intrinsic {
            Intrinsic::Sqrt => "llvm.sqrt",
            Intrinsic::Abs => "llvm.fabs",
            Intrinsic::Floor => "llvm.floor",
            Intrinsic::Ceil => "llvm.ceil",
            Intrinsic::Round => "llvm.round",
            Intrinsic::Sin => "llvm.sin",
            Intrinsic::Cos => "llvm.cos",
            Intrinsic::Tan => "llvm.tan",
            Intrinsic::Exp => "llvm.exp",
            Intrinsic::Log => "llvm.log",
            Intrinsic::Pow => "llvm.pow",
            Intrinsic::Min | Intrinsic::Max => unreachable!(),
        };
        let suffix = if matches!(expected_type, TypeInfo::F32) { "f32" } else { "f64" };
        let symbol = format!("{}.{}", intrinsic_name, suffix);
        let function_type = result_type.fn_type(&vec![result_type.into(); float_values.len()], false);
        let intrinsic_function = self.module.get_function(&symbol).unwrap_or_else(|| self.module.add_function(&symbol, function_type, None));
        let args = float_values.iter().map(|value| (*value).into()).collect::<Vec<_>>();
        let call = builder.build_call(intrinsic_function, &args, "intrinsic")
            .map_err(|error| format!("failed to build {:?} intrinsic: {:?}", intrinsic, error))?;
        let result = call.try_as_basic_value().basic().ok_or_else(|| format!("{:?} intrinsic did not return a value", intrinsic))?;
        Ok(LoweredValue::Value(result))
    }
}
