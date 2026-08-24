use closure_ir::{call, compile_closure};

#[test]
fn test_slice_indexing_i32() {
    let compiled = compile_closure!(|values: &[i32]| -> i32 {
        values[0] + values[1] + values[2]
    });
    let values: &[i32] = &[1, 2, 3];
    assert_eq!(call!(compiled, values), 6);
}

#[test]
fn test_slice_indexing_f64() {
    let compiled = compile_closure!(|values: &[f64]| -> f64 {
        values[0] * values[2]
    });
    let values: &[f64] = &[2.0, 4.0, 5.0];
    assert_eq!(call!(compiled, values), 10.0);
}

#[test]
fn test_nested_slice_indexing() {
    let compiled = compile_closure!(|values: &[[i32; 2]]| -> i32 {
        values[0][1] + values[1][0]
    });
    let values: &[[i32; 2]] = &[[1, 2], [3, 4]];
    assert_eq!(call!(compiled, values), 5);
}

#[test]
fn test_empty_slice_is_valid_type() {
    let compiled = compile_closure!(|values: &[i32]| -> i32 {
        values[0]
    });
    let values: &[i32] = &[42];
    assert_eq!(call!(compiled, values), 42);
}

#[test]
fn test_slice_last_index_is_valid() {
    let compiled = compile_closure!(|values: &[i32]| -> i32 {
        values[2]
    });
    let values: &[i32] = &[10, 20, 30];
    assert_eq!(call!(compiled, values), 30);
}

#[test]
fn test_single_element_slice_index_zero_is_valid() {
    let compiled = compile_closure!(|values: &[i32]| -> i32 {
        values[0]
    });
    let values: &[i32] = &[42];
    assert_eq!(call!(compiled, values), 42);
}
