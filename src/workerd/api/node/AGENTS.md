# NODE.JS COMPAT (C++ Layer)

C++ native bindings for Node.js compatibility. ~33 files.

## OVERVIEW

Split implementation with TypeScript layer (`src/node/`):

- **This dir**: C++ bindings requiring native code
- **src/node/**: TypeScript/JS implementations

## KEY FILES

| File            | Purpose                             |
| --------------- | ----------------------------------- |
| `buffer.c++`    | Buffer internal bindings            |
| `crypto.c++`    | Node crypto primitives (1643 lines) |
| `zlib-util.c++` | Compression (zlib, brotli, gzip)    |
| `i18n.c++`      | ICU bindings                        |

## CONVENTIONS

- Classes extend `jsg::Object`
- Use `JSG_RESOURCE_TYPE` for JS-exposed types
- Node module name in `JSG_TS_ROOT` annotations

## TESTS

Tests in `tests/` subdirectory. See `tests/AGENTS.md` for patterns.

## COMPAT FLAGS

Common combinations:

```capnp
compatibilityFlags = ["nodejs_compat", "nodejs_compat_v2"]
compatibilityFlags = ["nodejs_compat", "nodejs_compat_v2", "enable_nodejs_http_modules"]
compatibilityFlags = ["nodejs_compat", "nodejs_compat_v2", "enable_nodejs_fs_module"]
```

## ADDING BINDINGS

1. Create `<module>.{h,c++}` here
2. Create TypeScript wrapper in `src/node/internal/<module>.ts`
3. Register in parent's `BUILD.bazel`
4. Add tests in `tests/`
