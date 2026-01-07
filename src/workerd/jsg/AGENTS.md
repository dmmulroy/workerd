# JSG (JAVASCRIPT GLUE) LAYER

**Generated:** 2026-01-07 | **Component:** src/workerd/jsg

## OVERVIEW

Bidirectional type translation between C++ and V8 JavaScript. Macro-driven system generates V8 trampoline callbacks, manages GC integration, handles refcounting. Core abstraction: C++ resource/struct types → JS objects via lazy wrapper creation.

## MACRO REFERENCE

```cpp
// Resource type (by-reference, C++ methods callable from JS)
class MyApi: public jsg::Object {
  JSG_RESOURCE_TYPE(MyApi) {
    JSG_METHOD(foo);                          // Instance method
    JSG_STATIC_METHOD(bar);                   // Constructor method
    JSG_INSTANCE_PROPERTY(baz, getBaz, setBaz); // Getter/setter
    JSG_READONLY_INSTANCE_PROPERTY(qux, getQux); // Readonly
    JSG_PROTOTYPE_PROPERTY(widget, getWidget, setWidget); // On prototype
    JSG_ITERABLE(entries);                    // For-of iteration
    JSG_CALLABLE(call);                       // Invocable as function
    JSG_NESTED_TYPE(SubType);                 // Nested resource
  }
  void visitForGc(jsg::GcVisitor& visitor);   // Required if holds JS handles
};

// Struct type (by-value, deep copy to JS object)
struct MyStruct {
  kj::String name;
  int value;
  JSG_STRUCT(name, value);                    // Field list
  JSG_STRUCT_TS_OVERRIDE(SomeType);           // Override TypeScript def
};
```

## TYPE MAPPING

| C++ Type                  | JS Type           | Notes                                       |
| ------------------------- | ----------------- | ------------------------------------------- |
| `jsg::Ref<T>`             | Resource object   | Lazy wrapper, refcounted                    |
| `jsg::Value`              | any               | Opaque JS value                             |
| `jsg::V8Ref<v8::T>`       | v8 handle         | Safe destruction outside isolate lock       |
| `jsg::Optional<T>`        | T \| undefined    | Standard optional                           |
| `jsg::LenientOptional<T>` | T \| undefined    | Type errors → undefined (coercion)          |
| `kj::Maybe<T>`            | T \| null         | Null handling                               |
| `kj::OneOf<T, U>`         | T \| U            | Union types                                 |
| `jsg::Promise<T>`         | Promise<T>        | Async/await integration                     |
| `jsg::Function<T(U,V)>`   | (u: U, v: V) => T | Callback wrapper                            |
| `jsg::Dict<T>`            | Record<string, T> | String-keyed map                            |
| `kj::Array<byte>`         | ArrayBuffer       | Binary data                                 |
| `v8::Local<T>`            | any               | Escape hatch (requires HandleScope)         |
| `const TypeHandler<T>&`   | N/A               | Magic param: C++ ↔ V8 converter (trailing) |

## GC INTEGRATION

**Wrappable lifecycle:**

- Refcounted (`kj::Refcounted`) + wrapper refcount for JS object identity
- Lazy wrapper creation on first JS exposure
- Wrapper keeps C++ alive; wrapper refcount prevents GC while C++ refs exist
- `visitForGc()` must call `visitor.visit()` on all `jsg::Ref`, `jsg::Value`, `jsg::V8Ref` members

```cpp
void MyApi::visitForGc(jsg::GcVisitor& visitor) {
  visitor.visit(otherResource, someValue, v8State); // Trace all JS handles
}
```

**Destructor safety:** Resources may be destroyed outside isolate lock. Use `jsg::V8Ref<T>` not raw `v8::Global<T>`.

## GOTCHAS

- **No v8::Local in JSG_STRUCT fields (receive side)** - use `jsg::V8Ref` / `jsg::JsRef` (no HandleScope guarantee)
- **FunctionCallbackInfo magic** - first param `const v8::FunctionCallbackInfo<v8::Value>&` bypasses unwrapping
- **Trailing magic params** - `const TypeHandler<T>&`, `v8::Isolate*` don't consume JS args
- **Constructor naming** - static method named `constructor` becomes JS `new MyApi(...)`
- **Prototype vs instance** - property placement controlled by compat flags (historical Web IDL issues)
- **Trampoline pattern** - macros generate V8 callbacks → unwrap args → call C++ → wrap result
- **visitForGc required** - missing implementation = memory leaks when holding JS handles
- **Struct vs resource** - structs copy deeply (no identity), resources maintain JS wrapper identity
