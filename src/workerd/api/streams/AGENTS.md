# STREAMS

ReadableStream/WritableStream/TransformStream. Dual implementation architecture.

## OVERVIEW

Two parallel implementations:

- **Internal** (`*InternalController`): kj-backed, byte-only, single pending read
- **Standard** (`*JsController`): JS promise-based, value/byte, WHATWG-compliant

## KEY FILES

| File           | Lines | Purpose                             |
| -------------- | ----- | ----------------------------------- |
| `standard.c++` | 4284  | WHATWG stream controllers           |
| `internal.c++` | 2331  | kj-backed stream plumbing           |
| `readable.h`   |       | `ReadableStream` class              |
| `writable.h`   |       | `WritableStream` class              |
| `transform.h`  |       | `TransformStream` class             |
| `common.h`     |       | Shared types, controller interfaces |
| `queue.h`      |       | Internal queue implementations      |

## CONTROLLER HIERARCHY

```
ReadableStream
└── ReadableStreamController (interface)
    ├── ReadableStreamInternalController  # kj::AsyncInputStream
    └── ReadableStreamJsController        # JS promise chains
        ├── ReadableStreamDefaultController
        └── ReadableByteStreamController

WritableStream
└── WritableStreamController (interface)
    ├── WritableStreamInternalController  # kj::AsyncOutputStream
    └── WritableStreamJsController        # JS promise chains
```

## LOCK STATES

Streams have lock states: `Unlocked`, `Locked`, `ReaderLocked`, `WriterLocked`, `PipeLocked`

## PIPE STRATEGIES

`tryPipeFrom()` selects optimal path:

1. **kj→kj**: Direct async copy (fastest)
2. **JS→JS**: Promise chain
3. **kj→JS** / **JS→kj**: Bridge layer

## COMPAT FLAGS

| Flag                                           | Effect                             |
| ---------------------------------------------- | ---------------------------------- |
| `streamsJavaScriptControllers`                 | Enable JS controllers (2022-11-30) |
| `transformStreamInitializesFullyErroredOutput` | TS compat                          |

## ANTI-PATTERNS

- Internal streams: Only ONE pending read at a time
- `removeSink()`: Deprecated - use `detach()`
- TransformStream pump: See HACK in `internal.c++:291`

## TESTING

```bash
just test //src/workerd/api/tests:streams-test
```
