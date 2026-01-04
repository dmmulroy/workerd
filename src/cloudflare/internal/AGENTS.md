# CLOUDFLARE INTERNAL APIs

Cloudflare-specific service bindings (D1, R2, AI, etc.). TypeScript implementations.

## STRUCTURE

```
internal/
├── d1-api.ts          # D1 database binding
├── ai-api.ts          # Workers AI binding
├── vectorize-api.ts   # Vectorize binding
├── pipeline-transform.ts
├── aig-*.ts           # AI Gateway
├── images-api.ts      # Images binding
├── test/              # Test infrastructure
│   ├── d1/            # D1 tests + mocks
│   ├── ai/            # AI tests + mocks
│   └── ...
```

## MOCK PATTERN

Each binding has mock implementation for tests:

```
test/<api>/
├── <api>-api-test.wd-test   # Test config
├── <api>-api-test.js        # JS test cases
├── <api>-api-test.py        # Python test cases (optional)
└── <api>-mock.js            # Mock backend (Durable Object)
```

## TRACING INSTRUMENTATION

All bindings use consistent tracing:

```typescript
withSpan(name, fn); // Wraps async operations
// Span attributes follow OpenTelemetry conventions:
// db.system.name, db.operation.name, cloudflare.*
```

## BINDING-AS-FETCHER

Bindings expose via `Fetcher` interface:

```typescript
makeBinding(env: { fetcher: Fetcher })
// Session state via custom headers (x-cf-d1-session-commit-token)
```

## TYPE DECLARATIONS

- Implementation: `*-api.ts`
- Types: matching `*.d.ts`
- Internal module specifier: `cloudflare-internal:*`

## TESTS

```bash
just test //src/cloudflare/internal/test/d1:d1-api-test
```

Python tests use `py_wd_test.bzl` macro.
