use closure_ir::{call, compile_closure};

#[test] fn test_i32_addition() { let compiled = compile_closure!(|a: i32| -> i32 { a + 5 }); assert_eq!(call!(compiled, 2), 7); }
#[test] fn test_i32_arithmetic() { let compiled = compile_closure!(|a: i32| -> i32 { ((a + 5) * 2) - 3 }); assert_eq!(call!(compiled, 2), 11); }
#[test] fn test_i32_division() { let compiled = compile_closure!(|a: i32| -> i32 { a / 2 }); assert_eq!(call!(compiled, 10), 5); }
#[test] fn test_i32_remainder() { let compiled = compile_closure!(|a: i32| -> i32 { a % 3 }); assert_eq!(call!(compiled, 10), 1); }
#[test] fn test_i8() { let compiled = compile_closure!(|a: i8| -> i8 { a + 5 }); assert_eq!(call!(compiled, 2i8), 7i8); }
#[test] fn test_i16() { let compiled = compile_closure!(|a: i16| -> i16 { a * 3 }); assert_eq!(call!(compiled, 4i16), 12i16); }
#[test] fn test_i64() { let compiled = compile_closure!(|a: i64| -> i64 { a - 10 }); assert_eq!(call!(compiled, 25i64), 15i64); }
#[test] fn test_i128() { let compiled = compile_closure!(|a: i128| -> i128 { a + 100i128 }); assert_eq!(call!(compiled, 25i128), 125i128); }
#[test] fn test_u8() { let compiled = compile_closure!(|a: u8| -> u8 { a + 5 }); assert_eq!(call!(compiled, 2u8), 7u8); }
#[test] fn test_u16() { let compiled = compile_closure!(|a: u16| -> u16 { a * 3 }); assert_eq!(call!(compiled, 4u16), 12u16); }
#[test] fn test_u32() { let compiled = compile_closure!(|a: u32| -> u32 { a + 5 }); assert_eq!(call!(compiled, 2u32), 7u32); }
#[test] fn test_u64() { let compiled = compile_closure!(|a: u64| -> u64 { a * 2 }); assert_eq!(call!(compiled, 21u64), 42u64); }
#[test] fn test_u128() { let compiled = compile_closure!(|a: u128| -> u128 { a + 100u128 }); assert_eq!(call!(compiled, 25u128), 125u128); }
#[test] fn test_f32_addition() { let compiled = compile_closure!(|a: f32| -> f32 { a + 5.0f32 }); assert_eq!(call!(compiled, 2.0f32), 7.0f32); }
#[test] fn test_f32_arithmetic() { let compiled = compile_closure!(|a: f32| -> f32 { (a * 2.0f32) - 1.0f32 }); assert_eq!(call!(compiled, 4.0f32), 7.0f32); }
#[test] fn test_f64_addition() { let compiled = compile_closure!(|a: f64| -> f64 { a + 5.0 }); assert_eq!(call!(compiled, 2.0), 7.0); }
#[test] fn test_f64_remainder() { let compiled = compile_closure!(|a: f64| -> f64 { a % 3.0 }); assert_eq!(call!(compiled, 10.0), 1.0); }
#[test] fn test_boolean_not() { let compiled = compile_closure!(|b: bool| -> bool { !b }); assert!(!call!(compiled, true)); assert!(call!(compiled, false)); }
#[test] fn test_integer_negation() { let compiled = compile_closure!(|a: i32| -> i32 { -a }); assert_eq!(call!(compiled, 5), -5); }
#[test] fn test_float_negation() { let compiled = compile_closure!(|a: f64| -> f64 { -a }); assert_eq!(call!(compiled, 5.5), -5.5); }
#[test] fn test_i32_equal() { let compiled = compile_closure!(|a: i32| -> bool { a == 5 }); assert!(call!(compiled, 5)); assert!(!call!(compiled, 4)); }
#[test] fn test_i32_less_than() { let compiled = compile_closure!(|a: i32| -> bool { a < 5 }); assert!(call!(compiled, 4)); assert!(!call!(compiled, 5)); }
#[test] fn test_i32_greater_than() { let compiled = compile_closure!(|a: i32| -> bool { a > 5 }); assert!(call!(compiled, 6)); assert!(!call!(compiled, 5)); }
#[test] fn test_u32_comparison() { let compiled = compile_closure!(|a: u32| -> bool { a < 10u32 }); assert!(call!(compiled, 5u32)); assert!(!call!(compiled, 10u32)); }
#[test] fn test_f64_comparison() { let compiled = compile_closure!(|a: f64| -> bool { a >= 5.0 }); assert!(call!(compiled, 5.0)); assert!(call!(compiled, 6.0)); assert!(!call!(compiled, 4.0)); }
#[test] fn test_boolean_and() { let compiled = compile_closure!(|a: bool| -> bool { a && true }); assert!(call!(compiled, true)); assert!(!call!(compiled, false)); }
#[test] fn test_boolean_or() { let compiled = compile_closure!(|a: bool| -> bool { a || false }); assert!(call!(compiled, true)); assert!(!call!(compiled, false)); }
#[test] fn test_bitwise_and() { let compiled = compile_closure!(|a: u32| -> u32 { a & 0xffu32 }); assert_eq!(call!(compiled, 0x1234u32), 0x34u32); }
#[test] fn test_bitwise_or() { let compiled = compile_closure!(|a: u32| -> u32 { a | 0x10u32 }); assert_eq!(call!(compiled, 0x20u32), 0x30u32); }
#[test] fn test_bitwise_xor() { let compiled = compile_closure!(|a: u32| -> u32 { a ^ 0xffu32 }); assert_eq!(call!(compiled, 0xf0u32), 0x0fu32); }
#[test] fn test_shift_left() { let compiled = compile_closure!(|a: u32| -> u32 { a << 2 }); assert_eq!(call!(compiled, 3u32), 12u32); }
#[test] fn test_shift_right() { let compiled = compile_closure!(|a: u32| -> u32 { a >> 2 }); assert_eq!(call!(compiled, 12u32), 3u32); }
