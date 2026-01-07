# UTIL KNOWLEDGE BASE

Runtime utilities for type-safety, feature flags, SQLite, threading.

## OVERVIEW

Core utilities used throughout workerd: type-safe state machines, runtime feature flags (autogates), SQLite wrapper, UUID generation, threading primitives.

## STATE MACHINE

`StateMachine<Specs..., States...>` - Type-safe wrapper over `kj::OneOf` preventing UAF bugs.

**Key Problem**: Raw `kj::OneOf` allows use-after-free when callbacks trigger state transitions while holding references.

**Solution**:

- `whenState<S>([](S& s) {...})` - Locks transitions during callback, throws if attempted
- `deferTransitionTo<S>(...)` - Queue transition until lock released
- `TerminalStates<Closed, Error>` - Enforce states never transition back
- `scopedOperation()` - RAII lock for multi-step operations

**Usage**:

```cpp
StateMachine<TerminalStates<Closed, Error>, Readable, Closed, Error> state;
state.whenState<Readable>([](Readable& r) {
  r.read();  // Safe - transition blocked during callback
});
```

See `state-machine.h:1-2148` for full pattern details and examples.

## AUTOGATE

Runtime feature flags for gradual rollout of risky changes, independent of binary releases.

**Usage**:

```cpp
if (Autogate::isEnabled(AutogateKey::MY_FEATURE)) {
  // New code path
}
```

**Adding gate**: Edit `AutogateKey` enum in `autogate.h`, add to `KJ_STRINGIFY()` in `autogate.c++`, initialize via config or `--all-autogates`.

## KEY UTILITIES

- **SQLite** (`sqlite.h/c++`) - KJ filesystem-backed SQLite wrapper, supports in-memory FS for tests, metrics via `SqliteObserver`
- **UUID** (`uuid.h`) - UUID generation
- **Thread Scopes** (`thread-scopes.h`) - Threading utilities
