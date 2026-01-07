# RUST INTEGRATION

Rust components via cxx.rs FFI bridge. JSG Rust bindings + pure Rust utilities.

## OVERVIEW

```
src/rust/
├── jsg/            # Core JSG Rust bindings (v8::Local, Realm, Resource lifecycle)
├── jsg-macros/     # Procedural macros: #[jsg_struct], #[jsg_resource], #[jsg_method]
├── api/            # Runtime APIs (dns.rs - DNS parsing utilities)
├── kj/             # KJ library bindings (OwnOrRef, HTTP, IO)
├── dns/            # Pure Rust DNS utilities
├── python-parser/  # Pure Rust Python parser
└── transpiler/     # Pure Rust code transpiler
```

## FFI PATTERNS

### Handle Wrapper

V8 handles cross FFI as `usize` pointers:

```rust
#[cxx::bridge]
pub mod ffi {
    struct Local { ptr: usize }
    unsafe fn local_new_string(isolate: *mut Isolate, value: &str) -> Local;
}
```

### OwnOrRef<T> - Unified C++ Ownership

Unifies `kj::Own<T>`, `const T&`, `T&` across FFI (kj/own.rs:9):

```rust
pub enum OwnOrRef<'a, T> {
    Own(KjOwn<T>),      // kj::Own<T> → pass by value
    Ref(&'a T),         // const T& → &Wrapper
    MutRef(Pin<&'a mut T>), // T& → &mut Wrapper
}
```

### Resource Lifecycle (jsg/lib.rs:88)

1. `Ref<R>` leaked to raw pointer → stored in `ResourceState.this`
2. Wrapped in V8 object with drop fn callback as usize
3. `global_make_weak` sets weak callback on V8 GC
4. Callback drops leaked `Ref<R>` → Rust destructor runs

## JSG MACROS

### #[jsg_struct] - Plain Data (jsg-macros/lib.rs:36)

```rust
#[jsg_struct]
pub struct CaaRecord {
    pub critical: u8,    // Only pub fields exposed to JS
    pub field: String,
}
```

Generates `jsg::Type` impl, serializes to JS object via `ToLocalValue`.

### #[jsg_resource] - JS API Class

```rust
#[jsg_resource]
impl MyResource {
    #[jsg_method]
    pub fn my_method(&self, arg: String) -> jsg::Result<i32> { }
}
```

Generates `Resource` trait impl, method descriptors for V8 function templates.

### Realm - Per-Context Cleanup

Tracks all wrapped resources per isolate context. Weak callbacks deregister from Realm on GC.

## ADDING RUST CODE

1. **Pure Rust utility**: Add to `dns/`, `python-parser/`, or `transpiler/` - no FFI needed
2. **JSG API**:
   - Add to `api/`, use `#[jsg_resource]` + `#[jsg_method]`
   - Expose via C++ in `src/workerd/api/`, wire to JSG via `JSG_RESOURCE_TYPE`
3. **KJ/C++ bridge**: Add to `kj/`, define `#[cxx::bridge]` in separate `ffi` mod
4. **Build**: Update `BUILD.bazel` in parent dir, add crate to `rust_cxx_bridge` or `rust_library`

### Conventions

- `#[cxx::bridge]` always in nested `ffi` or `pub mod ffi`
- Pointers cross FFI as `usize` for V8 handles, raw pointers for C++ objects
- Error types: `impl From<MyError> for jsg::Error` for auto-conversion
- Use `jsg::Result<T, E>` for fallible JSG operations (E defaults to `jsg::Error`)
