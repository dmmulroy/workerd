# WORKERD API LAYER

Runtime APIs exposed to JavaScript via JSG (JavaScript Glue). HTTP, crypto, streams, actors, WebSocket, Node.js compat.

## WHERE TO LOOK

| Task                  | Location                           | Notes                                  |
| --------------------- | ---------------------------------- | -------------------------------------- |
| HTTP/Request/Response | `http.c++` (2504L)                 | Body mixin, Headers, fetch()           |
| Web Crypto            | `crypto/` (50+ algorithms)         | `impl.h` = OpenSSL, split by algorithm |
| Streams               | `streams/standard.c++` (4284L)     | WHATWG Streams, 50+ WPT deviations     |
| Events/EventTarget    | `basics.h`                         | Event, EventTarget, AbortSignal        |
| Worker RPC            | `worker-rpc.c++` (2244L)           | Inter-worker RPC, stub system          |
| Global scope          | `global-scope.h`                   | ServiceWorkerGlobalScope               |
| Node.js compat (C++)  | `node/`                            | C++ side, pairs with `src/node/` (TS)  |
| Python integration    | `pyodide/`                         | Pyodide/WASM bridge                    |
| Integration tests     | `tests/*.wd-test`                  | Cap'n Proto config tests               |
| TypeScript overrides  | `JSG_TS_DEFINE`, `JSG_TS_OVERRIDE` | Inline TS type definitions             |

## KEY PATTERNS

### JSG Resource Registration

```cpp
class MyApi: public jsg::Object {
  kj::String myMethod(jsg::Lock& js, kj::String input);
  int getProp() { return value; }

  JSG_RESOURCE_TYPE(MyApi) {
    JSG_METHOD(myMethod);
    JSG_READONLY_INSTANCE_PROPERTY(prop, getProp);
    JSG_TS_OVERRIDE(MyApi { specialType: CustomType });  // TypeScript override
  }
};

void registerMyApi(auto& builder) {
  builder.addBuiltinModule("my:api", workerd::MyApiModule::Reader::INSTANCE);
}
```

### Body Mixin Pattern

Used by Request/Response to share body methods (`.json()`, `.text()`, `.arrayBuffer()`). Implemented via CRTP (Curiously Recurring Template Pattern).

### Compat Flag Property Placement

```cpp
JSG_INSTANCE_PROPERTY(prop, getProp, setProp);           // Instance (new default)
JSG_PROTOTYPE_PROPERTY(prop, getProp, setProp);          // Prototype (legacy)
JSG_READONLY_PROTOTYPE_PROPERTY(prop, getProp);          // Readonly prototype
```

Controlled by compat flags in `io/compatibility-date.capnp`. Breaking changes = new compat date.

### EventTarget Architecture

```cpp
class MyTarget: public EventTarget { /* ... */ };
// Inherits addEventListener(), dispatchEvent(), removeEventListener()
```

## CONVENTIONS

- **File org**: Large APIs split by feature (`crypto/` by algorithm, `streams/` by spec section)
- **Naming**: Match Web Platform APIs exactly (case-sensitive)
- **Lock management**: Pass `jsg::Lock& js` for V8 operations, never store raw V8 handles
- **Memory**: Use `kj::Own<T>`, `jsg::Ref<T>`, avoid raw pointers
- **Async**: Return `jsg::Promise<T>`, use `IoContext::current()` for I/O
- **TS types**: Use `JSG_TS_DEFINE`/`JSG_TS_OVERRIDE` for complex types, overloads

## ANTI-PATTERNS

- **DO NOT** hold `jsg::Lock&` across async boundaries (V8 unlocks)
- **DO NOT** use raw V8 API directly - use JSG wrappers
- **DO NOT** deviate from Web Platform specs without compat flag
- **NEVER** expose C++ exceptions to JS - wrap in `jsg::throwException()`
- **AVOID** synchronous I/O - everything must be async via promises
- **DO NOT** register properties without considering compat flag impact
