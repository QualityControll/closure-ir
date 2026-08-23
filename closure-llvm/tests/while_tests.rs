use closure_llvm::{call, compile_closure};

#[test]
fn test_while_loop_mutates_local() {
    let compiled = compile_closure!(|x: i32| -> i32 {
        let mut value = x;
        while value < 10 {
            value = value + 1;
        }
        value
    });

    assert_eq!(call!(compiled, 0), 10);
    assert_eq!(call!(compiled, 7), 10);
    assert_eq!(call!(compiled, 10), 10);
}

#[test]
fn test_while_loop_zero_iterations() {
    let compiled = compile_closure!(|x: i32| -> i32 {
        let mut value = x;
        while value < 0 {
            value = value + 1;
        }
        value
    });

    assert_eq!(call!(compiled, 5), 5);
}

#[test]
fn test_while_loop_multiple_iterations() {
    let compiled = compile_closure!(|x: i32| -> i32 {
        let mut value = 0;
        while value < x {
            value = value + 2;
        }
        value
    });

    assert_eq!(call!(compiled, 0), 0);
    assert_eq!(call!(compiled, 5), 6);
    assert_eq!(call!(compiled, 10), 10);
}
