# WPT (Web Platform Tests)

WHATWG/W3C test suite integration for spec compliance.

## STRUCTURE

```
wpt/
├── harness/                # Custom WPT harness
│   ├── harness.ts          # Main runner, config parsing
│   ├── test.ts             # test(), async_test(), promise_test()
│   ├── assertions.ts       # assert_* functions
│   ├── utils.ts            # subsetTestByKey, step_timeout
│   ├── globals.ts          # Browser-like globals
│   └── common.ts           # FilterList, getHostInfo()
├── BUILD.bazel             # wpt_test() invocations
├── *-test.ts               # Per-module test configs
└── tsconfig.json
```

## ADDING WPT TESTS

### 1. Add to BUILD.bazel:

```python
wpt_test(
    name = "url",
    config = "url-test.ts",
    wpt_directory = "@wpt//:url@module",
    # Optional:
    size = "large",
    start_server = True,  # For HTTP tests
)
```

### 2. Create config (`*-test.ts`):

```typescript
import { type TestRunnerConfig } from 'harness/harness';

export default {
  'test-file.any.js': {}, // Run all subtests

  'failing-test.any.js': {
    comment: 'Explanation required',
    expectedFailures: ['exact subtest name', /regex pattern/],
  },

  'skip-test.any.js': {
    comment: 'Reason for skip',
    disabledTests: true, // Skip entire file
  },
} satisfies TestRunnerConfig;
```

## CONFIG OPTIONS

| Option             | Type                         | Purpose                             |
| ------------------ | ---------------------------- | ----------------------------------- |
| `comment`          | string                       | **Required** with failures/disabled |
| `expectedFailures` | `(string\|RegExp)[]`         | Known failing subtests              |
| `disabledTests`    | `(string\|RegExp)[] \| true` | Skip subtests/file                  |
| `omittedTests`     | `(string\|RegExp)[] \| true` | Exclude from stats                  |
| `verbose`          | boolean                      | Log subtest progress                |
| `only`             | boolean                      | Debug: run only this                |
| `replace`          | `(code) => string`           | Transform source                    |

## COMMANDS

```bash
just wpt-test              # All WPT tests
just wpt-test url          # Specific module
just wpt-test dom/abort    # Nested module

# Generate config template
GEN_TEST_CONFIG=1 bazel test //src/wpt:url

# Generate JSON report
GEN_TEST_REPORT=1 bazel test //src/wpt:url
```

## HARNESS GLOBALS

### Test Functions

- `test(fn, name)` - Sync test
- `async_test(fn, name)` - Async (call `t.done()`)
- `promise_test(fn, name)` - Promise-based

### Assertions

- `assert_equals`, `assert_not_equals`
- `assert_true`, `assert_false`
- `assert_array_equals`, `assert_object_equals`
- `assert_throws_js(ctor, fn)`
- `assert_throws_dom(type, fn)`

### Utilities

- `step_timeout(fn, ms)`
- `get_host_info()` - HTTP_ORIGIN, etc.
- `token()` - Random UUID

## CONVENTIONS

- `comment` required when using `expectedFailures`/`disabledTests`
- Use regex for groups: `/request\.formData/`
- `disabledTests: true` skips entire file
- Unexpected success = test failure (must update config)
