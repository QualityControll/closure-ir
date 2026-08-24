use melior::{
    dialect::{arith, func},
    ir::{
        attribute::{FloatAttribute, IntegerAttribute, StringAttribute, TypeAttribute},
        operation::OperationLike,
        r#type::FunctionType,
        Block, BlockLike, Location, Module, Region, RegionLike, Type, Value,
    },
    utility::register_all_dialects,
    Context,
};

use crate::{
    expr::{Block as ClosureBlock, Closure, Expr},
    mlir_types::mlir_type,
    value::Value as ClosureValue,
};

pub(crate) struct MlirLowerer<'c> {
    context: &'c Context,
    location: Location<'c>,
}

impl<'c> MlirLowerer<'c> {
    pub(crate) fn new(context: &'c Context) -> Self {
        Self { context, location: Location::unknown(context) }
    }

    pub(crate) fn lower_simple_closure(&self, closure: &Closure) -> Result<Module<'c>, String> {
        if !closure.body.statements.is_empty() {
            return Err("MLIR simple closure lowering does not support statements yet".into());
        }
        let return_expr = closure.body.result.as_ref()
            .ok_or_else(|| "MLIR simple closure lowering requires a return expression".to_string())?;
        let argument_types = closure.arguments.iter()
            .map(|ty| mlir_type(self.context, ty))
            .collect::<Result<Vec<_>, _>>()?;
        let return_type = mlir_type(self.context, &closure.return_type)?;

        let module = Module::new(self.location);
        let region = Region::new();
        let block = Block::new(&argument_types.iter().map(|ty| (*ty, self.location)).collect::<Vec<_>>());
        let result = self.lower_expr(&block, &closure.body, return_expr)?;
        block.append_operation(func::r#return(&[result], self.location));
        region.append_block(block);
        let function_type = FunctionType::new(self.context, &argument_types, &[return_type]);
        let function = func::func(
            self.context,
            StringAttribute::new(self.context, "compiled_closure"),
            TypeAttribute::new(function_type.into()),
            region,
            &[],
            self.location,
        );
        module.body().append_operation(function);
        if !module.as_operation().verify() {
            return Err("generated MLIR function failed verification".into());
        }
        Ok(module)
    }

    fn lower_expr<'a>(&self, block: &'a Block<'c>, _closure_block: &ClosureBlock, expr: &Expr) -> Result<Value<'c, 'a>, String> {
        match expr {
            Expr::Argument(index) => block.argument(*index)
                .map(|argument| argument.into())
                .map_err(|error| format!("invalid closure argument {index}: {error:?}")),
            Expr::Constant(value) => self.lower_constant(block, value),
            other => Err(format!("MLIR simple expression lowering does not support {other:?}")),
        }
    }

    fn lower_constant<'a>(&self, block: &'a Block<'c>, value: &ClosureValue) -> Result<Value<'c, 'a>, String> {
        let operation = match value {
            ClosureValue::F32(value) => arith::constant(self.context, FloatAttribute::new(self.context, Type::float32(self.context), *value as f64).into(), self.location),
            ClosureValue::F64(value) => arith::constant(self.context, FloatAttribute::new(self.context, Type::float64(self.context), *value).into(), self.location),
            ClosureValue::I8(value) => self.integer_constant(*value as i64, 8)?,
            ClosureValue::I16(value) => self.integer_constant(*value as i64, 16)?,
            ClosureValue::I32(value) => self.integer_constant(*value as i64, 32)?,
            ClosureValue::I64(value) => self.integer_constant(*value, 64)?,
            ClosureValue::U8(value) => self.integer_constant(*value as i64, 8)?,
            ClosureValue::U16(value) => self.integer_constant(*value as i64, 16)?,
            ClosureValue::U32(value) => self.integer_constant(*value as i64, 32)?,
            ClosureValue::U64(value) => self.integer_constant(*value as i64, 64)?,
            ClosureValue::Usize(value) => self.integer_constant(*value as i64, 64)?,
            ClosureValue::Bool(value) => self.integer_constant(i64::from(*value), 1)?,
            ClosureValue::I128(_) | ClosureValue::U128(_) | ClosureValue::Array(_) => {
                return Err(format!("MLIR constant lowering does not yet support {value:?}"));
            }
        };
        Ok(block.append_operation(operation).result(0).unwrap().into())
    }

    fn integer_constant(&self, value: i64, bits: u32) -> Result<melior::ir::Operation<'c>, String> {
        let ty = Type::parse(self.context, &format!("i{bits}"))
            .ok_or_else(|| format!("failed to create MLIR integer type i{bits}"))?;
        Ok(arith::constant(
            self.context,
            IntegerAttribute::new(ty, value).into(),
            self.location,
        ))
    }
}

pub(crate) fn lower_simple_closure<'c>(context: &'c Context, closure: &Closure) -> Result<Module<'c>, String> {
    register_all_dialects(context);
    MlirLowerer::new(context).lower_simple_closure(closure)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expr::Block, types::TypeInfo, value::Value};

    #[test]
    fn lowers_argument_to_mlir_function() {
        let context = Context::new();
        let closure = Closure { arguments: vec![TypeInfo::I32], return_type: TypeInfo::I32, body: Block::expression(Expr::Argument(0)) };
        let module = lower_simple_closure(&context, &closure).unwrap();
        let text = module.as_operation().to_string();
        assert!(text.contains("func.func"));
        assert!(text.contains("i32"));
        assert!(text.contains("func.return"));
    }

    #[test]
    fn lowers_integer_constant_to_mlir() {
        let context = Context::new();
        let closure = Closure { arguments: vec![], return_type: TypeInfo::I32, body: Block::expression(Expr::Constant(Value::I32(42))) };
        let module = lower_simple_closure(&context, &closure).unwrap();
        let text = module.as_operation().to_string();
        assert!(text.contains("arith.constant"));
        assert!(text.contains("42"));
        assert!(text.contains("i32"));
    }

    #[test]
    fn lowers_float_constant_to_mlir() {
        let context = Context::new();
        let closure = Closure { arguments: vec![], return_type: TypeInfo::F64, body: Block::expression(Expr::Constant(Value::F64(3.5))) };
        let module = lower_simple_closure(&context, &closure).unwrap();
        let text = module.as_operation().to_string();
        assert!(text.contains("arith.constant"));
        assert!(text.contains("f64"));
    }
}
