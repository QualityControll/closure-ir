use closure_pack::{Block, Closure, Compiler, Expr, ExternalFunction, TypeInfo};

#[test]
fn external_function_call_compiles() {
    let closure = Closure {
        captures: vec![],
        arguments: vec![TypeInfo::F64],
        return_type: TypeInfo::F64,
        body: Block::expression(Expr::ExternalCall {
            function: "sqrt".into(),
            arguments: vec![Expr::Argument(0)],
            return_type: TypeInfo::F64,
        }),
        external_functions: vec![ExternalFunction::new(
            "sqrt",
            vec![TypeInfo::F64],
            TypeInfo::F64,
        )],
    };

    let context = Box::leak(Box::new(closure_pack::melior::Context::new()));
    let compiler = Compiler::new(context);
    let _compiled = compiler
        .compile::<(f64,), f64>(&closure)
        .expect("external function closure should compile");
}

#[test]
fn undeclared_external_function_is_rejected() {
    let closure = Closure {
        captures: vec![],
        arguments: vec![TypeInfo::F64],
        return_type: TypeInfo::F64,
        body: Block::expression(Expr::ExternalCall {
            function: "not_declared".into(),
            arguments: vec![Expr::Argument(0)],
            return_type: TypeInfo::F64,
        }),
        external_functions: vec![],
    };

    let context = Box::leak(Box::new(closure_pack::melior::Context::new()));
    let compiler = Compiler::new(context);
    let error = compiler.compile::<(f64,), f64>(&closure).unwrap_err();
    assert!(error.contains("not_declared"));
}
