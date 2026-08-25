use closure_pack::{call, compile_closure};

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
