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
    let mut values = [
        Complex { re: 3.0, im: 4.0 },
        Complex { re: 1.0, im: 2.0 },
        Complex { re: 5.0, im: 12.0 },
    ];

    println!("Input: {:?}", values);

    let compiled = compile_closure!(
        |values: &mut [Complex], count: usize| -> usize {
            for i in 0..count {
                let z: Complex = values[i];
                let denominator: f64 = z.re * z.re + z.im * z.im;

                if denominator == 0.0 {
                    values[i] = Complex {
                        re: 0.0,
                        im: 0.0,
                    };
                } else {
                    values[i] = Complex {
                        re: z.re / denominator,
                        im: -z.im / denominator,
                    };
                }
            }

            count
        }
    );

    let processed = call!(compiled, &mut values[..], values.len());
    assert_eq!(processed, values.len());

    println!("JIT result: {:?}", values);

    assert_eq!(values[0].re, 0.12);
    assert_eq!(values[0].im, -0.16);
    assert_eq!(values[1].re, 0.2);
    assert_eq!(values[1].im, -0.4);
    assert_eq!(values[2].re, 5.0 / 169.0);
    assert_eq!(values[2].im, -12.0 / 169.0);

    quote_expr();
}
