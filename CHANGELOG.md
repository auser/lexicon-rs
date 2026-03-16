# Changelog

## [0.1.4] - 2026-03-16

### Miscellaneous

- Remove stale [workspace] section from Cargo.toml ([bb267f6](https://github.com/auser/lexicon-rs/commit/bb267f6d6c3de5c43ca48cac3bfbe14c9f7ccfc4))

### Refactoring

- Consolidate 15-crate workspace into single lexicon-rs crate ([b609d8c](https://github.com/auser/lexicon-rs/commit/b609d8c24efa5faf467ada0a66ff1e794a4882ae))
## [0.1.3] - 2026-03-16

### Bug Fixes

- Add versions to workspace deps and vendor OpenSSL for cross-compilation ([bcf2915](https://github.com/auser/lexicon-rs/commit/bcf29158568e93284e66203e12865bea927df8d2))
- Prevent release-auto from creating duplicate changelog commits ([236c6fd](https://github.com/auser/lexicon-rs/commit/236c6fd11bfe8d485499295bb8afba1101427cc1))
- Switch to rustls-tls, skip published crates, add fail-fast false ([50b32bb](https://github.com/auser/lexicon-rs/commit/50b32bb570003ca1ddbc914de5f6e348cbd8c0bc))
- Increase publish sleep to 65s and add 429 retry logic ([f5f08fb](https://github.com/auser/lexicon-rs/commit/f5f08fb323d8c001191b673bc8c09196ad85cda0))
- Add missing description to lexicon-coverage ([5c4c096](https://github.com/auser/lexicon-rs/commit/5c4c09677ce4f84a27e4ac0294f9a9847667ab7f))
- Parse retry-after time from 429 response to avoid repeated rate limiting ([66ed584](https://github.com/auser/lexicon-rs/commit/66ed584dc4a5009cd8ae85d9b8f977043abcbf89))

### Miscellaneous

- Update changelog for v0.1.2 ([c147445](https://github.com/auser/lexicon-rs/commit/c147445082e95cc3b9b4682987250816c4410e9f))
- Update changelog for v0.1.3 ([dbbb068](https://github.com/auser/lexicon-rs/commit/dbbb0684a3768c31e7162b7b9e6c36eb188a444f))
- Opt into Node.js 24 for all GitHub Actions workflows ([817304a](https://github.com/auser/lexicon-rs/commit/817304a3da41347d8a12c8cbbe724e93fab064dc))
## [0.1.2] - 2026-03-16

### Bug Fixes

- Correct GitHub link, release permissions, and publish error handling ([d63a89c](https://github.com/auser/lexicon-rs/commit/d63a89c96fe28b5d5cdedf2a1c3b133aa72b0109))

### Miscellaneous

- Update changelog for v0.1.2 ([205f607](https://github.com/auser/lexicon-rs/commit/205f60730960bd494b25304b01472a072ec8b929))
## [0.1.1] - 2026-03-16

### Miscellaneous

- Update changelog for v0.1.1 ([148dd21](https://github.com/auser/lexicon-rs/commit/148dd21df1a530ef76281b1503ae5d494dad3060))
- Update changelog for v0.1.1 ([38761d9](https://github.com/auser/lexicon-rs/commit/38761d9c5bb59256b73c2fcdcbed1459ae2b59d7))
