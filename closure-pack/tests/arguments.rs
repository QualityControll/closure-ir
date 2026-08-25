use closure_pack::{call, compile_closure};

#[test] fn test_four_arguments_add() { let compiled = compile_closure!(|w: i32, x: i32, y: i32, z: i32| -> i32 { w + x + y + z }); assert_eq!(call!(compiled, 2, 3, 4, 5), 14); }
#[test] fn test_two_arguments_add() { let compiled = compile_closure!(|x: i32, y: i32| -> i32 { x + y }); assert_eq!(call!(compiled, 2, 3), 5); }
#[test] fn test_two_arguments_subtract() { let compiled = compile_closure!(|x: i32, y: i32| -> i32 { x - y }); assert_eq!(call!(compiled, 10, 3), 7); }
#[test] fn test_two_arguments_comparison() { let compiled = compile_closure!(|x: i32, y: i32| -> bool { x > y }); assert!(call!(compiled, 10, 5)); assert!(!call!(compiled, 5, 10)); }
#[test] fn test_two_arguments_if_else() { let compiled = compile_closure!(|x: i32, y: i32| -> i32 { if x > y { x } else { y } }); assert_eq!(call!(compiled, 10, 5), 10); assert_eq!(call!(compiled, 5, 10), 10); }
#[test] fn test_zero_arguments() { let compiled = compile_closure!(|| -> i32 { 100 }); assert_eq!(call!(compiled), 100); }
