use closure_llvm::{call, compile_closure};

#[test]
fn test_for_loop_sum() {
    let compiled = compile_closure!(|| -> i32 {
        let mut sum = 0;
        for i in 0..5 {
            sum = sum + i;
        }
        sum
    });

    assert_eq!(call!(compiled), 10);
}

#[test]
fn test_for_loop_zero_iterations() {
    let compiled = compile_closure!(|x: i32| -> i32 {
        let mut sum = 7;
        for i in x..x {
            sum = sum + i;
        }
        sum
    });

    assert_eq!(call!(compiled, 3), 7);
}

#[test]
fn test_for_loop_uses_closure_argument() {
    let compiled = compile_closure!(|n: i32| -> i32 {
        let mut sum = 0;
        for i in 0..n {
            sum = sum + i;
        }
        sum
    });

    assert_eq!(call!(compiled, 0), 0);
    assert_eq!(call!(compiled, 1), 0);
    assert_eq!(call!(compiled, 5), 10);
}

#[test]
fn test_for_loop_inclusive_range() {
    let compiled = compile_closure!(|| -> i32 {
        let mut sum = 0;
        for i in 1..=5 {
            sum = sum + i;
        }
        sum
    });

    assert_eq!(call!(compiled), 15);
}

#[test]
fn test_for_loop_u32() {
    let compiled = compile_closure!(|| -> u32 {
        let mut sum = 0u32;
        for i in 0u32..4u32 {
            sum = sum + i;
        }
        sum
    });

    assert_eq!(call!(compiled), 6u32);
}

#[test]
fn test_for_loop_nested_body_mutation() {
    let compiled = compile_closure!(|n: i32| -> i32 {
        let mut value = 0;
        for i in 0..n {
            value = value + 1;
        }
        value
    });

    assert_eq!(call!(compiled, 6), 6);
}
