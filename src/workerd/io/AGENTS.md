# I/O SUBSYSTEM

**Core responsibility:** Threading bridge, actor lifecycle, async ordering, resource limits.

## OVERVIEW

I/O layer bridges JS heap (thread-mobile) with I/O context (thread-pinned). Manages actor storage consistency via gates, handles worker lifecycle, enforces resource limits. Two fundamental patterns: gates for async ordering, caches for storage.

## KEY ABSTRACTIONS

**IoContext** (`io-context.h:52`) - Per-request execution context

- Bridge between JSG (thread-mobile) and I/O (thread-pinned)
- Owns timers, channels, traces, limit enforcers
- `IoContext_IncomingRequest` tracks per-request metrics/tracing (even in multi-request actors)
- `addWaitUntil()` for background tasks (DO NOT use `getWaitUntilTasks()`)

**Worker hierarchy** (`worker.h`)

- `Worker::Isolate` → `Worker::Script` → `Worker` instances
- `Worker::Actor` extends `Worker` with input/output gates, storage, hibernation
- Each actor pinned to single thread, JS heap can migrate

**RequestTracker** (`request-tracker.h:14`) - Hibernation readiness

- Tracks active request count via `ActiveRequest` RAII
- Hooks fire on 0→1 (`active()`) and N→0 (`inactive()`)
- Used by actors to determine hibernation eligibility

## GATE PATTERNS

**InputGate** (`io-gate.h:34`) - Blocks incoming events while storage ops pending

- `wait()` → `Lock` → release: serializes actor event delivery
- `CriticalSection`: nested lock hierarchy, failed CS breaks gate permanently
- Held during storage reads to prevent concurrent events from observing inconsistent state

**OutputGate** (`io-gate.h:241`) - Blocks outgoing messages until writes confirmed

- Prevents external observation of uncommitted writes
- ActorCache holds OutputGate lock during flush
- If flush fails, messages never sent → no premature confirmations

**Common mistake:** Awaiting inside lock without critical section → deadlock

## ACTOR STORAGE

**ActorCache** (`actor-cache.h:52`, `actor-cache.c++:44`)

- LRU cache over `rpc::ActorStorage::Stage::Client`
- Deferred writes: `put()` → dirty in cache → batched flush → OutputGate release
- `SharedLru` tracks size across all actors in process
- `noCache` option: read/write without caching (consistency preserved)
- `allowUnconfirmed`: OutputGate doesn't wait (risky!)

**ActorSqlite** (`actor-sqlite.h`)

- SQLite-backed storage (newer actors)
- Auto-batching via implicit transactions
- `getSqliteDatabase()` / `getSqliteKv()` for direct SQL access

**Transaction pattern:**

```cpp
auto txn = cache.startTransaction();
co_await txn.get(key);
txn.put(key, value);
co_await txn.commit();  // Writes to parent ActorCache
```

## GOTCHAS

- **NEVER** `awaitIoLegacy()` - refactor to `awaitIo()`
- **InputGate** locks must be acquired via `wait()`, not held across async boundaries without CS
- **OutputGate** automatically held by ActorCache flushes - don't double-lock
- **IoContext** is thread-pinned - don't capture across threads
- **ActorCache** size tracked globally via `SharedLru` - eviction can happen anytime
- **HibernationManager** (`hibernation-manager.h:19`): WebSocket hibernation, only for actors with accepted WebSockets
- **LimitEnforcer** (`limit-enforcer.h:29`): Per-request CPU/memory/subrequest limits, separate from IsolateLimitEnforcer

## THREADING MODEL

- **Isolate:** Can move between threads (JSG heap mobile)
- **IoContext:** Thread-pinned (KJ event loop, network, storage RPC)
- **Actor:** Thread-pinned once instantiated
- Lock order: Always JS lock → input gate → storage
