# PYODIDE RUNTIME AGENTS

Python Workers runtime via Pyodide/Emscripten/WASM. See `/src/workerd/AGENTS.md` for build/test commands.

## OVERVIEW

Entry: `python-entrypoint.js` → `python-entrypoint-helper.ts` (BUILTIN) → `internal/python.ts`
Emscripten bootstraps in isolated context (`setup-emscripten.c++`, `pool/emscriptenSetup.ts`)
Module namespaces: `pyodide:`, `pyodide-internal:`

## SNAPSHOT SYSTEM

**Three snapshot types:**

- **Baseline**: Universal Pyodide + stdlib. Created offline (`make_snapshots.py`), shipped as artifact
- **Package**: Baseline + worker requirements. Created during validation if `requirements` specified
- **Dedicated**: Package + top-level code execution. Created if dedicated worker, restored per-request

**Wire format** (ArtifactBundler):

- Header (8 bytes): magic + version + metadata length
- JSON metadata: `SnapshotMeta` (dsoHandles, loadOrder, hiwire state, jsModules, settings)
- HEAP8 snapshot: Linear memory dump from `Module.HEAP8`

**Key files:**

- `internal/snapshot.ts` - `maybeRestoreSnapshot()`, `maybeCollectSnapshot()`, `finalizeBootstrap()`
- `internal/python.ts:prepareWasmLinearMemory()` - Restoration entrypoint
- `pyodide_extra.capnp:PythonSnapshotRelease` - Version/hash tracking

**Memory restoration:**

- Check `MEMORY_SNAPSHOT_READER !== undefined`
- Parse metadata: `readSnapshotMetadata()`
- Restore HEAP8: `restoreMemory(Module, meta)`
- Replay dso loads: `restoreDsoHandles(Module, dsoHandles, loadOrder)`
- Restore hiwire refs: `applySnapshotConfig(Module.API.public_api.hiwire_store, hiwireConfig)`
- Deserialize JS modules: `deserializeJsModule()`

## MODULE LOADING

**Package loading** (`setupPackages.ts`):

- Embedded packages: `EmbeddedPackagesTarReader` reads from bundled tar
- Virtual filesystem: `VirtualizedDir` overlays packages onto `/lib/python3.12/site-packages`
- Dynamic libraries: Mounted to `/usr/lib`, loaded via `dlopen()`
- Tar parsing: `internal/tar.ts` parses tar headers, creates `TarFSInfo` tree

**Worker files** (`mountWorkerFiles()`):

- User code mounted to `/session` (writable), `/session/metadata` (read-only via `metadatafs.ts`)
- `sys.path` adjusted: `/session` → stdlib → `/session/metadata/vendor` → site-packages

**Entrypoint proxying** (`python-entrypoint-helper.ts`, `serializeJsModule.ts`):

- Python class exported via `default` becomes JS `WorkerEntrypoint`/`DurableObject`
- JS Proxy intercepts method calls: `handler.fetch()` → `pyHandler.fetch()`
- Lifecycle: `pyodide_entrypoint_helper.doAnImport("__main__")` → snapshot → per-request `beforeRequest()`

## WORKER LIFECYCLE

1. **Bootstrap** (once per isolate):
   - `loadPyodide()` → `SetupEmscripten` instantiates WASM in separate context
   - `prepareWasmLinearMemory()` → restore snapshot or cold boot
   - `setupPythonSearchPath()` → finalize `sys.path`
2. **Package load** (if requirements):
   - `loadPackages(TRANSITIVE_REQUIREMENTS)` → `mountOverlay()` → `dlopen()` .so files
3. **Top-level execution** (once per worker):
   - `doAnImport("__main__")` → runs user code
   - Dedicated snapshot collected if applicable
4. **Per-request**:
   - `beforeRequest()` → `entropyBeforeRequest()` (RNG reset), `clearSignals()` (CPU limit)
   - Handler invoked via entrypoint proxy

**CPU limiting**: SIGXCPU signal raised when near limit (`setCpuLimitNearlyExceededCallback()`)
