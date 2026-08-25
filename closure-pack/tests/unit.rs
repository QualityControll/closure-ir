use closure_pack::{call, compile_closure};

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
    let compiled = compile_closure!(|values: &mut [i32]| {
        values[0] = 42;
    });
    let mut values = [0];
    call!(compiled, &mut values[..]);
    assert_eq!(values[0], 42);
}

#[test]
fn test_unit_closure_with_multiple_statements() {
    let compiled = compile_closure!(|values: &mut [i32]| {
        let mut result: i32 = values[0];
        result = result + 10;
        values[0] = result;
    });
    let mut values = [5];
    call!(compiled, &mut values[..]);
    assert_eq!(values[0], 15);
}
