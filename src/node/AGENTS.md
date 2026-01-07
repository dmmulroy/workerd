# NODE.JS COMPATIBILITY LAYER

**Purpose:** Node.js API compatibility via hybrid TypeScript/C++ implementation

## ARCHITECTURE

```
src/node/                          # Public TS modules (node:*)
├── internal/                      # 108+ internal helpers (node-internal:*)
├── assert/, crypto/, fs/, ...     # Public module subdirs
└── *.ts                           # Top-level public modules

src/workerd/api/node/              # C++ implementations (JSG bindings)
```

**Import Pattern:**

```typescript
// Public module (node:buffer)
import { Buffer } from 'node-internal:internal_buffer';

// Internal implementation imports C++
import { default as bufferUtil } from 'node-internal:buffer'; // → C++
```

## KEY PATTERNS

### Hybrid Boundary

- **Public API** (`.ts`): Re-exports from `node-internal:*`, light glue code
- **Internal** (`internal/*.ts`): Business logic, error handling, transforms
- **C++ Core** (`src/workerd/api/node/*.c++`): Performance-critical/native APIs via JSG

### Symbol-Based State

```typescript
const kHandle = Symbol('kHandle');
const kState = Symbol('kState');
const kFinalized = Symbol('kFinalized');
```

### Error Hierarchy

```typescript
// internal/internal_errors.ts
NodeError / NodeTypeError / NodeRangeError;
// Codes: ERR_INVALID_ARG_TYPE, ERR_OUT_OF_RANGE, etc.
```

### Transform Inheritance

```typescript
// Streams, crypto, zlib use Transform base
class MyTransform extends Transform {
  _transform(chunk, encoding, callback) { ... }
}
```

### Progressive Stubbing

```typescript
// Unimplemented APIs throw with helpful message
export function notImplemented(name: string): never {
  throw new Error(`node:${name} is not yet implemented`);
}
```

## ADDING MODULES

1. **Create public module**: `src/node/mymodule.ts` → re-exports from internal
2. **Implement logic**: `src/node/internal/internal_mymodule.ts` or `.d.ts` stub
3. **C++ binding** (if needed): `src/workerd/api/node/mymodule.c++` with JSG macros
4. **Register**: Add to `src/workerd/server/workerd.capnp` module list
5. **Compat flags**: Use `nodejs_compat_v2` or add `nodejs_compat_mymodule`

### Compatibility Flags

- `nodejs_compat` - Base Node.js compat (v1)
- `nodejs_compat_v2` - Enhanced compat (default for new modules)
- Per-module: `nodejs_compat_streams`, `nodejs_compat_buffer`, etc.

## WHERE TO LOOK

| Task           | Location                               |
| -------------- | -------------------------------------- |
| Error classes  | `internal/internal_errors.ts`          |
| Buffer impl    | `internal/internal_buffer.ts`          |
| Crypto helpers | `internal/crypto_*.ts`                 |
| Streams base   | `internal/streams_*.ts`, `.d.ts` stubs |
| C++ bindings   | `../workerd/api/node/`                 |
| Validators     | `internal/validators.ts`               |
