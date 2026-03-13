# My KV Store Library

This is a high-performance concurrent key-value store for Rust.

## Development

- Run tests: `cargo test`
- Run lints: `cargo clippy -- -D warnings`
- Format code: `cargo fmt`

## Architecture

The core store lives in `src/store.rs` and uses `DashMap` for lock-free concurrent access.
TTL expiration is handled by a background sweep task in `src/expiry.rs`.

<!-- lexicon:begin:lexicon-context -->
## Lexicon Verification Context

This section is managed by `lexicon sync claude`. Do not edit manually.

# Project: my-kv-store
Domain: key-value store
Type: Library

## Contracts
- **Key-Value Store Contract** (key-value-store): Defines the behavioral guarantees of a concurrent in-memory key-value store supporting get, set, delete, and list operations. [status: Active, stability: Stable]
  Invariants:
  - inv-001: A key set with a value must return that exact value on subsequent get, until overwritten or deleted.
  - inv-002: After deleting a key, get must return None for that key.
  - inv-003: The store must be safe to access concurrently from multiple threads without data corruption.
  - inv-004: Keys with an expired TTL must not be returned by get or list.
  Required semantics:
  - req-001: get(key) returns None for keys that have never been set.
  - req-002: set(key, value) inserts or overwrites the value for the given key.
  - req-003: delete(key) removes the key and returns the previous value if present, or None.
  - req-004: list_keys() returns all currently stored, non-expired keys in unspecified order.
  - req-005: compare_and_swap(key, expected, new) atomically sets key to new only if the current value equals expected.
  Forbidden:
  - forbid-001: Must not panic on get or delete of a missing key.
  - forbid-002: Must not silently drop writes under contention.
  - forbid-003: Must not return expired entries from get() or list_keys().

## Scoring
Pass threshold: 80%, Warn threshold: 60%
- Correctness (weight: 30, Required)
- Conformance Coverage (weight: 25, Scored)
- Behavior Pass Rate (weight: 15, Scored)
- Lint Quality (weight: 10, Scored)
- Documentation Completeness (weight: 10, Advisory)
- Panic Safety (weight: 10, Scored)

## Gates
- Format Check (Required): `cargo fmt -- --check`
- Clippy Lints (Required): `cargo clippy -- -D warnings`
- Unit Tests (Required): `cargo test`
- Documentation Tests (Scored, skippable): `cargo test --doc`
<!-- lexicon:end:lexicon-context -->
