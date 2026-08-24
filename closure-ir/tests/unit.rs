use closure_ir::{call, compile_closure};

#[test]
fn test_closure_without_explicit_return_type() {
    let compiled = compile_closure!(|| {});
    call!(compiled);
}

#[test]
fn test_explicit_unit_return_type() {
    let compiled = compile_closure!(|| -> () {});
    call!(compiled);
}

#[test]
fn test_unit_closure_with_argument() {
    let compiled = compile_closure!(|value: &mut i32| {
        *value = 42;
    });
    let mut value = 0;
    call!(compiled, &mut value);
    assert_eq!(value, 42);
}

#[test]
fn test_unit_closure_with_multiple_statements() {
    let compiled = compile_closure!(|value: &mut i32| {
        let mut result: i32 = *value;
        result = result + 10;
        *value = result;
    });
    let mut value = 5;
    call!(compiled, &mut value);
    assert_eq!(value, 15);
}
