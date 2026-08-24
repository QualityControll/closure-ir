use melior::{
    dialect::{arith, func, DialectRegistry},
    ir::{
        attribute::{StringAttribute, TypeAttribute},
        operation::OperationLike,
        r#type::{FunctionType, TypeLike},
        Block, BlockLike, Location, Module, Region, RegionLike,
    },
    pass::PassManager,
    Context,
    ExecutionEngine,
};

/// Build the MLIR equivalent of `|x: i32| x + x`.
///
/// This deliberately does not depend on closure-ir's existing Expr/Statement IR.
/// The point of the POC is to prove that the Rust frontend can construct MLIR
/// directly and that standard MLIR lowering can take it to LLVM and native code.
pub fn build_add_module(context: &Context) -> Module {
    let location = Location::unknown(context);
    let module = Module::new(location);
    let i32_type = melior::ir::Type::parse(context, "i32").expect("valid i32 type");

    let block = Block::new(&[(i32_type, location)]);
    let arg = block.argument(0).unwrap().into();
    let sum = block.append_operation(arith::addi(arg, arg, location));
    block.append_operation(func::r#return(&[sum.result(0).unwrap().into()], location));

    let region = Region::new();
    region.append_block(block);

    let function = func::func(
        context,
        StringAttribute::new(context, "closure_add"),
        TypeAttribute::new(FunctionType::new(context, &[i32_type], &[i32_type]).into()),
        region,
        &[],
        location,
    );

    module.body().append_operation(function);
    module
}

/// Demonstrates the complete backend half of the proposed architecture:
/// Rust -> MLIR -> LLVM dialect -> native JIT execution.
pub fn execute_add(x: i32) -> Result<i32, String> {
    let registry = DialectRegistry::new();
    melior::utility::register_all_dialects(&registry);

    let context = Context::new();
    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();

    let mut module = build_add_module(&context);
    if !module.as_operation().verify() {
        return Err("generated MLIR failed verification".into());
    }

    let pass_manager = PassManager::new(&context);
    pass_manager.add_pass(melior::pass::conversion::create_to_llvm());
    pass_manager
        .run(&mut module)
        .map_err(|_| "MLIR to LLVM lowering failed".to_string())?;

    let engine = ExecutionEngine::new(&module, 2, &[], false, false);
    let function = engine.lookup("closure_add");
    if function.is_null() {
        return Err("JIT could not find closure_add".into());
    }

    let function: unsafe extern "C" fn(i32) -> i32 = unsafe { std::mem::transmute(function) };
    Ok(unsafe { function(x) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_valid_mlir() {
        let registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&registry);
        let context = Context::new();
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();

        let module = build_add_module(&context);
        assert!(module.as_operation().verify());
    }

    #[test]
    fn lowers_mlir_to_llvm_and_executes() {
        assert_eq!(execute_add(21).unwrap(), 42);
    }
}
