# BUILD SYSTEM

Custom Bazel rules and macros for workerd.

## KEY RULES

| Rule            | File                | Purpose                                   |
| --------------- | ------------------- | ----------------------------------------- |
| `wd_test`       | `wd_test.bzl`       | Integration tests with `.wd-test` configs |
| `kj_test`       | `kj_test.bzl`       | C++ unit tests with KJ framework          |
| `wd_cc_embed`   | `wd_cc_embed.bzl`   | C23 `#embed` for binary embedding         |
| `wd_js_bundle`  | `wd_js_bundle.bzl`  | TS→Cap'n Proto→C++ embedding              |
| `wpt_test`      | `wpt_test.bzl`      | Web Platform Test integration             |
| `wd_cc_binary`  | `wd_cc_binary.bzl`  | Binary with cross-compile support         |
| `wd_rust_crate` | `wd_rust_crate.bzl` | Rust library via Bazel                    |

## TEST VARIANTS

`wd_test` auto-generates 3 variants:

- `name@` - Oldest compat date (2000-01-01)
- `name@all-compat-flags` - Newest date (2999-12-31)
- `name@all-autogates` - All autogates enabled

## SIDECAR SUPPORT

Tests can spawn helper processes:

```python
wd_test(
    name = "my-test",
    sidecar = "helper-server",
    sidecar_port_bindings = ["PORT"],
)
```

## EXTERNAL DEPS

`deps/` contains MODULE.bazel fragments:

- `deps/rust/` - Rust crate BUILD files
- `build/deps/*.MODULE.bazel` - Dep declarations

## BUILD FILES

External deps get custom BUILD files: `BUILD.sqlite3`, `BUILD.zlib`, `BUILD.pyodide`, etc.

## QUIRKS

- `run_binary_target`: Workaround for bazel#14848 (target vs exec config)
- Windows: `--nowindows_enable_symlinks` pre-build for TS
- CI: Files >100MB trimmed from cache
