# WORKERD KNOWLEDGE BASE

**Generated:** 2026-01-07 | **Commit:** cfbf4151a

## OVERVIEW

Cloudflare Workers JS/WASM runtime. C++23/Bazel/Cap'n Proto. Powers production Workers + local dev (wrangler).

## STRUCTURE

```
workerd/
├── src/workerd/        # Core runtime (C++)
│   ├── api/            # Runtime APIs exposed to JS (HTTP, crypto, streams, actors)
│   ├── jsg/            # JavaScript Glue - V8 bindings via macros
│   ├── io/             # I/O subsystem, actor storage, gates, IoContext
│   ├── server/         # Server binary, Cap'n Proto config
│   └── util/           # Utilities (SQLite, autogate, state machine)
├── src/node/           # Node.js compat (TypeScript public API)
├── src/rust/           # Rust integration via cxx.rs FFI
├── src/pyodide/        # Python runtime (Pyodide/WASM)
├── src/cloudflare/     # Cloudflare-specific APIs (TypeScript)
├── types/              # TypeScript definition generation
├── samples/            # Example worker configurations
└── build/              # Bazel rules, CI config
```

## WHERE TO LOOK

| Task                | Location                                         | Notes                                    |
| ------------------- | ------------------------------------------------ | ---------------------------------------- |
| Add new JS API      | `src/workerd/api/`                               | Use JSG macros, see jsg/AGENTS.md        |
| V8 binding patterns | `src/workerd/jsg/`                               | `JSG_RESOURCE_TYPE`, `JSG_METHOD` macros |
| Actor/storage       | `src/workerd/io/`                                | `ActorCache`, `IoContext`, gates         |
| Server config       | `src/workerd/server/workerd.capnp`               | Main schema                              |
| Compat flags        | `src/workerd/io/compatibility-date.capnp`        | 153+ flags                               |
| Node.js compat      | `src/node/` (TS) + `src/workerd/api/node/` (C++) | Hybrid impl                              |
| Add Rust code       | `src/rust/`                                      | cxx.rs bridge, `#[jsg_resource]`         |
| Python workers      | `src/pyodide/`                                   | Emscripten + snapshot system             |
| Run tests           | `just test` or `bazel test //...`                |                                          |
| Format code         | `just format`                                    | clang-format + prettier                  |

## BUILD

```bash
# Primary (Bazel)
bazel build //src/workerd/server:workerd
bazel test //...

# Developer shortcuts (just)
just build          # or just b
just test           # or just t
just format         # or just f
just node-test zlib # Node.js compat test
just wpt-test url   # Web Platform Test
just stream-test //path:target  # Stream output
```

## KEY PATTERNS

### JSG (JavaScript Glue)

```cpp
class MyApi: public jsg::Object {
  JSG_RESOURCE_TYPE(MyApi) {
    JSG_METHOD(myMethod);
    JSG_READONLY_INSTANCE_PROPERTY(prop, getProp);
  }
};
```

### Compatibility Flags

```capnp
myNewBehavior @42 :Bool
    $compatEnableFlag("my_new_behavior")
    $compatEnableDate("2025-01-01");
```

- Strong backward compat commitment
- Use compat date for breaking changes
- Use autogates (`src/workerd/util/autogate.*`) for risky rollouts

### Test Types

- `.wd-test` - Cap'n Proto integration tests (primary)
- `*-test.c++` - KJ unit tests
- `just node-test <name>` - Node.js compat
- `just wpt-test <name>` - Web Platform Tests

## CONVENTIONS

### File Naming

- `*.c++` / `*.h` (not .cc/.cpp/.hpp)
- `*-test.c++` for C++ tests
- `*.wd-test` for integration tests

### Code Style

- C++23, 100 char lines, 2-space indent
- `clang-format` enforced in CI
- Include order: local → workerd → 3rd party → kj/capnp → system

### TypeScript

- Strict mode, ESNext target
- `exactOptionalPropertyTypes`, `noUncheckedIndexedAccess`

## ANTI-PATTERNS

- **DO NOT** use `IoContext::getWaitUntilTasks()` - use `addWaitUntil()`
- **DO NOT** use `awaitIoLegacy()` - refactor to `awaitIo()`
- **NEVER** remove/change deployed features without compat flag
- **AVOID** non-standard APIs - prefer web standards

## GOTCHAS

- Cap'n Proto in `external/capnp-cpp` - consult for `kj::` and `capnp::` questions
- Property placement (prototype vs instance) controlled by compat flags
- Actor storage uses deferred writes with OutputGate for durability
- Streams API has 50+ known spec deviations (see WPT tests)

## DEPENDENCIES

- **V8** - JavaScript engine (`@workerd-v8//:v8`)
- **Cap'n Proto** - Config, RPC, schemas (`@capnp-cpp//...`)
- **KJ** - C++ utility library (part of Cap'n Proto)
- **pnpm** - Node package manager (v10.18.3)
