use closure_ir::{call, compile_closure};

#[test]
fn test_sqrt() {
    let compiled = compile_closure!(|x: f64| -> f64 { sqrt(x) });
    assert!((call!(compiled, 9.0) - 3.0).abs() < 1e-12);
}

#[test]
fn test_abs() {
    let compiled = compile_closure!(|x: f64| -> f64 { abs(x) });
    assert_eq!(call!(compiled, -3.5), 3.5);
    assert_eq!(call!(compiled, 3.5), 3.5);
}

#[test]
fn test_min_and_max() {
    let min = compile_closure!(|x: f64, y: f64| -> f64 { min(x, y) });
    let max = compile_closure!(|x: f64, y: f64| -> f64 { max(x, y) });
    assert_eq!(call!(min, 2.0, 5.0), 2.0);
    assert_eq!(call!(max, 2.0, 5.0), 5.0);
}

#[test]
fn test_floor_ceil_round() {
    let floor = compile_closure!(|x: f64| -> f64 { floor(x) });
    let ceil = compile_closure!(|x: f64| -> f64 { ceil(x) });
    let round = compile_closure!(|x: f64| -> f64 { round(x) });
    assert_eq!(call!(floor, 2.7), 2.0);
    assert_eq!(call!(ceil, 2.2), 3.0);
    assert_eq!(call!(round, 2.5), 3.0);
}

#[test]
fn test_sin_cos_tan() {
    let sin = compile_closure!(|x: f64| -> f64 { sin(x) });
    let cos = compile_closure!(|x: f64| -> f64 { cos(x) });
    let tan = compile_closure!(|x: f64| -> f64 { tan(x) });
    assert!((call!(sin, 0.0)).abs() < 1e-12);
    assert!((call!(cos, 0.0) - 1.0).abs() < 1e-12);
    assert!((call!(tan, 0.0)).abs() < 1e-12);
}

#[test]
fn test_exp_and_log() {
    let exp = compile_closure!(|x: f64| -> f64 { exp(x) });
    let log = compile_closure!(|x: f64| -> f64 { log(x) });
    let e = 2.718281828459045_f64;
    assert!((call!(exp, 1.0) - e).abs() < 1e-12);
    assert!((call!(log, e) - 1.0).abs() < 1e-12);
}

#[test]
fn test_pow() {
    let compiled = compile_closure!(|x: f64| -> f64 { pow(x, 2.0) });
    assert!((call!(compiled, 3.0) - 9.0).abs() < 1e-12);
}

#[test]
fn test_f32_intrinsics() {
    let compiled = compile_closure!(|x: f32| -> f32 { sqrt(x) + abs(x) });
    let value = call!(compiled, -4.0_f32);
    assert!((value - 6.0).abs() < 1e-6);
}

#[test]
fn test_nested_intrinsics() {
    let compiled = compile_closure!(|x: f64| -> f64 { pow(sqrt(abs(x)), 2.0) });
    assert!((call!(compiled, -16.0) - 16.0).abs() < 1e-12);
}
