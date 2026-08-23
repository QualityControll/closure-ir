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
        └── main.rs
```

## Example

```rust
use closure_ir::{
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
 - [ ] Multiple statements
 - [ ] Function calls
 - [ ] Nested expressions
 - [ ] Arrays
 - [x] Tuples
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
