# Copilot Instructions

## Tech Stack

- **Language:** Rust (latest stable)
- **Dependencies:** Latest versions only; prefer mature, well-maintained crates
- **Dependency Policy:** Only add external crates when functionality cannot be quickly implemented in-repo

## Code Quality

- **Linting:** `clippy` with default rules — zero warnings allowed before commit
- **Formatting:** `rustfmt` with default rules — all code must be formatted before commit
- Run `cargo fmt && cargo clippy -- -D warnings` before every commit

### Build Requirements

**Before finishing any task, the project MUST:**
1. Build successfully with `cargo build --release`
2. Have **zero warnings** during compilation
3. Pass `cargo clippy -- -D warnings` with no errors

If there are warnings, fix them before considering the task complete.

### CI Verification for Larger Features

**Before finishing a bigger feature or significant change:**

1. **First, run local checks (fast):**
   ```bash
   cargo fmt --all
   cargo clippy -- -D warnings
   cargo build --release
   cargo test --all-features
   ```

2. **Then, run full CI verification (Docker-based, matches GitHub Actions):**
   ```bash
   make ci
   ```
   Or for quicker validation: `make ci-quick` (format + clippy only)

This ensures the CI pipeline will pass. The Docker-based checks use the same Rust version and environment as GitHub Actions, catching issues that local toolchains might miss.

**Available CI commands:**
- `make ci` — Full CI pipeline (format, clippy, build, test, doc)
- `make ci-quick` — Quick checks (format + clippy)
- `make ci-msrv` — Check minimum supported Rust version (1.88)

## Project Structure

Follow standard Rust CLI project layout:
```
src/
  main.rs          # Entry point, argument parsing
  lib.rs           # Core library exports
  cli/             # CLI-specific code (args, commands, TUI)
  core/            # Business logic
  utils/           # Shared utilities
doc/
  progress.md      # Changelog (update on EVERY change)
  features.md      # Feature list with timestamps
  documentation.md # Detailed CLI documentation
```

## Documentation Requirements

| File | Purpose | Update Frequency |
|------|---------|------------------|
| `README.md` | Brief intro, motivation, prerequisites, quickstart | On significant changes |
| `doc/progress.md` | Historical changelog | **Every change** |
| `doc/features.md` | High-level feature list with timestamps | When features are added |
| `doc/documentation.md` | Detailed CLI usage documentation | When features change |
| `doc/roadmap.md` | Implementation roadmap with action items | Check items when completed |

### Roadmap Tracking

When completing action items from `doc/roadmap.md`:
- Mark completed items with `[x]` instead of `[ ]`
- Keep the roadmap up-to-date as features are implemented

## CLI Design

### Dual-Mode Architecture

1. **App Mode (TUI):** Full-screen interactive shell
   - Uses full terminal dimensions
   - All navigation, actions, and results within the TUI
   - Standard exit commands (q, Ctrl+C, Ctrl+D)
   - Graceful handling of terminal resize
   - Display message when terminal dimensions are too small

2. **CLI Mode:** Traditional command-line with arguments
   - Comprehensive `--help` at all levels
   - Follows POSIX conventions

### UX Standards

- Self-explanatory interface; add inline help where needed
- Consistent color scheme for status, errors, and highlights
- Responsive to terminal dimension changes
- Always handle edge cases (invalid input, small terminals, interrupts)

## Development Practices

- Research best practices before implementing new patterns
- Consider edge cases proactively for every feature
- Keep functions focused and testable
- Use `Result` and `Option` idiomatically for error handling
- Prefer compile-time guarantees over runtime checks
