use closure_llvm::{call, compile_closure, quote_closure, CompileType};

#[cfg(test)]
mod tests {

    use super::*;

    // ============================================================
    // Let bindings
    // ============================================================
    #[test]
    fn test_let_binding() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            let y = x + 1;
            y * 2
        });

        assert_eq!(call!(compiled, 4), 10);
    }

    #[test]
    fn test_multiple_let_bindings() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            let y = x + 1;
            let z = y * 2;
            z - 3
        });

        assert_eq!(call!(compiled, 4), 7);
    }

    #[test]
    fn test_annotated_let_binding() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            let y: i32 = x + 1;
            y * 2
        });

        assert_eq!(call!(compiled, 4), 10);
    }

    #[test]
    fn test_let_binding_in_if_expression() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            let y = x + 1;

            if y > 10 {
                y
            } else {
                0
            }
        });

        assert_eq!(call!(compiled, 4), 0);
        assert_eq!(call!(compiled, 10), 11);
    }

    #[test]
    fn test_let_binding_shadowing() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            let y = x + 1;
            let z = y * 2;
            let y = x + 10;
            y + z
        });

        assert_eq!(call!(compiled, 4), 24);
    }

    // ============================================================
    // Multiple arguments
    // ============================================================
    #[test]
    fn test_four_arguments_add() {
        let compiled = compile_closure!(|w: i32, x: i32, y: i32, z: i32| -> i32 { w + x + y + z });

        assert_eq!(call!(compiled, 2, 3, 4, 5), 14);
    }

    #[test]
    fn test_two_arguments_add() {
        let compiled = compile_closure!(|x: i32, y: i32| -> i32 { x + y });

        assert_eq!(call!(compiled, 2, 3), 5);
    }

    #[test]
    fn test_two_arguments_subtract() {
        let compiled = compile_closure!(|x: i32, y: i32| -> i32 { x - y });

        assert_eq!(call!(compiled, 10, 3), 7);
    }

    #[test]
    fn test_two_arguments_comparison() {
        let compiled = compile_closure!(|x: i32, y: i32| -> bool { x > y });

        assert!(call!(compiled, 10, 5));
        assert!(!call!(compiled, 5, 10));
    }

    #[test]
    fn test_two_arguments_if_else() {
        let compiled = compile_closure!(|x: i32, y: i32| -> i32 {
            if x > y {
                x
            } else {
                y
            }
        });

        assert_eq!(call!(compiled, 10, 5), 10);
        assert_eq!(call!(compiled, 5, 10), 10);
    }

    #[test]
    fn test_zero_arguments() {
        let compiled = compile_closure!(|| -> i32 { 100 });

        assert_eq!(call!(compiled), 100);
    }

    // ============================================================
    // Compound expressions
    // ============================================================

    #[test]
    fn test_compound_comparison() {
        let compiled = compile_closure!(|x: i32| -> bool { x > 10 && x < 20 });

        assert!(!call!(compiled, 5));
        assert!(call!(compiled, 15));
        assert!(!call!(compiled, 25));
    }

    #[test]
    fn test_compound_if_condition() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x >= 10 && x <= 20 {
                1
            } else {
                0
            }
        });

        assert_eq!(call!(compiled, 5), 0);
        assert_eq!(call!(compiled, 10), 1);
        assert_eq!(call!(compiled, 15), 1);
        assert_eq!(call!(compiled, 20), 1);
        assert_eq!(call!(compiled, 25), 0);
    }

    // ============================================================
    // Basic if/else
    // ============================================================

    #[test]
    fn test_if_else_true() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x > 10 {
                100
            } else {
                200
            }
        });

        assert_eq!(call!(compiled, 20), 100);
    }

    // ============================================================
    // Basic if/else false branch
    // ============================================================

    #[test]
    fn test_if_else_false() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x > 10 {
                100
            } else {
                200
            }
        });

        assert_eq!(call!(compiled, 5), 200);
    }

    // ============================================================
    // If/else using the argument in both branches
    // ============================================================

    #[test]
    fn test_if_else_argument() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x > 0 {
                x + 10
            } else {
                x - 10
            }
        });

        assert_eq!(call!(compiled, 5), 15);

        assert_eq!(call!(compiled, -5), -15);
    }

    // ============================================================
    // Equality condition
    // ============================================================

    #[test]
    fn test_if_else_equal() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x == 10 {
                1
            } else {
                0
            }
        });

        assert_eq!(call!(compiled, 10), 1);

        assert_eq!(call!(compiled, 11), 0);
    }

    // ============================================================
    // Less-than condition
    // ============================================================

    #[test]
    fn test_if_else_less_than() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x < 10 {
                1
            } else {
                2
            }
        });

        assert_eq!(call!(compiled, 5), 1);

        assert_eq!(call!(compiled, 15), 2);
    }

    // ============================================================
    // Nested else-if
    // ============================================================

    #[test]
    fn test_else_if() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x > 10 {
                100
            } else if x > 5 {
                50
            } else {
                0
            }
        });

        assert_eq!(call!(compiled, 20), 100);

        assert_eq!(call!(compiled, 7), 50);

        assert_eq!(call!(compiled, 2), 0);
    }

    // ============================================================
    // Nested if/else
    // ============================================================

    #[test]
    fn test_nested_if_else() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x > 0 {
                if x > 10 {
                    100
                } else {
                    50
                }
            } else {
                0
            }
        });

        assert_eq!(call!(compiled, 20), 100);

        assert_eq!(call!(compiled, 5), 50);

        assert_eq!(call!(compiled, -5), 0);
    }

    // ============================================================
    // Unsigned integer if/else
    // ============================================================

    #[test]
    fn test_if_else_u32() {
        let compiled = compile_closure!(|x: u32| -> u32 {
            if x > 10 {
                100
            } else {
                200
            }
        });

        assert_eq!(call!(compiled, 20), 100);

        assert_eq!(call!(compiled, 5), 200);
    }

    // ============================================================
    // Boolean condition
    // ============================================================

    #[test]
    fn test_if_else_bool() {
        let compiled = compile_closure!(|x: bool| -> i32 {
            if x {
                1
            } else {
                0
            }
        });

        assert_eq!(call!(compiled, true), 1);

        assert_eq!(call!(compiled, false), 0);
    }

    // ============================================================
    // Floating-point condition
    // ============================================================

    #[test]
    fn test_if_else_f32() {
        let compiled = compile_closure!(|x: f32| -> f32 {
            if x > 10.0 {
                x + 1.0
            } else {
                x - 1.0
            }
        });

        assert_eq!(call!(compiled, 20.0), 21.0);

        assert_eq!(call!(compiled, 5.0), 4.0);
    }

    // ============================================================
    // Floating-point equality
    // ============================================================

    #[test]
    fn test_if_else_f64_equal() {
        let compiled = compile_closure!(|x: f64| -> f64 {
            if x == 10.0 {
                100.0
            } else {
                200.0
            }
        });

        assert_eq!(call!(compiled, 10.0), 100.0);

        assert_eq!(call!(compiled, 5.0), 200.0);
    }

    // ============================================================
    // If/else with arithmetic expressions
    // ============================================================

    #[test]
    fn test_if_else_arithmetic() {
        let compiled = compile_closure!(|x: i32| -> i32 {
            if x > 10 {
                x * 2 + 5
            } else {
                x * 3 - 5
            }
        });

        assert_eq!(call!(compiled, 20), 45);

        assert_eq!(call!(compiled, 5), 10);
    }

    // ============================================================
    // Integer arithmetic
    // ============================================================

    #[test]
    fn test_i32_addition() {
        let compiled = compile_closure!(|a: i32| -> i32 { a + 5 });
        assert_eq!(call!(compiled, 2), 7);
    }

    #[test]
    fn test_i32_arithmetic() {
        let compiled = compile_closure!(|a: i32| -> i32 { ((a + 5) * 2) - 3 });

        assert_eq!(call!(compiled, 2), 11);
    }

    #[test]
    fn test_i32_division() {
        let compiled = compile_closure!(|a: i32| -> i32 { a / 2 });

        assert_eq!(call!(compiled, 10), 5);
    }

    #[test]
    fn test_i32_remainder() {
        let compiled = compile_closure!(|a: i32| -> i32 { a % 3 });

        assert_eq!(call!(compiled, 10), 1);
    }

    // ============================================================
    // Other signed integers
    // ============================================================

    #[test]
    fn test_i8() {
        let compiled = compile_closure!(|a: i8| -> i8 { a + 5 });

        assert_eq!(call!(compiled, 2i8), 7i8);
    }

    #[test]
    fn test_i16() {
        let compiled = compile_closure!(|a: i16| -> i16 { a * 3 });

        assert_eq!(call!(compiled, 4i16), 12i16);
    }

    #[test]
    fn test_i64() {
        let compiled = compile_closure!(|a: i64| -> i64 { a - 10 });

        assert_eq!(call!(compiled, 25i64), 15i64);
    }

    #[test]
    fn test_i128() {
        let compiled = compile_closure!(|a: i128| -> i128 { a + 100i128 });

        assert_eq!(call!(compiled, 25i128), 125i128);
    }

    // ============================================================
    // Unsigned integers
    // ============================================================

    #[test]
    fn test_u8() {
        let compiled = compile_closure!(|a: u8| -> u8 { a + 5 });

        assert_eq!(call!(compiled, 2u8), 7u8);
    }

    #[test]
    fn test_u16() {
        let compiled = compile_closure!(|a: u16| -> u16 { a * 3 });

        assert_eq!(call!(compiled, 4u16), 12u16);
    }

    #[test]
    fn test_u32() {
        let compiled = compile_closure!(|a: u32| -> u32 { a + 5 });

        assert_eq!(call!(compiled, 2u32), 7u32);
    }

    #[test]
    fn test_u64() {
        let compiled = compile_closure!(|a: u64| -> u64 { a * 2 });

        assert_eq!(call!(compiled, 21u64), 42u64);
    }

    #[test]
    fn test_u128() {
        let compiled = compile_closure!(|a: u128| -> u128 { a + 100u128 });

        assert_eq!(call!(compiled, 25u128), 125u128);
    }

    // ============================================================
    // Floating point
    // ============================================================

    #[test]
    fn test_f32_addition() {
        let compiled = compile_closure!(|a: f32| -> f32 { a + 5.0f32 });

        assert_eq!(call!(compiled, 2.0f32), 7.0f32);
    }

    #[test]
    fn test_f32_arithmetic() {
        let compiled = compile_closure!(|a: f32| -> f32 { (a * 2.0f32) - 1.0f32 });

        assert_eq!(call!(compiled, 4.0f32), 7.0f32);
    }

    #[test]
    fn test_f64_addition() {
        let compiled = compile_closure!(|a: f64| -> f64 { a + 5.0 });

        assert_eq!(call!(compiled, 2.0), 7.0);
    }

    #[test]
    fn test_f64_remainder() {
        let compiled = compile_closure!(|a: f64| -> f64 { a % 3.0 });

        assert_eq!(call!(compiled, 10.0), 1.0);
    }

    // ============================================================
    // Unary operators
    // ============================================================

    #[test]
    fn test_boolean_not() {
        let compiled = compile_closure!(|b: bool| -> bool { !b });

        assert!(!call!(compiled, true));
        assert!(call!(compiled, false));
    }

    #[test]
    fn test_integer_negation() {
        let compiled = compile_closure!(|a: i32| -> i32 { -a });

        assert_eq!(call!(compiled, 5), -5);
    }

    #[test]
    fn test_float_negation() {
        let compiled = compile_closure!(|a: f64| -> f64 { -a });

        assert_eq!(call!(compiled, 5.5), -5.5);
    }

    // ============================================================
    // Comparisons
    // ============================================================

    #[test]
    fn test_i32_equal() {
        let compiled = compile_closure!(|a: i32| -> bool { a == 5 });

        assert!(call!(compiled, 5));
        assert!(!call!(compiled, 4));
    }

    #[test]
    fn test_i32_less_than() {
        let compiled = compile_closure!(|a: i32| -> bool { a < 5 });

        assert!(call!(compiled, 4));
        assert!(!call!(compiled, 5));
    }

    #[test]
    fn test_i32_greater_than() {
        let compiled = compile_closure!(|a: i32| -> bool { a > 5 });

        assert!(call!(compiled, 6));
        assert!(!call!(compiled, 5));
    }

    #[test]
    fn test_u32_comparison() {
        let compiled = compile_closure!(|a: u32| -> bool { a < 10u32 });

        assert!(call!(compiled, 5u32));
        assert!(!call!(compiled, 10u32));
    }

    #[test]
    fn test_f64_comparison() {
        let compiled = compile_closure!(|a: f64| -> bool { a >= 5.0 });

        assert!(call!(compiled, 5.0));
        assert!(call!(compiled, 6.0));
        assert!(!call!(compiled, 4.0));
    }

    // ============================================================
    // Boolean operators
    // ============================================================

    #[test]
    fn test_boolean_and() {
        let compiled = compile_closure!(|a: bool| -> bool { a && true });

        assert!(call!(compiled, true));
        assert!(!call!(compiled, false));
    }

    #[test]
    fn test_boolean_or() {
        let compiled = compile_closure!(|a: bool| -> bool { a || false });

        assert!(call!(compiled, true));
        assert!(!call!(compiled, false));
    }

    // ============================================================
    // Bitwise operators
    // ============================================================

    #[test]
    fn test_bitwise_and() {
        let compiled = compile_closure!(|a: u32| -> u32 { a & 0xffu32 });

        assert_eq!(call!(compiled, 0x1234u32), 0x34u32);
    }

    #[test]
    fn test_bitwise_or() {
        let compiled = compile_closure!(|a: u32| -> u32 { a | 0x10u32 });

        assert_eq!(call!(compiled, 0x20u32), 0x30u32);
    }

    #[test]
    fn test_bitwise_xor() {
        let compiled = compile_closure!(|a: u32| -> u32 { a ^ 0xffu32 });

        assert_eq!(call!(compiled, 0xf0u32), 0x0fu32);
    }

    #[test]
    fn test_shift_left() {
        let compiled = compile_closure!(|a: u32| -> u32 { a << 2 });

        assert_eq!(call!(compiled, 3u32), 12u32);
    }

    #[test]
    fn test_shift_right() {
        let compiled = compile_closure!(|a: u32| -> u32 { a >> 2 });

        assert_eq!(call!(compiled, 12u32), 3u32);
    }

    // ============================================================
    // Struct support
    // ============================================================

    #[repr(C)]
    #[derive(Debug, Clone, Copy, CompileType)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, CompileType)]
    struct Rectangle {
        top_left: Point,
        bottom_right: Point,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, CompileType)]
    struct Nested {
        r: Rectangle,
        p: Point,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy, CompileType, PartialEq)]
    struct Point2(bool, i32);

    #[test]
    fn test_returned_tuple_struct() {
        let compiled = compile_closure!(|n: Point2| -> Point2 { n });

        let tup = Point2 { 0: true, 1: 10 };

        assert_eq!(call!(compiled, tup), Point2 { 0: true, 1: 10 });
    }

    #[test]
    fn test_returned_tuple() {
        let compiled = compile_closure!(|n: (f32, f32)| -> (f32, f32) { n });

        let tup = (20.0, 10.0);

        assert_eq!(call!(compiled, tup), (20.0, 10.0));
    }

    #[test]
    fn test_tuple_value() {
        let compiled = compile_closure!(|x: i32, y: i32| -> (i32, i32) { (x, y) });

        assert_eq!(call!(compiled, 40, 10), (40, 10));
    }

    #[test]
    fn test_nested_struct() {
        let compiled = compile_closure!(|n: Nested| -> f64 { n.r.bottom_right.x - n.p.x });

        let nested = Nested {
            r: Rectangle {
                top_left: Point { x: 10.0, y: 20.0 },
                bottom_right: Point { x: 30.0, y: 40.0 },
            },
            p: Point { x: 10.0, y: 5.0 },
        };

        assert_eq!(call!(compiled, nested), 20.0);
    }

    #[test]
    fn test_struct_field_arithmetic() {
        let compiled = compile_closure!(|r: Rectangle| -> f64 { r.bottom_right.x - r.top_left.x });

        let rectangle = Rectangle {
            top_left: Point { x: 10.0, y: 20.0 },

            bottom_right: Point { x: 30.0, y: 40.0 },
        };

        assert_eq!(call!(compiled, rectangle), 20.0);
    }

    #[test]
    fn dynamic_invocation_add_i32() {
        let closure = quote_closure!(|x: i32, y: i32| -> i32 { x + y });

        let context = inkwell::context::Context::create();

        let compiler = closure_llvm::Compiler::new(&context);

        let compiled = compiler
            .compile_dynamic(&closure)
            .expect("failed to compile closure");

        let result = unsafe {
            compiled
                .call(&[closure_llvm::Value::I32(10), closure_llvm::Value::I32(20)])
                .expect("failed to invoke closure")
        };

        assert_eq!(result, closure_llvm::Value::I32(30));
    }

    #[test]
    fn test_serialization() {
        let expr = quote_closure!(|x: i32| -> i32 { x });

        let serialized = serde_json::to_string(&expr).unwrap();

        println!("serialized expr is {}", serialized);

        let deserialized = serde_json::from_str(&serialized).unwrap();

        assert_eq!(expr, deserialized);
    }

    #[test]
    fn print_quoted_expr() {
        let expr = quote_closure!(|x: i32| -> i32 { x });

        println!("expr is {:?}", expr);
    }
}