# ADR 0003: SQLite and secret boundary

Status: accepted.

SQLite lives in the application-data directory and is reachable only through Rust repositories. Migrations are versioned and transactional. Future integration records may hold metadata and a `secret_ref`; credential material belongs in Windows Credential Manager.
