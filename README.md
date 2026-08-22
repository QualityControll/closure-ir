# Rust Closure → LLVM

An experimental Rust library for turning Rust closures into an intermediate representation (IR) using procedural macros, and then compiling that IR to executable LLVM code with [Inkwell](https://github.com/TheDan64/inkwell).

The goal is to explore whether Rust's compiler-visible syntax and procedural macros can be used to build a generic system that:

1. Accepts a normal Rust closure.
2. Inspects its syntax at compile time using a procedural macro.
3. Converts the closure into a language-independent IR.
4. Uses generated type metadata to understand user-defined Rust types.
5. Lowers the IR into LLVM IR.
6. JIT-compiles the resulting LLVM code.
7. Executes the generated function.

> **Status:** Experimental / proof of concept. The current implementation supports only a small subset of Rust expressions and types.

---

## Architecture

The project is split into two crates:

```text
rust-closure-llvm/
│
├── closure-llvm/
│   └── src/
│       └── lib.rs
│
├── closure-llvm-macro/
│   └── src/
│       └── lib.rs
│
└── example/
    ├── Cargo.toml
    └── src/
        └── main.rs
```
## Example

```rust
use closure_llvm::{
    call,
    compile_closure,
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
}
```

## Roadmap

 - [x] Generalize LLVM type lowering
 - [x] Complete struct argument lowering
 - [x] Struct field access in LLVM
 - [x] Floating-point expressions
 - [x] Boolean expressions
 - [x] Comparisons
 - [x] Conditional expressions
 - [x] if expressions
 - [ ] Local variables
 - [ ] Multiple statements
 - [ ] Function calls
 - [ ] Nested expressions
 - [ ] Arrays
 - [ ] Tuples
 - [ ] Enums
 - [ ] Generic types
 - [ ] Better error reporting
 - [ ] LLVM optimization passes
 - [ ] A stable executable-function API
 - [ ] Benchmark generated LLVM against native Rust
 - [ ] Support more complex user-defined types
 - [ ] Explore automatic serialization/deserialization
 - [ ] Explore distributed execution

 ## License

 License information will be added as the project develops.

