use closure_pack::{call, compile_closure, CompileType};

#[repr(C)]
#[derive(Debug, Clone, Copy, CompileType)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

pub fn fft(values: &mut [Complex], pi: f64) {
    let compiled = compile_closure!(|values: &mut [Complex], pi: f64| -> usize {
        let n = values.len();

        let mut j: usize = 0;
        for i in 1..n {
            let mut bit: usize = n >> 1;
            while (j & bit) != 0 {
                j = j ^ bit;
                bit = bit >> 1;
            }
            j = j ^ bit;

            let swap_i: Complex = if i < j { values[j] } else { values[i] };
            let swap_j: Complex = if i < j { values[i] } else { values[j] };
            values[i] = swap_i;
            values[j] = swap_j;
        }

        let mut length: usize = 2;
        while length <= n {
            let half: usize = length >> 1;
            let angle_step: f64 = -2.0 * pi / (length as f64);

            let mut start: usize = 0;
            while start < n {
                let mut k: usize = 0;
                while k < half {
                    let angle: f64 = angle_step * (k as f64);
                    let w: Complex = Complex {
                        re: cos(angle),
                        im: sin(angle),
                    };

                    let even: Complex = values[start + k];
                    let odd: Complex = values[start + k + half];

                    let product: Complex = Complex {
                        re: odd.re * w.re - odd.im * w.im,
                        im: odd.re * w.im + odd.im * w.re,
                    };

                    values[start + k] = Complex {
                        re: even.re + product.re,
                        im: even.im + product.im,
                    };
                    values[start + k + half] = Complex {
                        re: even.re - product.re,
                        im: even.im - product.im,
                    };

                    k = k + 1;
                }
                start = start + length;
            }
            length = length << 1;
        }
        n
    });

    call!(compiled, values, pi);
}

fn main() {
    let mut values = [
        Complex { re: 1.0, im: 0.0 },
        Complex { re: 2.0, im: 0.0 },
        Complex { re: 3.0, im: 0.0 },
        Complex { re: 4.0, im: 0.0 },
        Complex { re: 5.0, im: 0.0 },
        Complex { re: 6.0, im: 0.0 },
        Complex { re: 7.0, im: 0.0 },
        Complex { re: 8.0, im: 0.0 },
    ];

    println!("Input: {:?}", values);
    fft(&mut values, std::f64::consts::PI);
    println!("8-point FFT: {:?}", values);
}
