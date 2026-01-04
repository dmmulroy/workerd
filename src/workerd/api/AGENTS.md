# API LAYER

JS runtime APIs exposed to Workers. Largest code surface (~47k lines).

## STRUCTURE

```
api/
├── crypto/        # Web Crypto + Node crypto → see crypto/AGENTS.md
├── node/          # Node.js compat C++ → see node/AGENTS.md
├── streams/       # ReadableStream/WritableStream → see streams/AGENTS.md
├── tests/         # .wd-test + .js test pairs
├── pyodide/       # Python runtime hooks
├── *.h/*.c++      # Individual APIs (http, websocket, sql, etc.)
└── BUILD.bazel
```

## KEY FILES

| File              | Purpose                                                 |
| ----------------- | ------------------------------------------------------- |
| `workerd-api.c++` | **API registration hub** - 66 includes, ties everything |
| `http.c++`        | Fetch/Request/Response (2504 lines)                     |
| `global-scope.h`  | Worker global scope, event handlers                     |
| `web-socket.c++`  | WebSocket implementation                                |
| `sql.c++`         | D1/SQL API surface                                      |

## ADDING NEW APIs

1. Create `<api>.h` / `<api>.c++`
2. Use `JSG_RESOURCE_TYPE` for classes
3. Register in `workerd-api.c++`
4. Add tests in `tests/` (`.wd-test` + `.js`)

## JSG PATTERNS USED HERE

```cpp
// Typical API class
class Foo: public jsg::Object {
public:
  JSG_RESOURCE_TYPE(Foo) {
    JSG_METHOD(bar);
    JSG_PROTOTYPE_PROPERTY(baz);  // Not JSG_INSTANCE_PROPERTY
  }
};
```

## ANTI-PATTERNS

- **New Fetcher methods**: Must be gated via compat flag
- **Header guard enums**: Numeric values are serialized - never change
- **Property named "then"**: V8 treats as thenable
- **JSG_INSTANCE_PROPERTY**: Usually wrong (use PROTOTYPE)

## TESTS

```bash
just test //src/workerd/api/tests:http-test
just stream-test //src/workerd/api/tests:streams-test
```

Test pattern: `<api>-test.wd-test` + `<api>-test.js`
