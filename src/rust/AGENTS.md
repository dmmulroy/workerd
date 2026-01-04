# RUST COMPONENTS

Rust FFI layer via `cxx` bridge. No Cargo.toml - managed entirely by Bazel.

## STRUCTURE

```
rust/
├── jsg/              # Rust↔V8 JSG bindings
├── kj/               # KJ async integration
├── api/              # Rust API layer
├── cxx-integration/  # C++/Rust FFI bridge
├── dns/              # DNS resolver
├── net/              # Network utilities
├── transpiler/       # JS transpiler
├── python-parser/    # Python parser
├── jsg-macros/       # Proc macros for JSG
└── gen-compile-cache/# V8 compile cache tool (only binary)
```

## BUILD

All via Bazel rules:

- `wd_rust_crate` - Library crate
- `wd_rust_binary` - Binary crate
- `wd_rust_proc_macro` - Proc macro crate

## CXX BRIDGE PATTERN

```rust
#[cxx::bridge(namespace = "workerd::rust::jsg")]
mod ffi {
    extern "Rust" {
        fn some_rust_function() -> i32;
    }
    extern "C++" {
        fn some_cpp_function();
    }
}
```

FFI structs use pointer-as-integer pattern:

```rust
pub struct Local { ptr: usize }
pub struct Global { ptr: usize }
```

## CONVENTIONS

- Pedantic + nursery clippy
- No `unwrap()` outside tests (use `expect` or `?`)
- One import per line (`imports_granularity = "Item"`)
- `dbg!` macro: warn
- `allow_attributes`: warn

## LINT

```bash
just clippy <package>   # e.g., just clippy jsg-macros
```

## INITIALIZATION

Rust components init at startup:

```cpp
workerd::rust::cxx_integration::init();
```

## KEY CRATES

| Crate             | Purpose                                      |
| ----------------- | -------------------------------------------- |
| `jsg`             | V8 handle abstractions, Type/Resource traits |
| `jsg-macros`      | Proc macros for JSG integration              |
| `kj`              | KJ Promise/async integration                 |
| `cxx-integration` | FFI bridge setup                             |

## TYPE SYSTEM

- `Type` trait: `class_name()`, `wrap()`, `unwrap()`, `is_exact()`
- `Resource` trait: Extends Type with `members()`, `get_drop_fn()`
- `NonCoercible<T>`: Prevents JS type coercion
