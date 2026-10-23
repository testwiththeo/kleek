# AGENTS.md — Development Guidelines for AI Coding Agents

This repository (`kleek`) is engineered for autonomous and parallel multi-agent development. Every agent working in this repository MUST strictly abide by the rules and invariants defined below.

---

## 1. Directory & Crate Boundary Invariants

To prevent race conditions, merge conflicts, and state contamination across parallel agent sessions:

* **Strict Crate Isolation:**
  * When assigned a ticket for `crates/kleek-proto/`, touch ONLY files in `crates/kleek-proto/`.
  * When assigned a ticket for `crates/kleek-driver/`, touch ONLY files in `crates/kleek-driver/`.
  * When assigned a ticket for `crates/kleek-daemon/`, touch ONLY files in `crates/kleek-daemon/`.
  * When assigned a ticket for `crates/kleek-cli/`, touch ONLY files in `crates/kleek-cli/`.
* **Zero Cross-Crate Dependency Violations:**
  * `kleek-proto` MUST NEVER import I/O crates (`hidapi`, `tokio`, `udev`, `evdev`). It is a pure, zero-I/O codec crate.
  * `kleek-cli` MUST NEVER communicate with hardware directly via `hidapi` or `/dev/hidraw*`. It MUST only communicate through the D-Bus interface (`zbus`) exposed by `kleek-daemon`.
* **Root Changes:**
  * Do NOT modify the root `Cargo.toml` without explicit ticket instruction.

---

## 2. Code Quality & Rust Standards

* **Idiomatic Rust 2024:**
  * Prefer strong types, enums, and newtype wrappers over raw primitives.
  * Avoid `unwrap()` or `expect()` in production code paths. Use pattern matching or `?`.
* **Error Handling Conventions:**
  * **Library crates (`kleek-proto`, `kleek-driver`):** Use `thiserror` to define structured, strongly typed error enums.
  * **Binary / Application crates (`kleek-daemon`, `kleek-cli`):** Use `anyhow` for top-level error context and propagation.
* **Code Hygiene (Anti-Slop):**
  * Do NOT add decorative or section divider comments (e.g. `// =============================`).
  * Do NOT add repetitive explanatory comments that merely restate what the code does. Code must be self-documenting.
  * Do NOT add persona intros, preambles, or conversational filler in commit messages or code docs.

---

## 3. Mandatory Quality Gates

Before declaring any ticket complete, you MUST execute and pass all three verification commands:

```bash
# 1. Format check
cargo fmt --check

# 2. Strict linter (zero warnings allowed)
cargo clippy --workspace --all-targets -- -D warnings

# 3. Complete test suite
cargo test --workspace
```

If any check fails, resolve the failure within your designated crate boundary before finishing your turn.

---

## 4. Hardware & Safety Invariants

1. **Never Intercept Motion (`REL_X`/`REL_Y`):**
   * High-polling mice (1000Hz–8000Hz) will jitter and consume excessive CPU if pointer motion passes through userspace event loops. Let `libinput` handle pointer velocity and acceleration natively.
2. **Never Guess Hardware Byte Offsets:**
   * Do not fabricate HID report formats or checksum algorithms. If a protocol specification or packet capture is missing, reference `SPEC.md` or request verification against recorded PCAP vectors.
