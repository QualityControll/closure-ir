use closure_ir::{call, compile_closure};

#[test] fn test_let_mut_assignment() { let compiled = compile_closure!(|x: i32| -> i32 { let mut y = x + 1; y = y * 2; y }); assert_eq!(call!(compiled, 4), 10); }
#[test] fn test_multiple_mutable_assignments() { let compiled = compile_closure!(|x: i32| -> i32 { let mut y = x; y = y + 5; y = y * 2; y = y - 3; y }); assert_eq!(call!(compiled, 4), 15); }
#[test] fn test_annotated_let_mut_assignment() { let compiled = compile_closure!(|x: i32| -> i32 { let mut y: i32 = x + 1; y = y + 10; y }); assert_eq!(call!(compiled, 4), 15); }
#[test] fn test_mutable_variable_can_be_used_by_other_locals() { let compiled = compile_closure!(|x: i32| -> i32 { let mut y = x + 1; let z = y * 2; y = z + 3; y }); assert_eq!(call!(compiled, 4), 13); }
#[test] fn test_let_binding() { let compiled = compile_closure!(|x: i32| -> i32 { let y = x + 1; y * 2 }); assert_eq!(call!(compiled, 4), 10); }
#[test] fn test_multiple_let_bindings() { let compiled = compile_closure!(|x: i32| -> i32 { let y = x + 1; let z = y * 2; z - 3 }); assert_eq!(call!(compiled, 4), 7); }
#[test] fn test_annotated_let_binding() { let compiled = compile_closure!(|x: i32| -> i32 { let y: i32 = x + 1; y * 2 }); assert_eq!(call!(compiled, 4), 10); }
#[test] fn test_let_binding_in_if_expression() { let compiled = compile_closure!(|x: i32| -> i32 { let y = x + 1; if y > 10 { y } else { 0 } }); assert_eq!(call!(compiled, 4), 0); assert_eq!(call!(compiled, 10), 11); }
#[test] fn test_let_binding_shadowing() { let compiled = compile_closure!(|x: i32| -> i32 { let y = x + 1; let z = y * 2; let y = x + 10; y + z }); assert_eq!(call!(compiled, 4), 24); }
