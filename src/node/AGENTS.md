# NODE.JS COMPATIBILITY (TypeScript Layer)

TypeScript implementations of Node.js built-in modules. ~56 modules.

## OVERVIEW

Split implementation:

- **C++ layer**: `src/workerd/api/node/` - Native bindings
- **TS layer**: `src/node/` (this dir) - JS/TS APIs

## STRUCTURE

```
node/
├── internal/          # Internal implementations (108 files)
│   ├── buffer.ts
│   ├── crypto.ts
│   ├── fs.ts
│   └── ...
├── *.ts               # Public module exports (56 files)
│   ├── buffer.ts      # → exports from internal/buffer.ts
│   ├── crypto.ts
│   └── ...
└── BUILD.bazel        # wd_ts_bundle target
```

## CONVENTIONS

- Public `node/<module>.ts` re-exports from `internal/<module>.ts`
- Use `#` private fields (not `private` keyword)
- Explicit return types required
- Unused vars prefix with `_`
- `noUncheckedIndexedAccess: true`

## BUILD

Bundled via `wd_ts_bundle` → Cap'n Proto → embedded in C++ binary.

## ADDING NEW MODULE

1. Create `internal/<module>.ts` with implementation
2. Create `<module>.ts` with public exports
3. Add to `BUILD.bazel` `internal_modules` list
4. Add C++ bindings in `src/workerd/api/node/` if needed

## TESTING

```bash
just node-test <module>   # e.g., just node-test zlib
```

Test files: `src/workerd/api/node/tests/*-nodejs-test.{wd-test,js}`

## KEY MODULES

| Module                | Notes                          |
| --------------------- | ------------------------------ |
| `buffer`              | `Buffer` polyfill              |
| `crypto`              | Web Crypto + Node APIs         |
| `fs`                  | Limited filesystem (worker-fs) |
| `stream`              | Node stream compat             |
| `path`, `url`, `util` | Standard utilities             |

## C++ COUNTERPARTS

For modules requiring native code, C++ lives in `src/workerd/api/node/`:

- `crypto.c++` - Crypto primitives
- `buffer.c++` - Buffer internals
- `zlib-util.c++` - Compression

Types flow: **TS → Cap'n Proto bundle → C++ embed → Runtime**
