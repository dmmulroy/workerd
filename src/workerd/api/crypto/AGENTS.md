# CRYPTO

Web Crypto + Node.js crypto APIs. ~9.6k lines, 33 files.

## STRUCTURE

```
crypto/
├── crypto.h           # PUBLIC API: Crypto, SubtleCrypto, CryptoKey, DigestStream
├── impl.h             # INTERNAL ONLY - don't include outside crypto/*.c++
├── keys.h             # AsymmetricKeyCryptoKeyImpl base
├── Algorithm impls:
│   ├── aes.c++        # AES-GCM/CBC/CTR/KW (anonymous namespace classes)
│   ├── rsa.{h,c++}    # RSA helper
│   ├── ec.{h,c++}     # EC helper
│   └── dh.{h,c++}     # Diffie-Hellman (Node.js)
├── Key derivation:
│   ├── kdf.h          # Signatures only
│   ├── hkdf.c++, pbkdf2.c++, scrypt.c++
├── Utilities:
│   ├── jwk.{h,c++}    # JWK via ncrypto
│   ├── x509.{h,c++}   # X509Certificate
│   └── crc-impl.{h,c++}
```

## HEADER EXPOSURE

- `crypto.h` = public (included by workerd-api.c++)
- `impl.h` = internal (guards: "Don't include unless your name is crypto\*.c++")
- Algorithm `.c++` files use anonymous namespace classes (not in headers)

## KEY TYPE HIERARCHY

```cpp
CryptoKey::Impl                    # Base - static import/generate ptrs
  └── AsymmetricKeyCryptoKeyImpl   # Shared RSA/EC/EdDSA logic
      └── (algorithm-specific)     # In anonymous namespaces
```

## OPENSSL PATTERNS

```cpp
// Auto-throw on failure
OSSLCALL(EVP_EncryptInit_ex(...));

// RAII ownership
#define OSSL_NEW(T, ...) OSSLCALL_OWN(T, T##_new(...))

// Error stack management
ClearErrorOnReturn clearErrorOnReturn;
MarkPopErrorOnReturn markPop;
```

## SECURITY

```cpp
// Key material cleanup
class ZeroOnFree { ~ZeroOnFree() { OPENSSL_cleanse(...); } };

// Timing-safe compare
CRYPTO_memcmp(a, b, size)  // Not memcmp()
```

## ALGORITHM REGISTRY

```cpp
static const std::set<CryptoAlgorithm> ALGORITHMS = {
  {"AES-CTR"_kj, &CryptoKey::Impl::importAes, ...},
  // ...
};
// Fallback: Worker::Api::current().getCryptoAlgorithm(name)
```

## TESTS

```bash
just test //src/workerd/api/crypto:impl-test
just test //src/workerd/api/crypto:aes-test
```

C++ unit tests in `*-test.c++`; JS integration in `api/tests/crypto-*-test.*`
