use closure_ir::{call, compile_closure};

#[test] fn test_i8_to_i32() { let compiled = compile_closure!(|a: i8| -> i32 { a as i32 }); assert_eq!(call!(compiled, -7i8), -7); }
#[test] fn test_u8_to_i64() { let compiled = compile_closure!(|a: u8| -> i64 { a as i64 }); assert_eq!(call!(compiled, 250u8), 250); }
#[test] fn test_i64_to_i8() { let compiled = compile_closure!(|a: i64| -> i8 { a as i8 }); assert_eq!(call!(compiled, 257i64), 1i8); }
#[test] fn test_i32_to_f64() { let compiled = compile_closure!(|a: i32| -> f64 { a as f64 }); assert_eq!(call!(compiled, -12), -12.0); }
#[test] fn test_u32_to_f32() { let compiled = compile_closure!(|a: u32| -> f32 { a as f32 }); assert_eq!(call!(compiled, 125u32), 125.0); }
#[test] fn test_f64_to_i32() { let compiled = compile_closure!(|a: f64| -> i32 { a as i32 }); assert_eq!(call!(compiled, -12.75), -12); }
#[test] fn test_f32_to_f64() { let compiled = compile_closure!(|a: f32| -> f64 { a as f64 }); assert_eq!(call!(compiled, 3.5f32), 3.5); }
#[test] fn test_f64_to_f32() { let compiled = compile_closure!(|a: f64| -> f32 { a as f32 }); assert_eq!(call!(compiled, 3.5), 3.5f32); }
#[test] fn test_usize_to_i64() { let compiled = compile_closure!(|a: usize| -> i64 { a as i64 }); assert_eq!(call!(compiled, 42usize), 42); }
