# closure-ir

A portable intermediate representation for Rust closures.

`closure-ir` is an experimental Rust project that turns Rust closures into an intermediate representation (IR), then lowers that representation to LLVM and JIT-compiles it with [Inkwell](https://github.com/TheDan64/inkwell).

The project explores a longer-term question: **can a Rust closure become a portable, serializable executable representation that can be transported to another execution environment and compiled there?**

> **Status:** Experimental / proof of concept. The current implementation supports only a small subset of Rust expressions and types. Remote/distributed execution is exploratory future work and is not currently implemented.

## What makes this different?

Traditional RPC and distributed runtimes generally send **arguments to a function that is already compiled into the remote application**. For example, an active-message system can identify a registered operation and send it serialized state:

```text
function / AM ID + arguments
            |
            v
    pre-installed function
            |
            v
         execute
```

`closure-ir` explores a different model:

```text
Rust closure
     |
     v
  closure IR
     |
     +---- closure code representation
     |
     +---- captured state
     |
     v
 serialize / transport
     |
     v
remote execution environment
     |
     v
   LLVM / JIT
     |
     v
  execute
```

The goal is therefore not simply to serialize closure arguments. The longer-term goal is to make the **closure's executable representation itself portable**.

This could eventually enable use cases such as:

- serializable Rust closures
- portable closure execution
- remote closure execution
- JIT compilation on a remote worker
- distributed execution of user-defined computations
- sending computations to machines that do not have the original closure compiled into their application

These capabilities are future goals; the current project is focused on building the closure IR, type metadata, lowering, and LLVM execution pieces needed to explore them.

## Goals

The current implementation explores whether Rust's compiler-visible syntax and procedural macros can be used to build a generic system that:

1. Accepts a normal Rust closure.
2. Inspects its syntax at compile time using a procedural macro.
3. Converts the closure into a language-independent IR.
4. Uses generated type metadata to understand user-defined Rust types.
5. Lowers the IR into LLVM IR.
6. JIT-compiles the resulting LLVM code.
7. Executes the generated function.
8. Eventually provides a representation that can be serialized independently of the original process.

## Architecture

The project is split into two crates:

```text
closure-ir/
│
├── closure-ir/
│   └── src/
│       └── lib.rs
│
├── closure-ir-macro/
│   └── src/
│       └── lib.rs
│
└── example/
    ├── Cargo.toml
    └── src/
        ├── main.rs
        └── fft.rs
```

## Example

The following example demonstrates a JIT-compiled complex-number reciprocal. It exercises struct arguments, struct field access, arithmetic, conditionals, unary negation, and **struct construction inside the closure**.

```rust
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
            if z.re * z.re + z.im * z.im == 0.0 {
                Complex {
                    re: 0.0,
                    im: 0.0,
                }
            } else {
                Complex {
                    re: z.re / (z.re * z.re + z.im * z.im),
                    im: -z.im / (z.re * z.re + z.im * z.im),
                }
            }
        }
    );

    let result = call!(compiled, value);

    println!("JIT result: {:?}", result);

    assert_eq!(result.re, 0.12);
    assert_eq!(result.im, -0.16);

    quote_expr();
}
```

The reciprocal is computed as:

\[
\frac{1}{a + bi} = \frac{a - bi}{a^2 + b^2}
\]

For `3 + 4i`, the expected result is `0.12 - 0.16i`.

### FFT example

The project also includes a small in-place radix-2 Cooley–Tukey FFT over a mutable slice of `Complex` values. The FFT demonstrates a much larger portion of the supported closure language, including `len()`, `for` and `while` loops, mutable locals, dynamic slice indexing, indexed assignment, casts, bitwise operations, conditionals, and the `sin`/`cos` intrinsics.

```rust
use closure_ir::{call, compile_closure, CompileType};

#[repr(C)]
#[derive(Debug, Clone, Copy, CompileType)]
struct Complex {
    re: f64,
    im: f64,
}

let compiled = compile_closure!(
    |values: &mut [Complex], pi: f64| {
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
    }
);

call!(compiled, &mut values[..], std::f64::consts::PI);
```

For the complete runnable version, see `example/src/fft.rs`.

## Related work

### Distributed active messages

Projects such as [Lamellar](https://github.com/pnnl/lamellar-runtime) provide remote execution through distributed active messages. An active-message type and its `exec()` implementation are compiled into the participating application; the runtime sends serialized state to the remote PE, which invokes the already-installed implementation.

`closure-ir` explores a different layer of the problem: making the **closure representation itself portable**, so a remote execution environment could potentially receive the computation, compile/JIT it, and then execute it without having the original closure precompiled into the worker application.

This is complementary to a distributed runtime rather than an attempt to replace one. A future system could potentially use a runtime such as Lamellar for communication and scheduling while using a portable closure representation for computation.

## Roadmap

 - [x] Generalize LLVM type lowering
 - [x] Complete struct argument lowering
 - [x] Struct field access in LLVM
 - [x] Floating-point expressions
 - [x] Boolean expressions
 - [x] Comparisons
 - [x] Conditional expressions
 - [x] if expressions
 - [x] Local variables
 - [x] Multiple statements
 - [ ] Function calls
 - [x] Nested expressions
 - [ ] Arrays
 - [x] Tuples
 - [x] Struct literals
 - [ ] Enums
 - [ ] Generic types
 - [ ] Better error reporting
 - [ ] LLVM optimization passes
 - [ ] A stable executable-function API
 - [ ] Benchmark generated LLVM against native Rust
 - [ ] Support more complex user-defined types
 - [ ] Explore automatic serialization/deserialization
 - [ ] Explore distributed execution
 - [ ] Explore portable/remote closure execution

## License

License information will be added as the project develops.
