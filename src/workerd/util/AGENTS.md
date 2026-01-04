# UTILITIES

Shared infrastructure: SQLite, state machines, threading, logging. ~16k lines.

## KEY FILES

| File              | Purpose                              |
| ----------------- | ------------------------------------ |
| `sqlite.c++`      | SQLite wrapper with KJ (2543 lines)  |
| `state-machine.h` | Type-safe state machine (2148 lines) |
| `autogate.h`      | Feature flags for gradual rollout    |
| `sentry.h`        | Logging macros                       |
| `weak-refs.h`     | Thread-safe weak references          |
| `stream-utils.h`  | Async stream helpers                 |

## AUTOGATE SYSTEM

For risky changes requiring gradual rollout:

```cpp
if (Autogate::isEnabled(AutogateKey::YOUR_FEATURE)) { ... }
```

Initialize ONCE at startup (before threads):

```cpp
initAutogate(provider);
```

Current keys: `V8_FAST_API`, `STREAMING_TAIL_WORKER`, `TAIL_STREAM_REFACTOR`, etc.

## STATE MACHINE

Type-safe state transitions:

```cpp
StateMachine<State, Lock> machine(State::Initial);
machine.transitionTo(State::Running);
// Terminal states cannot be transitioned FROM
```

**Gotchas**:

- `forceTo()` bypasses terminal protection
- `getUnsafe()` returns unlocked reference - can dangle

## LOGGING

```cpp
LOG_EXCEPTION("message", exception);
LOG_ONCE("one-time message");
LOG_PERIODICALLY("periodic", kj::SECONDS * 60);
LOG_NOSENTRY("not to sentry", details);
```

## OTHER UTILITIES

| File            | Purpose                               |
| --------------- | ------------------------------------- |
| `mimetype.h`    | MIME type parsing                     |
| `uuid.h`        | UUID generation                       |
| `http-util.h`   | HTTP helpers                          |
| `strong-bool.h` | Type-safe booleans (`WD_STRONG_BOOL`) |
| `wait-list.h`   | Promise-based synchronization         |
