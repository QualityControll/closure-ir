use closure_pack::compile_closure;

#[test]
fn captures_f64_value() {
    let scale = 2.5_f64;
    let compiled = compile_closure!(|x: f64| -> f64 { x * scale });
    let mut args = (4.0_f64,);
    assert_eq!(unsafe { compiled.call(&mut args) }, 10.0);
}

#[test]
fn captures_multiple_values() {
    let scale = 2_i32;
    let offset = 3_i32;
    let compiled = compile_closure!(|x: i32| -> i32 { x * scale + offset });
    let mut args = (4_i32,);
    assert_eq!(unsafe { compiled.call(&mut args) }, 11);
}

#[test]
fn capture_is_stored_in_compiled_environment() {
    let scale = 3_i32;
    let compiled = compile_closure!(|x: i32| -> i32 { x * scale });
    let mut first = (2_i32,);
    let mut second = (7_i32,);
    assert_eq!(unsafe { compiled.call(&mut first) }, 6);
    assert_eq!(unsafe { compiled.call(&mut second) }, 21);
}

#[test]
fn f64_capture_type_is_not_inferred_from_unrelated_local() {
    let pi = std::f64::consts::PI;
    let compiled = compile_closure!(|x: f64| -> f64 {
        let n: usize = 2;
        let angle = -2.0 * pi / (n as f64);
        cos(angle) + x * 0.0
    });
    let mut args = (0.0_f64,);
    let result = unsafe { compiled.call(&mut args) };
    assert!((result + 1.0).abs() < 1e-12);
}
