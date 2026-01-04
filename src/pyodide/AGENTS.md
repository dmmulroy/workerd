# PYODIDE (Python Workers)

Python runtime integration via Pyodide (WebAssembly-compiled CPython).

## STRUCTURE

```
pyodide/
├── python-entrypoint.js       # USER module entry point
├── python-entrypoint-helper.ts # BUILTIN module (actual init logic)
├── internal/
│   ├── python.ts              # Pyodide loading, Emscripten instantiation
│   ├── snapshot.ts            # Memory snapshot create/restore
│   ├── metadata.ts            # Compat flags, requirements exposure
│   ├── setupPackages.ts       # Site-packages virtualization
│   ├── loadPackage.ts         # Package loading from artifact bundler
│   ├── tar.ts, tarfs.ts       # TAR filesystem
│   ├── pool/                  # Isolate pool, Emscripten setup
│   │   └── builtin_wrappers.ts # Patched builtins (crypto, setTimeout)
│   ├── topLevelEntropy/       # Deterministic startup entropy
│   │   └── entropy_patches.py # Python-side entropy patches
│   └── workers-api/           # Python Workers SDK
│       └── src/workers/_workers.py  # Idiomatic Python API
├── types/                     # TypeScript declarations
├── BUILD.bazel
└── helpers.bzl                # python_bundles, pyodide_extra macros
```

## MODULE VISIBILITY TIERS

```
USER modules  →  python-entrypoint.js    →  Can import BUILTIN
BUILTIN       →  python-entrypoint-helper.ts  →  Can import INTERNAL
INTERNAL      →  pyodide-internal:*      →  Not user-visible
```

## SNAPSHOT SYSTEM

| Type      | Purpose               | Cold Start |
| --------- | --------------------- | ---------- |
| Baseline  | Core stdlib           | Slowest    |
| Package   | + user dependencies   | Medium     |
| Dedicated | + top-level execution | Fastest    |

Set via: `use_snapshot = "stacked"` in Bazel

## PYTHON SDK PATTERNS

```python
# Entrypoint classes (discovered via introspection)
class MyDO(DurableObject): ...
class MyWorker(WorkerEntrypoint): ...
class MyWorkflow(WorkflowEntrypoint): ...

# Legacy handler pattern
def on_fetch(request, env, ctx): ...
```

## KEY FILES

| File                                           | Purpose                  |
| ---------------------------------------------- | ------------------------ |
| `internal/python.ts`                           | Main Pyodide loading     |
| `internal/snapshot.ts`                         | Snapshot save/restore    |
| `internal/workers-api/src/workers/_workers.py` | Request, Response, fetch |
| `internal/topLevelEntropy/lib.ts`              | Entropy patching         |
| `helpers.bzl`                                  | `python_bundles` macro   |

## ANTI-PATTERNS

| Pattern                                   | Why                             |
| ----------------------------------------- | ------------------------------- |
| Import `pyodide-internal:` from user code | Route through BUILTIN           |
| `crypto.getRandomValues()` at top-level   | Throws; patched at request time |
| Module named `src.*`                      | Common mistake; helpful error   |

## COMPAT FLAGS

- `python_workflows` - Workflow entrypoints
- `python_no_global_handlers` - Class-only entrypoints
- `python_dedicated_snapshot` - Dedicated snapshots
- `enable_python_external_sdk` - Skip built-in SDK

## BUILD

Pyodide is patched at build time:

- `var _createPyodideModule` → `export const _createPyodideModule`
- `new WebAssembly.Module` → custom wasm loading
- `crypto.getRandomValues(` → controlled entropy

## C++ INTEGRATION

`src/workerd/api/pyodide/`:

- `pyodide.h` - PyodideMetadataReader, ArtifactBundler
- `setup-emscripten.c++` - EmscriptenRuntime init
- `requirements.c++` - Transitive dependency resolution
