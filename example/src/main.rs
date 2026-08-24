use closure_ir::{
    call,
    compile_closure,
    closure_ir,
    CompileType,
};


// ============================================================
// Complex
// ============================================================

#[repr(C)]
#[derive(
    Debug,
    Clone,
    Copy,
    CompileType,
)]
struct Complex {
    re: f64,
    im: f64,
}


fn quote_expr() {
    let expr = closure_ir!(|x: i32| -> bool {
        true
    });

    println!("quoted expression: {:?}", expr);
}


// ============================================================
// Main
// ============================================================

fn main() {
    let value = Complex {
        re: 3.0,
        im: 4.0,
    };

    println!(
        "Complex: {:?}",
        value
    );


    // --------------------------------------------------------
    // Compile the closure to LLVM
    //
    // The reciprocal of a complex number is:
    //
    //     1 / (a + bi) = (a - bi) / (a² + b²)
    //
    // Therefore:
    //
    //     re = a / (a² + b²)
    //     im = -b / (a² + b²)
    // --------------------------------------------------------

    let compiled =
        compile_closure!(
            |z: Complex| -> Complex {
                let denominator =
                    z.re * z.re
                    + z.im * z.im;

                if denominator == 0.0 {
                    Complex {
                        re: 0.0,
                        im: 0.0,
                    }
                } else {
                    let re =
                        z.re / denominator;

                    let im =
                        -z.im / denominator;

                    Complex {
                        re,
                        im,
                    }
                }
            }
        );


    // --------------------------------------------------------
    // Call the JIT compiled function
    // --------------------------------------------------------

    let result =
        call!(
            compiled,
            value
        );


    println!(
        "JIT result: {:?}",
        result
    );


    // --------------------------------------------------------
    // Expected result:
    //
    //     1 / (3 + 4i)
    //       = (3 - 4i) / 25
    //       = 0.12 - 0.16i
    // --------------------------------------------------------

    assert_eq!(
        result.re,
        0.12
    );

    assert_eq!(
        result.im,
        -0.16
    );

    // Print the expression generated from a helper macro.
    quote_expr();
}
