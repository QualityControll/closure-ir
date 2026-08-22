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
