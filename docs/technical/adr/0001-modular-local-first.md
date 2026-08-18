# ADR 0001: Modular local-first monolith

Status: accepted.

Use Tauri as a deployable monolith with strict internal module boundaries. This keeps installation and offline operation simple while allowing collectors, repositories, and future engines to evolve independently. Rust is authoritative for operating-system data and persistence; React is authoritative only for interaction and presentation.
