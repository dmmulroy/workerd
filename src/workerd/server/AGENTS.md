# SERVER

Main binary entry point and configuration. ~19k lines.

## KEY FILES

| File              | Purpose                                                 |
| ----------------- | ------------------------------------------------------- |
| `workerd.c++`     | **Main entry** - `int main()` at line 1746              |
| `server.c++`      | Server orchestration - 70 internal classes (5883 lines) |
| `workerd.capnp`   | Configuration schema                                    |
| `workerd-api.c++` | API registration hub                                    |

## ENTRY POINT

```cpp
// src/workerd/server/workerd.c++:1746
int main(int argc, char* argv[]) {
  return kj::runMainAndExit(ctx, CliMain(ctx), argc, argv);
}
```

`CliMain` provides subcommands: `serve`, `test`, `compile`

## CONFIGURATION

Cap'n Proto format (NOT YAML/JSON):

```capnp
using Workerd = import "/workerd/workerd.capnp";
const config :Workerd.Config = (
  services = [ ... ],
  sockets = [ ... ]
);
```

## SERVER.C++ INTERNALS

70+ internal service classes:

- `ExternalService`, `NetworkService`, `DiskService`
- `WorkerService`, `ActorService`
- HTTP listeners, TLS, Inspector

## ANTI-PATTERNS

| Pattern                      | Why                         |
| ---------------------------- | --------------------------- |
| Wasm in ServiceWorker syntax | Deprecated - use ES modules |
| Lose encryption keys         | Data may be unrecoverable   |

## COMMANDS

```bash
workerd serve config.capnp       # Run server
workerd test config.capnp        # Run tests
workerd compile config.capnp     # AOT compile
```
