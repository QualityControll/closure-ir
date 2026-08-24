use closure_ir::{
    call,
    compile_closure,
    closure_ir,
    CompileType,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, CompileType)]
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

fn main() {
    let value = Complex { re: 3.0, im: 4.0 };

    println!("Complex: {:?}", value);

    let compiled = compile_closure!(
        |z: Complex| -> Complex {
            let denominator: f64 =
                z.re * z.re + z.im * z.im;

            if denominator == 0.0 {
                Complex { re: 0.0, im: 0.0 }
            } else {
                let re: f64 = z.re / denominator;
                let im: f64 = -z.im / denominator;

                Complex { re, im }
            }
        }
    );

    let result = call!(compiled, value);

    println!("JIT result: {:?}", result);

    assert_eq!(result.re, 0.12);
    assert_eq!(result.im, -0.16);

    quote_expr();
}
