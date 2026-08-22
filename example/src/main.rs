use closure_llvm::{
    compile_closure,
    CompileType,
    Compiler,
};

use inkwell::context::Context;


// ============================================================
// User-defined type
// ============================================================

#[derive(Debug, CompileType)]
struct Point {
    x: f64,
    y: f64,
}


// ============================================================
// Nested user-defined type
// ============================================================

#[derive(Debug, CompileType)]
struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}


fn main() {

    // --------------------------------------------------------
    // Type information
    // --------------------------------------------------------

    let point_info =
        Point::type_info();

    println!(
        "Point type:\n{point_info:#?}"
    );


    let rectangle_info =
        Rectangle::type_info();

    println!(
        "Rectangle type:\n{rectangle_info:#?}"
    );


    // --------------------------------------------------------
    // Compile closure
    // --------------------------------------------------------

    let closure =
        compile_closure!(
            |x: i32, y: i32| -> i32 {
                (x * 2) + y
            }
        );


    println!(
        "Closure IR:\n{closure:#?}"
    );


    // --------------------------------------------------------
    // LLVM/JIT
    // --------------------------------------------------------

    let context =
        Context::create();

    let compiler =
        Compiler::new(&context);


    let function =
        compiler
            .compile_i32_binary(
                &closure
            )
            .expect(
                "failed to compile closure"
            );


    let result =
        unsafe {
            function.call(
                10,
                20,
            )
        };


    println!(
        "10 * 2 + 20 = {}",
        result
    );
}