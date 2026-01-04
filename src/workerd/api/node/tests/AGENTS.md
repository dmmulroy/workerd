# NODE.JS COMPAT TESTS

Integration tests for Node.js compatibility layer. ~89 files.

## FILE NAMING

| Pattern                         | Purpose                       |
| ------------------------------- | ----------------------------- |
| `<module>-nodejs-test.js`       | Primary Node.js compat test   |
| `<module>-test.js`              | General module test           |
| `crypto_<func>-test.js`         | Crypto subsystem (underscore) |
| `<module>-nodejs-tcp-server.js` | TCP sidecar server            |
| `<module>-nodejs-server.js`     | HTTP sidecar server           |
| `*.expected_stdout`             | Golden files for stdio tests  |

## SIDECAR PATTERN

For tests requiring real TCP/HTTP endpoints:

```starlark
js_binary(
    name = "net-nodejs-tcp-server",
    entry_point = "net-nodejs-tcp-server.js",
)

wd_test(
    src = "net-nodejs-test.wd-test",
    sidecar = "net-nodejs-tcp-server",
    sidecar_port_bindings = ["SERVER_PORT", "ECHO_SERVER_PORT"],
)
```

**Server conventions:**

- Use `process.env.SIDECAR_HOSTNAME` for bind address
- Use `process.env.<PORT_NAME>` for ports
- Call `reportPort(server)` to signal readiness

## FIXTURES

`fixtures/` contains crypto keys (46 `.pem` files):

- RSA (2048, 4096, PKCS1, PKCS8, PSS)
- EC (P-256, P-384, P-521, secp256k1)
- EdDSA (Ed25519, Ed448, X25519, X448)
- TLS certificates

**Usage in .wd-test:**

```capnp
bindings = [
  ( name = "rsa_private.pem", text = embed "fixtures/rsa_private.pem" ),
],
```

## TEST EXPORT PATTERN

```javascript
export const test_<name> = {
  test(ctrl, env, ctx) { ... },
};
```

## SIZE ANNOTATIONS

- Default: small
- `size = "large"`: streams, crypto_scrypt, tls, net
- `size = "enormous"`: crypto_dh

All tests use `args = ["--experimental"]` by default.
