use closure_ir::{call, compile_closure};

#[test]
fn test_mutate_array_element() {
    let compiled = compile_closure!(|mut values: [i32; 4]| -> i32 {
        values[2] = 42;
        values[2]
    });
    assert_eq!(call!(compiled, [1, 2, 3, 4]), 42);
}

#[test]
fn test_mutate_nested_array_element() {
    let compiled = compile_closure!(|mut values: [[i32; 2]; 2]| -> i32 {
        values[1][0] = 99;
        values[1][0]
    });
    assert_eq!(call!(compiled, [[1, 2], [3, 4]]), 99);
}

#[test]
fn test_mutate_array_with_runtime_index() {
    let compiled = compile_closure!(|mut values: [i32; 4], index: usize| -> i32 {
        values[index] = 77;
        values[index]
    });
    assert_eq!(call!(compiled, ([10, 20, 30, 40], 2usize)), 77);
}

#[test]
fn test_mutate_slice_element() {
    let compiled = compile_closure!(|values: &mut [i32]| -> i32 {
        values[1] = 55;
        values[1]
    });
    let mut values = [10, 20, 30];
    assert_eq!(call!(compiled, &mut values[..]), 55);
}
