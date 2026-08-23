use closure_llvm::{
    call,
    compile_closure,
    quote_closure,
    CompileType,
};


// ============================================================
// Point
// ============================================================

#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    CompileType,
)]
struct Point {
    x: f64,
    y: f64,
}


// ============================================================
// Rectangle
// ============================================================

#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    CompileType,
)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}


fn quote_expr() {
    let expr = quote_closure!(|x: i32| -> bool {
        true
    });

    println!("quoted expression: {:?}", expr);

}


// ============================================================
// Main
// ============================================================

fn main() {
    let rectangle =
        Rectangle {
            top_left: Point {
                x: 10.0,
                y: 20.0,
            },

            bottom_right: Point {
                x: 50.0,
                y: 80.0,
            },
        };


    println!(
        "Rectangle: {:?}",
        rectangle
    );


    // --------------------------------------------------------
    // Compile the closure to LLVM
    //
    // compile_closure! does all of the following:
    //
    //   1. Creates the LLVM context
    //   2. Builds the Closure description
    //   3. Lowers the Rust expression
    //   4. Generates LLVM IR
    //   5. JIT compiles the function
    //   6. Returns CompiledClosure<Rectangle, f64>
    //
    // There is NO additional .compile() call.
    // --------------------------------------------------------

    let compiled =
        compile_closure!(
            |r: Rectangle| -> f64 {
                r.bottom_right.x
                    - r.top_left.x
            }
        );


    // --------------------------------------------------------
    // Call the JIT compiled function
    // --------------------------------------------------------

    let result =
        call!(
            compiled,
            rectangle
        );


    println!(
        "JIT result: {}",
        result
    );


    // --------------------------------------------------------
    // Expected result:
    //
    //     50.0 - 10.0 = 40.0
    // --------------------------------------------------------

    assert_eq!(
        result,
        40.0
    );

    //print the expression that is generated from a helper macro
    quote_expr();

}