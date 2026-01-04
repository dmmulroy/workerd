# JSG (JavaScript Glue)

V8 binding layer. Foundation for all JS-exposed APIs. ~39k lines.

## OVERVIEW

Maps C++ types to JavaScript via V8. Provides macros, type wrappers, promise handling, GC integration.

## KEY FILES

| File              | Purpose                                              |
| ----------------- | ---------------------------------------------------- |
| `jsg.h`           | Core macros (`JSG_RESOURCE_TYPE`, etc.) - 3016 lines |
| `resource.h`      | V8 object binding templates - 2001 lines             |
| `type-wrapper.h`  | Type conversion machinery                            |
| `wrappable.h`     | Base class for JS-exposed objects                    |
| `promise.h`       | JS↔KJ promise bridging                              |
| `memory.h`        | Heap snapshot integration                            |
| `modules-new.c++` | ES module loading                                    |

## MACROS

### Type Definition

| Macro                     | Use                      |
| ------------------------- | ------------------------ |
| `JSG_RESOURCE_TYPE(T) {}` | Declare JS-exposed class |
| `JSG_STRUCT`              | Value type struct        |
| `JSG_CALLABLE`            | Make type callable       |
| `JSG_INHERIT(Base)`       | Type inheritance         |
| `JSG_NESTED_TYPE(T)`      | Nested type registration |

### Members

| Macro                                  | Use                        |
| -------------------------------------- | -------------------------- |
| `JSG_METHOD(name)`                     | Instance method            |
| `JSG_STATIC_METHOD(name)`              | Static method              |
| `JSG_PROTOTYPE_PROPERTY(name)`         | Getter/setter on prototype |
| `JSG_READONLY_INSTANCE_PROPERTY(name)` | Read-only instance prop    |
| `JSG_LAZY_INSTANCE_PROPERTY(name)`     | Lazy-init property         |
| `JSG_STATIC_CONSTANT(name)`            | Static constant            |
| `JSG_ITERABLE(method)`                 | Make iterable              |

### TypeScript

| Macro                  | Use                         |
| ---------------------- | --------------------------- |
| `JSG_TS_OVERRIDE(...)` | Override TS type generation |
| `JSG_TS_DEFINE(...)`   | Custom TS definition        |
| `JSG_TS_ROOT`          | Mark as TS root type        |

## ANTI-PATTERNS

| Pattern                          | Why                                          |
| -------------------------------- | -------------------------------------------- |
| `JSG_INSTANCE_PROPERTY`          | Usually wrong - use `JSG_PROTOTYPE_PROPERTY` |
| Property named `"then"`          | V8 treats object as thenable                 |
| Store JS refs without GC visitor | Memory leak / use-after-free                 |
| Multiple `V8System` per process  | Forbidden - one per process                  |

## GC INTEGRATION

C++ objects holding JS refs MUST implement:

```cpp
void visitForGc(jsg::GcVisitor& visitor) {
  visitor.visit(myJsRef);
}
```

For heap snapshots:

```cpp
JSG_MEMORY_INFO(MyClass) {
  tracker.trackField("field", field);
}
```

## PROMISE BRIDGING

```cpp
// JS promise → KJ promise
jsg::Promise<T> jsPromise = ...;
kj::Promise<T> kjPromise = jsPromise.whenResolved(lock);

// KJ promise → JS promise (MUST use continuation)
auto jsPromise = ioContext.awaitIo(
  kj::mv(kjPromise),
  [](auto& context, T result) { return result; }
);
```

**NEVER use `awaitIoLegacy()`** - deprecated.

## TESTS

```bash
just test //src/workerd/jsg:jsg-test
```
