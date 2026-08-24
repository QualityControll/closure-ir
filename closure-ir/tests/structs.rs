use closure_ir::{call, closure_ir, compile_closure, CompileType};

#[repr(C)] #[derive(Debug, Clone, Copy, CompileType)] struct Point { x: f64, y: f64 }
#[repr(C)] #[derive(Debug, Clone, Copy, CompileType)] struct Rectangle { top_left: Point, bottom_right: Point }
#[repr(C)] #[derive(Debug, Clone, Copy, CompileType)] struct Nested { r: Rectangle, p: Point }
#[repr(C)] #[derive(Debug, Clone, Copy, CompileType, PartialEq)] struct Point2(bool, i32);

#[test] fn test_returned_tuple_struct() { let compiled = compile_closure!(|n: Point2| -> Point2 { n }); let tup = Point2 { 0: true, 1: 10 }; assert_eq!(call!(compiled, tup), Point2 { 0: true, 1: 10 }); }
#[test] fn test_returned_tuple() { let compiled = compile_closure!(|n: (f32, f32)| -> (f32, f32) { n }); let tup = (20.0, 10.0); assert_eq!(call!(compiled, tup), (20.0, 10.0)); }
#[test] fn test_tuple_value() { let compiled = compile_closure!(|x: i32, y: i32| -> (i32, i32) { (x, y) }); assert_eq!(call!(compiled, 40, 10), (40, 10)); }
#[test] fn test_nested_struct() { let compiled = compile_closure!(|n: Nested| -> f64 { n.r.bottom_right.x - n.p.x }); let nested = Nested { r: Rectangle { top_left: Point { x: 10.0, y: 20.0 }, bottom_right: Point { x: 30.0, y: 40.0 } }, p: Point { x: 10.0, y: 5.0 } }; assert_eq!(call!(compiled, nested), 20.0); }
#[test] fn test_struct_field_arithmetic() { let compiled = compile_closure!(|r: Rectangle| -> f64 { r.bottom_right.x - r.top_left.x }); let rectangle = Rectangle { top_left: Point { x: 10.0, y: 20.0 }, bottom_right: Point { x: 30.0, y: 40.0 } }; assert_eq!(call!(compiled, rectangle), 20.0); }
#[test] fn dynamic_invocation_add_i32() { let closure = closure_ir!(|x: i32, y: i32| -> i32 { x + y }); let context = inkwell::context::Context::create(); let compiler = closure_ir::Compiler::new(&context); let compiled = compiler.compile_dynamic(&closure).expect("failed to compile closure"); let result = unsafe { compiled.call(&[closure_ir::Value::I32(10), closure_ir::Value::I32(20)]).expect("failed to invoke closure") }; assert_eq!(result, closure_ir::Value::I32(30)); }
#[test] fn test_serialization() { let expr = closure_ir!(|x: i32| -> i32 { x }); let serialized = serde_json::to_string(&expr).unwrap(); println!("serialized expr is {}", serialized); let deserialized = serde_json::from_str(&serialized).unwrap(); assert_eq!(expr, deserialized); }
#[test] fn print_quoted_expr() { let expr = closure_ir!(|x: i32| -> i32 { x }); println!("expr is {:?}", expr); }
