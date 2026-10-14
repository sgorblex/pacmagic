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
mise run lint-targets  # the same clippy for the Windows and macOS targets (cfg-gated code)
mise run fmt-check     # cargo fmt --all -- --check
mise run format        # cargo fmt --all (writes files)
mise run fix           # clippy --fix, then cargo fmt (writes files)
mise run doc-check     # cargo doc --no-deps --all-features --locked (RUSTDOCFLAGS=-D warnings)
mise run msrv          # cargo check on the MSRV toolchain (installed via rustup)
mise run check-all     # bundled: fmt-check, lint, lint-targets, test, deny, doc-check, msrv, machete (alias: all)
```

`mise run check-all` mirrors the Woodpecker gates and needs `cargo-deny` and `cargo-machete` on `PATH`. Without mise, use plain Cargo (see `CONTRIBUTING.md` for the equivalent `cargo` invocations). The MSRV is `rust-version` in `Cargo.toml` (bumped by hand); the local toolchain pin is `rust` in `mise.toml` (bumped by Renovate). Don't restate either number elsewhere.

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
3. Confirms the adapter `supports_operation()`, and `supports_source()` for the requested `--official`/`--user` restriction.
4. For `Install`, first runs `handle_install_pre_update` (update/upgrade-before-install policy — see `ARCHITECTURE.md`), then dispatches to the adapter method matching the `Operation` variant.

### Core abstractions (`src/core/`)

- `traits.rs` — the `PackageManager` trait every adapter implements (`install`, `remove`, `update`, `upgrade`, `upgrade_all`, `search`, `list_installed`, `info`, `clean`, `files`, `check`, `autoremove`, `owns`, plus `name()`/`command_names()`/`is_available()`/`priority()`/`supports_operation()`/`supports_source()`). `PackageManagerExt` adds optional repo operations (`add_repository`/`remove_repository`/`list_repositories`) with a blanket default that returns `OperationNotSupported` — adapters only override what they need.
- `operation.rs` — the `Operation` enum (one variant per CLI action), its default aliases (e.g. `install`/`i`/`add`), and `OperationOptions` (builder-style: `no_confirm`, `use_sudo`, `verbose`, `dry_run`, `show_commands`, `install_behavior`, `extra_flags`, `force`, `source`). Also `PackageSource` (`Any`/`Official`/`User`, from `--official`/`--user`; only honored for operations where `is_source_filterable()`), `InstallBehavior` (what to do if a package is already installed: `Upgrade`/`Reinstall`/`Skip`/`Abort`/`Default`) and `InstallPreUpdateBehavior` (whether to update/upgrade before an install: `UpdateAndUpgrade`/`UpdateOnly`/`UpdateAndAsk`/`None`/`Default`, where `Default` means "update+upgrade only on rolling releases").
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

## Git and Graphite

- Branches are managed with **Graphite** (`gt`): `gt create` / `gt modify` / `gt restack` rather than raw `git commit --amend` or rebasing by hand. Check the current stack (`gt log short`) before creating branches, so new work stacks on the right parent and nobody else's branches get orphaned or restacked.
- Branch names are **`<type>/<short_name>`**: one slash, lowercase `snake_case`, two to four words, `<type>` being the Conventional Commit type the branch squashes to, no scope segment (`feat/package_source_flags`). Pass the name to `gt create` explicitly — without one it slugifies the commit message.
- **A branch ships as one unit**: PRs are squash-merged, so each branch becomes one commit on `main`. Within an unmerged branch, fix an earlier commit at the source (`gt modify`) rather than adding a commit that reverts it. Put the **why** in the commit body — a later review reads the branch, not the conversation that produced it.
- **Never run remote-mutating commands — `git push`, `gt submit`, `gt push`, `gh pr create`, merging, or commenting on a PR — unless the user explicitly asks in the current request.** Local commits, stack edits, and read-only inspection need no asking.
- **Worktrees**: `.claude/settings.json` runs background agents in their own worktree under `.claude/worktrees/` (gitignored). From the shared main checkout, ask before creating one for feature-sized work. A harness-generated `worktree-…` branch is untracked scratch that `gt` refuses to operate on: open the real branch with `git switch -c <type>/<short_name>` + `gt track --parent <parent>`, then delete the scratch branch with `git branch -d`; a `SessionEnd` hook sweeps any merged ones left behind. Hand a worktree back in place, committed and clean — do not `git worktree remove` it. Never use bare `git stash`: the stash stack is shared across worktrees.

The [`start-work`](.agents/skills/start-work/SKILL.md) and [`review-stack`](.agents/skills/review-stack/SKILL.md) skills walk through opening a branch and the pre-PR review of a branch or stack; [`woodpecker`](.agents/skills/woodpecker/SKILL.md) covers editing a `.woodpecker/` pipeline.

## AI-agent skills

Skills live under **`.agents/skills/<name>/SKILL.md`** so any agent can use them; `.claude/skills/<name>` is a **relative symlink** back to that directory, never a second copy — edit the `.agents/skills` original.

## After you change the repository

Update documentation in the **same change** when your edits touch the areas below. Write as little as the rule needs: no comments restating the code, and no documentation beyond what the change requires.

| You change… | Update… |
| --- | --- |
| User-visible behaviour: commands, flags, what a command prints | `README.md` (Usage) |
| The `PackageManager` trait, `OperationOptions`, errors, or the data flow | `ARCHITECTURE.md`; this file's **Architecture** section if its summary no longer holds |
| A new package manager | `README.md` supported list; `ARCHITECTURE.md` adapter tree; `CONTRIBUTING.md` (Adding a Package Manager) if the steps changed |
| A config option | `config.example.toml` and the `init-config` template in `Config::template` (`src/config/mod.rs`) |
| Dev commands, `mise.toml` tasks, the toolchain | **Commands** above; `CONTRIBUTING.md` |
| CI, release automation, Renovate | **Notes** below; `CONTRIBUTING.md` (Continuous Integration, Release Process); the `woodpecker` skill for pipeline conventions |
| Plans or priorities | `ROADMAP.md` |
| A convention agreed with the user in conversation | This file, `CONTRIBUTING.md`, or a skill — in the same change, not left in chat history |
| Versions or `CHANGELOG.md` | **Nothing by hand** — release-please owns them |

## Notes

- PacMagic abstracts package-manager **commands, not their output**: an adapter forwards the native command's output as-is (run via `execute_interactive`) instead of parsing and reformatting it, unless PacMagic itself consumes the result (e.g. `list_installed` for shell completion and `--install-behavior` checks).
- Security-sensitive boundary: `core::validation` is what stands between user input and shelling out to a real package manager. Any new code path that passes user-supplied strings to adapters must go through it (see `PacMagic::execute` in `lib.rs`).
- CI: **Woodpecker** (`.woodpecker/*.yaml`) runs every PR gate, mirrored locally by `mise run check-all` — run it before pushing. **GitHub Actions** keeps only what needs its runners or API: native macOS/Windows tests after merge (`cross-platform.yml`), releases, and PR assignment. Label a PR `ci:pause` to withhold its Woodpecker runs. Details: `CONTRIBUTING.md` — Continuous Integration.
- Dependencies are updated by [Renovate](https://docs.renovatebot.com/) (`renovate.json`): routine bumps wait for approval on the Dependency Dashboard issue, then automerge (minor/patch cargo only); security fixes skip approval and use `fix(deps):` so release-please cuts a release for them. `.github/workflows/assignee.yml` assigns Renovate's PRs to the repo owner.
- Rust edition is 2021 for the crate (`Cargo.toml`) but `.rustfmt.toml` is pinned to `edition = "2024"` formatting rules — this is intentional, not a mismatch to fix.
- **Commit messages matter beyond style**: releases are fully automated by [release-please](https://github.com/googleapis/release-please) (`.github/workflows/release-please.yml`, config in `release-please-config.json`/`.release-please-manifest.json`), driven by Conventional Commit types on `main` (`feat`/`fix`/breaking-change footers bump the version and populate `CHANGELOG.md`; see `CONTRIBUTING.md`). Get the commit type right.
- release-please's PRs and tags are authored with the default `GITHUB_TOKEN`, so per GitHub's recursive-workflow-trigger prevention no GitHub Actions workflow runs on its release PRs, and pushing its tag doesn't independently trigger a workflow — the binary-build/publish jobs in `release-please.yml` are chained via `needs:`/job outputs in the *same* workflow run instead. This is intentional, not a gap to fix by adding a PAT. Woodpecker is not subject to that rule, so the PR gates do run on release PRs; don't exclude `release-please--*` branches from them.
