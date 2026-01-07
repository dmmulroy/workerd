# SERVER KNOWLEDGE BASE

Entry: `workerd.c++` (CliMain), orchestrator: `server.c++` (5883 lines).

## CLI

```bash
workerd serve config.capnp        # Run server
workerd compile config.capnp -o bin  # Self-compiling binary
workerd test config.capnp         # Run .wd-test tests
```

Options: `--watch` (dev), `--socket-addr`, `--verbose`, `--experimental`

## CONFIG SCHEMA (`workerd.capnp`)

### Top-level (Config)

- `services` - Named services (worker, network, external, disk)
- `sockets` - HTTP/HTTPS listeners
- `v8Flags`, `extensions`, `autogates`, `logging`

### Worker Config

- `modules` - esModule, commonJsModule, pythonModule, wasm, json, text
- `bindings` - Capability-based resource access
- `compatibilityDate` - 153+ behavior flags
- `globalOutbound` - Where `fetch()` routes (default: "internet")
- `durableObjectNamespaces` - DO classes (uniqueKey/ephemeralLocal)

## SERVICE TYPES

| Type       | Purpose                                      |
| ---------- | -------------------------------------------- |
| `worker`   | JS/WASM/Python worker                        |
| `network`  | Network access (allow: public/private/local) |
| `external` | Proxy to remote HTTP server                  |
| `disk`     | Static file serving                          |

## BINDING TYPES (27+)

**Data**: `text`, `data`, `json`, `wasmModule`, `cryptoKey`
**Service**: `service`, `durableObjectClass`, `durableObjectNamespace`
**Storage**: `kvNamespace`, `r2Bucket`, `queue`, `analyticsEngine`, `memoryCache`
**Advanced**: `hyperdrive`, `wrapped`, `workerLoader`, `fromEnvironment`, `unsafeEval`

## KEY PATTERNS

- **Capability security** - Zero default privileges, explicit bindings
- **Self-compiling** - `compile` embeds config+code into standalone binary
- **Watch mode** - `--watch` hot-reloads on file change (inotify/kqueue)

## KEY FILES

| File            | Purpose                |
| --------------- | ---------------------- |
| `workerd.c++`   | CLI entry, subcommands |
| `server.c++`    | Server class, routing  |
| `workerd.capnp` | Main config schema     |
