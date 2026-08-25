use closure_pack::{call, closure_pack, compile_closure, CompileType};

#[repr(C)]
#[derive(Debug, Clone, Copy, CompileType)]
struct Complex {
    re: f64,
    im: f64,
}

fn print_ir() {
    let expr = closure_pack!(|x: i32| -> bool { true });

    println!("IR expression: {:?}", expr);
}

fn main() {
    let mut values = [
        Complex { re: 3.0, im: 4.0 },
        Complex { re: 1.0, im: 2.0 },
        Complex { re: 5.0, im: 12.0 },
    ];

    println!("Input: {:?}", values);

    let compiled = compile_closure!(|values: &mut [Complex]| -> usize {
        for i in 0..values.len() {
            let z: Complex = values[i];
            let denominator: f64 = z.re * z.re + z.im * z.im;

            let result: Complex = if denominator == 0.0 {
                Complex { re: 0.0, im: 0.0 }
            } else {
                Complex {
                    re: z.re / denominator,
                    im: -z.im / denominator,
                }
            };

            values[i] = result;
        }

        values.len()
    });

    let processed = call!(compiled, &mut values[..]);
    assert_eq!(processed, values.len());

    println!("JIT result: {:?}", values);

    assert_eq!(values[0].re, 0.12);
    assert_eq!(values[0].im, -0.16);
    assert_eq!(values[1].re, 0.2);
    assert_eq!(values[1].im, -0.4);
    assert_eq!(values[2].re, 5.0 / 169.0);
    assert_eq!(values[2].im, -12.0 / 169.0);

    print_ir();
}
