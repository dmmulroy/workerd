# WORKERD KNOWLEDGE BASE

**Generated:** 2026-01-03 | **Commit:** 39fb8495e

## OVERVIEW

Cloudflare's JS/WASM runtime (Workers). C++23/Rust/TypeScript. Bazel-only build. Cap'n Proto for config/RPC. V8 engine.

## STRUCTURE

```
src/
├── workerd/           # Core C++ runtime
│   ├── api/           # JS runtime APIs (HTTP, crypto, streams) → see api/AGENTS.md
│   ├── jsg/           # V8 binding glue (JSG macros) → see jsg/AGENTS.md
│   ├── io/            # I/O subsystem, actors, requests → see io/AGENTS.md
│   ├── server/        # Main binary, config → see server/AGENTS.md
│   └── util/          # Utilities (SQLite, state machines) → see util/AGENTS.md
├── node/              # Node.js compat TS layer → see node/AGENTS.md
├── rust/              # Rust FFI components → see rust/AGENTS.md
├── cloudflare/        # CF-specific APIs (D1, R2, AI) → see internal/AGENTS.md
├── pyodide/           # Python runtime support → see pyodide/AGENTS.md
└── wpt/               # Web Platform Tests harness → see wpt/AGENTS.md
build/                 # Bazel rules/macros → see build/AGENTS.md
types/                 # TS type generation from C++
```

## WHERE TO LOOK

| Task               | Location                                         | Notes                         |
| ------------------ | ------------------------------------------------ | ----------------------------- |
| Add new JS API     | `src/workerd/api/`                               | Use `JSG_RESOURCE_TYPE` macro |
| Node.js compat     | `src/workerd/api/node/` (C++) + `src/node/` (TS) | Split implementation          |
| V8 binding issues  | `src/workerd/jsg/`                               | JSG macro system              |
| Request lifecycle  | `src/workerd/io/io-context.h`                    | Central I/O context           |
| Actor/DO storage   | `src/workerd/io/actor-cache.c++`                 | Actor caching layer           |
| Config schema      | `src/workerd/server/workerd.capnp`               | Cap'n Proto format            |
| Compat flags       | `src/workerd/io/compatibility-date.capnp`        | Feature flags                 |
| Custom Bazel rules | `build/*.bzl`                                    | wd_test, wd_cc_embed, etc.    |
| TypeScript types   | `types/`                                         | Generated from C++ AST        |
| Python Workers     | `src/pyodide/`                                   | Pyodide integration           |
| WPT compliance     | `src/wpt/`                                       | Web Platform Tests            |

## CONVENTIONS

### C++ (non-standard)

- **C++23** standard (`-std=c++23`)
- **100-column** line limit
- **2-space** indent
- **Clang-only** toolchain
- Include order: local > workerd > kj/capnp > 3rd party > system
- Pointer alignment: left (`int* p`)
- KJ macros: `KJ_SWITCH_ONEOF`, `KJ_CASE_ONEOF`, `KJ_IF_SOME`

### TypeScript

- Strict mode + `noUncheckedIndexedAccess`
- Use `#` private fields (not `private` keyword)
- Explicit return types required
- Unused vars must prefix with `_`

### Rust

- Pedantic + nursery clippy
- No `unwrap()` outside tests (use `expect` or `?`)
- One import per line (`imports_granularity = "Item"`)

## ANTI-PATTERNS

| Pattern                     | Why Forbidden                                 |
| --------------------------- | --------------------------------------------- |
| Change compat flag behavior | Strong backward compat commitment             |
| Remove deployed features    | Cannot change once deployed                   |
| `JSG_INSTANCE_PROPERTY`     | Usually wrong; use `JSG_PROTOTYPE_PROPERTY`   |
| Property named `"then"`     | V8 treats as thenable                         |
| `awaitIoLegacy()`           | Deprecated; use `awaitIo()` with continuation |
| Store KJ I/O in JS heap     | Must use `IoOwn` wrapper                      |
| Modify outcome.capnp        | Sync with authoritative version               |
| New global exports          | Must be behind compat flag                    |
| New Fetcher methods         | Must be gated via compat flag                 |

## COMMANDS

```bash
# Build
just build                    # or: bazel build //src/workerd/server:workerd

# Test
just test                     # All tests
just node-test zlib           # Specific Node.js compat test
just wpt-test urlpattern      # Web Platform Test
just stream-test <target>     # Debug test output

# Format/Lint
just format                   # clang-format + Python
just clippy jsg-macros        # Rust clippy
just clang-tidy //target      # C++ tidy

# Debug
just build-asan               # AddressSanitizer build
just test-asan                # ASan tests
just compile-commands         # For clangd
```

## TEST PATTERNS

| Type           | File Pattern         | Runner           |
| -------------- | -------------------- | ---------------- |
| C++ unit       | `*-test.c++`         | KJ_TEST macro    |
| JS integration | `*.wd-test` + `*.js` | `workerd test`   |
| Node compat    | `*-nodejs-test.*`    | `just node-test` |
| WPT            | `src/wpt/*-test.ts`  | `just wpt-test`  |
| Benchmarks     | `bench-*.c++`        | `just bench`     |

Test variants auto-generated: `name@`, `name@all-compat-flags`, `name@all-autogates`

## RISKY CHANGES

Use **autogates** (`src/workerd/util/autogate.h`) for gradual rollout:

```cpp
if (Autogate::isEnabled(AutogateKey::YOUR_FEATURE)) { ... }
```

Use **compat flags** (`compatibility-date.capnp`) for behavior changes tied to dates.

## GOTCHAS

- **No Cargo.toml**: Rust managed entirely via Bazel
- **Split Node impl**: C++ in `api/node/`, TS in `src/node/`
- **TS→Cap'n Proto→C++**: JS modules embedded via Cap'n Proto into binary
- **One `V8System` per process**: JSG isolate constraint
- **Test size matters**: `enormous` excluded by default
- **External Cap'n Proto**: KJ library in `external/capnp-cpp`
- **BUILD.\* files**: `BUILD.zlib`, `BUILD.sqlite3`, etc. for external deps

## DEBUGGING

- V8 inspector: `--inspector` flag
- Request tracing: `src/workerd/io/trace.c++`
- Memory: Implement `JSG_MEMORY_INFO` for heap snapshots
- GC: Must implement `visitForGc()` for C++ objects holding JS refs

## COMPLEXITY HOTSPOTS

| File                       | Lines | Role                 |
| -------------------------- | ----- | -------------------- |
| `server/server.c++`        | 5883  | Server orchestration |
| `io/worker.c++`            | 4638  | Worker lifecycle     |
| `api/streams/standard.c++` | 4284  | WHATWG Streams       |
| `io/actor-cache.c++`       | 3375  | DO storage cache     |
| `jsg/jsg.h`                | 3016  | V8 binding core      |
