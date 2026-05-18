# Decisions (ADRs)

## SQLite Encryption Decision

- **Source**: `D:/Personal/Desktop/2/ArcaneCodex/docs/planning/sqlcipher-decision.md`
- **Status**: Proposed (locked: false)
- **Title**: SQLite encryption scheme decision document
- **Decision Statement**: Adopt Plan B (application-layer encryption of sensitive fields) as the current encryption scheme, with Plan A (SQLCipher full-database encryption) as a roadmap target for v1.1/v1.2.
- **Scope**: database encryption, SQLite, cryptography, privacy
- **Key Points**:
  - Current database is plaintext SQLite via rusqlite 0.31 bundled + r2d2 connection pool
  - API Key already has application-layer AES-256-GCM encryption (v2 with random nonce)
  - EXIF GPS coordinates are the primary privacy risk
  - Plan B: selectively encrypt GPS coordinates and file paths using existing crypto.rs infrastructure
  - Plan A (SQLCipher): planned for v1.1 or v1.2 via `bundled-sqlcipher-vendored-openssl` feature flag
  - Plan C (plaintext + OS-level): only recommended with BitLocker/FileVault
  - Migration strategy: phased approach across 3 stages
  - Query preservation requirement: encryption must not break search capabilities
- **Open Decision Points** (require user confirmation):
  1. Whether to encrypt only GPS coordinates vs. entire EXIF JSON
  2. Key management strategy: machine fingerprint vs. master password vs. OS credential manager
  3. CI build time acceptance for SQLCipher
  4. Compliance urgency
