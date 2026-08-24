use closure_ir::{call, compile_closure};

#[test]
fn test_abs_i32() {
    let compiled = compile_closure!(|x: i32| -> i32 {
        abs(x)
    });

    assert_eq!(call!(compiled, -7), 7);
    assert_eq!(call!(compiled, 0), 0);
    assert_eq!(call!(compiled, 7), 7);
}

#[test]
fn test_abs_f64() {
    let compiled = compile_closure!(|x: f64| -> f64 {
        abs(x)
    });

    assert_eq!(call!(compiled, -3.5), 3.5);
    assert_eq!(call!(compiled, 3.5), 3.5);
}

#[test]
fn test_min_i32() {
    let compiled = compile_closure!(|x: i32, y: i32| -> i32 {
        min(x, y)
    });

    assert_eq!(call!(compiled, 3, 7), 3);
    assert_eq!(call!(compiled, 7, 3), 3);
}

#[test]
fn test_max_i32() {
    let compiled = compile_closure!(|x: i32, y: i32| -> i32 {
        max(x, y)
    });

    assert_eq!(call!(compiled, 3, 7), 7);
    assert_eq!(call!(compiled, 7, 3), 7);
}

#[test]
fn test_min_max_nested() {
    let compiled = compile_closure!(|x: i32, y: i32, z: i32| -> i32 {
        max(min(x, y), z)
    });

    assert_eq!(call!(compiled, 10, 4, 6), 6);
    assert_eq!(call!(compiled, 2, 9, 7), 7);
}
