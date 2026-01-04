# I/O SUBSYSTEM

Request lifecycle, actor storage, worker management. ~39k lines.

## KEY FILES

| File                       | Purpose                                              |
| -------------------------- | ---------------------------------------------------- |
| `io-context.h`             | **Central I/O context** - request scope (1662 lines) |
| `worker.c++`               | Worker lifecycle, V8 isolate management (4638 lines) |
| `actor-cache.c++`          | Durable Object caching (3375 lines)                  |
| `actor-sqlite.c++`         | SQLite-backed actor storage                          |
| `trace.c++`                | Distributed tracing (1705 lines)                     |
| `compatibility-date.capnp` | Feature flags schema                                 |
| `worker-interface.h`       | Core service interface                               |

## IoContext

Every request runs in an `IoContext`. Access via:

```cpp
IoContext& ctx = IoContext::current();
```

Manages:

- Request limits and timeouts
- Async task scheduling
- Output gates
- Tracing spans
- Capability access

## ACTOR CACHING

`ActorCache` provides:

- Consistent caching over SQLite
- Transaction handling
- Output gate coordination (prevents stale reads)

Hierarchy:

- `ActorCacheOps` → `ActorCacheInterface` → `ActorCache`
- `ActorSqlite` for SQLite backend

## COMPATIBILITY FLAGS

Defined in `compatibility-date.capnp`. Runtime access:

```cpp
auto& flags = FeatureFlags::get(lock);
if (flags.getStreamsJavaScriptControllers()) { ... }
```

Annotations:

- `$compatEnableFlag("name")` - Enable via flag
- `$compatEnableDate("YYYY-MM-DD")` - Enable by date
- `$experimental` - Requires --experimental

## OBSERVERS

| Class             | Purpose                |
| ----------------- | ---------------------- |
| `RequestObserver` | Request lifecycle      |
| `IsolateObserver` | V8 isolate metrics     |
| `WorkerObserver`  | Worker lifecycle       |
| `ActorObserver`   | Durable Object metrics |
| `SpanObserver`    | Tracing spans          |

## ANTI-PATTERNS

| Pattern                 | Why                                            |
| ----------------------- | ---------------------------------------------- |
| `awaitIoLegacy()`       | Deprecated - use `awaitIo()` with continuation |
| Store KJ I/O in JS heap | Must use `IoOwn` wrapper                       |
| Modify outcome.capnp    | Sync with authoritative version                |

## WORKER LIFECYCLE

1. `Worker::create()` - Create worker instance
2. `Worker::Lock` - Enter JS isolate
3. `IoContext_IncomingRequest` - Handle request
4. Output gate release - Allow dependent reads
