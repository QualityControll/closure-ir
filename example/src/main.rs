use closure_llvm::{
    compile_closure,
    CompileType,
    Compiler,
};

use inkwell::context::Context;


// ============================================================
// Point
// ============================================================

#[derive(Debug, Clone, Copy, CompileType)]
struct Point {
    x: f64,
    y: f64,
}


// ============================================================
// Rectangle
// ============================================================

#[derive(Debug, Clone, Copy, CompileType)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}


// ============================================================
// Main
// ============================================================

fn main() {
    // --------------------------------------------------------
    // Create LLVM context
    // --------------------------------------------------------

    let context =
        Context::create();


    // --------------------------------------------------------
    // Build our Rust-side Rectangle
    // --------------------------------------------------------

    let rectangle =
        Rectangle {
            top_left: Point {
                x: 10.0,
                y: 20.0,
            },

            bottom_right: Point {
                x: 110.0,
                y: 70.0,
            },
        };


    println!(
        "Rectangle: {:?}",
        rectangle
    );


    // --------------------------------------------------------
    // Generate the closure IR
    //
    // This closure:
    //
    //     |r: Rectangle| -> f64 {
    //         r.bottom_right.x - r.top_left.x
    //     }
    //
    // should ultimately become approximately:
    //
    //     double @closure(%Rectangle* %r)
    //
    // and perform:
    //
    //     r.bottom_right.x - r.top_left.x
    // --------------------------------------------------------

    let closure =
        compile_closure!(
            |r: Rectangle| -> f64 {
                r.bottom_right.x - r.top_left.x
            }
        );


    // --------------------------------------------------------
    // Compile to LLVM/JIT
    // --------------------------------------------------------

    let compiler =
        Compiler::new(
            &context
        );


    let compiled =
        compiler
            .compile(&closure)
            .expect(
                "failed to compile closure"
            );


    println!(
        "Closure compiled successfully."
    );


    println!(
        "Function: {}",
        compiled.function_name
    );


    // --------------------------------------------------------
    // NOTE:
    //
    // The next step is obtaining a correctly typed JIT function
    // pointer and calling it with &rectangle.
    //
    // For example, conceptually:
    //
    //     type Fn =
    //         unsafe extern "C" fn(
    //             *const Rectangle
    //         ) -> f64;
    //
    // That part should be added once the ABI for generated
    // structs is finalized.
    // --------------------------------------------------------

    let _ = rectangle;
}