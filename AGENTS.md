# AGENTS.md

Instructions for coding agents (Claude Code, Cursor, Codex, or others) working in this repository.

## What this is

PacMagic is a Rust CLI + library that wraps native package managers (apt, pacman, paru, brew, winget) behind a single uniform interface (`pacmagic install neovim`, `pacmagic search python`, etc.), with a trait-based adapter architecture for adding more.

## Commands

Preferred (uses the `mise`-pinned Rust toolchain from `mise.toml`):

```bash
mise install          # install pinned Rust toolchain
mise run dev -- <args>  # e.g. mise run dev -- install --dry-run neovim
mise run test          # cargo test --all-targets --all-features --locked
mise run lint          # cargo clippy --all-targets --all-features --locked -- -D warnings
mise run fmt-check     # cargo fmt --all -- --check
mise run format        # cargo fmt --all (writes files)
mise run doc-check     # cargo doc --no-deps --all-features --locked (RUSTDOCFLAGS=-D warnings)
mise run all           # bundled: fmt-check, lint, test, deny, doc-check, machete
```

`mise run all` mirrors CI and needs `cargo-deny` and `cargo-machete` on `PATH`. Without mise, use plain Cargo (see `CONTRIBUTING.md` for the equivalent `cargo` invocations); MSRV is **1.82** (`rust-version` in `Cargo.toml`), toolchain pin for local dev is `1.94.0` (`mise.toml`).

Run a single test:
```bash
cargo test --locked test_name              # by name substring, across the whole crate
cargo test --locked --lib adapters::apt    # scope to a module path
```

Every adapter/module keeps its unit tests in an inline `#[cfg(test)] mod tests` block at the bottom of the same file — there is no separate `tests/` directory.

## Architecture

### Data flow

`main.rs` (clap CLI) parses args → resolves command/alias via `Config` → builds `OperationOptions` → calls `PacMagic::execute(operation, packages, options)` in `lib.rs`, which:
1. Validates package names / search query (`core::validation`) to block shell-injection characters — this runs before any adapter sees the input.
2. Picks a `PackageManager` via `AdapterRegistry` (explicit `-p/--package-manager` flag/config, else highest-`priority()` available adapter).
3. Confirms the adapter `supports_operation()`.
4. For `Install`, first runs `handle_install_pre_update` (update/upgrade-before-install policy — see `ARCHITECTURE.md`), then dispatches to the adapter method matching the `Operation` variant.

### Core abstractions (`src/core/`)

- `traits.rs` — the `PackageManager` trait every adapter implements (`install`, `remove`, `update`, `upgrade`, `upgrade_all`, `search`, `list_installed`, `info`, `clean`, `files`, `check`, `autoremove`, `owns`, plus `name()`/`command_names()`/`is_available()`/`priority()`/`supports_operation()`). `PackageManagerExt` adds optional repo operations (`add_repository`/`remove_repository`/`list_repositories`) with a blanket default that returns `OperationNotSupported` — adapters only override what they need.
- `operation.rs` — the `Operation` enum (one variant per CLI action), its default aliases (e.g. `install`/`i`/`add`), and `OperationOptions` (builder-style: `no_confirm`, `use_sudo`, `verbose`, `dry_run`, `show_commands`, `install_behavior`, `extra_flags`, `force`). Also `InstallBehavior` (what to do if a package is already installed: `Upgrade`/`Reinstall`/`Skip`/`Abort`/`Default`) and `InstallPreUpdateBehavior` (whether to update/upgrade before an install: `UpdateAndUpgrade`/`UpdateOnly`/`UpdateAndAsk`/`None`/`Default`, where `Default` means "update+upgrade only on rolling releases").
- `executor.rs` — `CommandExecutor` wraps `std::process::Command` execution, handling `dry_run` (prints instead of running), `show_commands` (echoes the command with 🔍/⚡ prefixes), and `verbose` logging. Adapters build a `std::process::Command` and always run it through a `CommandExecutor`, never `Command::output()`/`status()` directly.
- `validation.rs` — whitelist-based validators for package names and search queries; this is the injection-prevention boundary, since adapters shell out to real package managers.
- `error.rs` — `PacMagicError` (thiserror-based); has `is_recoverable()` and `help_text()` for user-facing hints, and `is_command_failure()` used by `main.rs`/`lib.rs` to suppress redundant error printing when the underlying command already wrote to stderr/stdout.

### Adapters (`src/adapters/`)

One file per package manager (`apt.rs`, `pacman.rs`, `paru.rs`, `brew.rs`, `winget.rs`), each a unit struct implementing `PackageManager`. `AdapterRegistry::new()` in `mod.rs` hardcodes the built-in adapter list; `auto_detect()` filters to `is_available()` adapters and picks the highest `priority()` (e.g. paru > pacman on Arch so AUR-capable tooling wins by default; apt priority depends on detecting `/etc/debian_version`). See `CONTRIBUTING.md` for the concrete steps and a code skeleton for adding a new adapter.

### Config (`src/config/mod.rs`)

TOML config loaded from `$XDG_CONFIG_HOME` (or `~/.config` on Linux, `~/Library/Application Support` on macOS) `/pacmagic/config.toml`, falling back to `Config::default()` if absent. Holds `general` settings, `aliases` (command name → `Operation`, values can be a single string or an array for multi-command sequences like `sync = ["update", "upgrade-all"]`), and per-package-manager overrides. `config.example.toml` documents the full schema. Default aliases are always merged in on load so user config only needs to override/add entries.

### Platform detection (`src/platform/mod.rs`)

`PlatformDetector` detects OS and (on Linux) distro from `/etc/*-release` files, maps distro → recommended package manager, and exposes `is_rolling_release()` (Arch/Gentoo/Tumbleweed/Debian-unstable/Fedora-Rawhide), which drives the `InstallPreUpdateBehavior::Default` policy in `lib.rs`.

### CLI-only code (`src/main.rs`, `src/completion.rs`, `src/command_names.rs`)

`main.rs` handles alias resolution (including multi-command aliases run sequentially via `run_multi_command`), a default-command-when-no-args flow, and a fallback re-parse path (`extract_unknown_args`) that lets unrecognized flags pass through to the underlying package manager via `--pm-args`/trailing `--`. `completion.rs` implements dynamic bash/fish/zsh completions (detects `COMP_LINE`/`_FISH_COMPLETE` env vars directly in `main()`, before clap even parses args) including live package-name completion with a 500ms timeout. Shell-completion generation is one of the few places doing non-trivial string/output parsing outside the adapters.

## Documentation map

- `README.md` — pitch, install, usage
- `CONTRIBUTING.md` — canonical dev setup, testing, PR process, commit conventions, release process, and the walkthrough for adding a new package manager
- `ARCHITECTURE.md` — how the pieces fit together and why, kept in sync with the actual trait/error definitions; its "Current Characteristics" section is the honest as-built baseline for the planned design/performance refactor
- `ROADMAP.md` — near-term plans; flags the design/performance refactor as the next priority
- `CHANGELOG.md` — generated by release-please; don't hand-edit it

## Notes

- Security-sensitive boundary: `core::validation` is what stands between user input and shelling out to a real package manager. Any new code path that passes user-supplied strings to adapters must go through it (see `PacMagic::execute` in `lib.rs`).
- CI (`.github/workflows/ci.yml`) runs fmt, clippy (`-D warnings`), tests on Linux/macOS/Windows (stable + beta), `cargo-deny`, doc build (`-D warnings`), MSRV check, and `cargo-machete`, gated behind a final `ci-pass` job — match this locally with `mise run all` before pushing.
- Rust edition is 2021 for the crate (`Cargo.toml`) but `.rustfmt.toml` is pinned to `edition = "2024"` formatting rules — this is intentional, not a mismatch to fix.
- **Commit messages matter beyond style**: releases are fully automated by [release-please](https://github.com/googleapis/release-please) (`.github/workflows/release-please.yml`, config in `release-please-config.json`/`.release-please-manifest.json`), driven by Conventional Commit types on `main` (`feat`/`fix`/breaking-change footers bump the version and populate `CHANGELOG.md`; see `CONTRIBUTING.md`). Get the commit type right.
- release-please's PRs and tags are authored with the default `GITHUB_TOKEN`, so per GitHub's recursive-workflow-trigger prevention, `ci.yml` does **not** run on release-please's own release PRs, and pushing its tag doesn't independently trigger a workflow — the binary-build/publish jobs in `release-please.yml` are chained via `needs:`/job outputs in the *same* workflow run instead. This is intentional, not a gap to fix by adding a PAT.
