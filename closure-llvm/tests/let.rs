use closure_llvm::{call, compile_closure};


#[test]
fn test_let_binding() {
    let compiled =
        compile_closure!(|x: i32| -> i32 {
            let y = x + 1;
            y * 2
        });

    assert_eq!(call!(compiled, 4), 10);
}


#[test]
fn test_multiple_let_bindings() {
    let compiled =
        compile_closure!(|x: i32| -> i32 {
            let y = x + 1;
            let z = y * 2;
            z - 3
        });

    assert_eq!(call!(compiled, 4), 7);
}


#[test]
fn test_annotated_let_binding() {
    let compiled =
        compile_closure!(|x: i32| -> i32 {
            let y: i32 = x + 1;
            y * 2
        });

    assert_eq!(call!(compiled, 4), 10);
}


#[test]
fn test_let_binding_in_if_expression() {
    let compiled =
        compile_closure!(|x: i32| -> i32 {
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
    let compiled =
        compile_closure!(|x: i32| -> i32 {
            let y = x + 1;
            let z = y * 2;
            let y = x + 10;
            y + z
        });

    assert_eq!(call!(compiled, 4), 24);
}