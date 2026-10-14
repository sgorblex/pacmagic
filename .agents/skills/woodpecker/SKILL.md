---
name: woodpecker
description: Use when adding, editing, or debugging a `.woodpecker/*.yaml` CI pipeline in this repository, or when adding a new CI check. Covers the multi-file pipeline convention, the step-level path filter rule, the `ci:pause` label, image and tool pinning, and Woodpecker gotchas.
---

# Woodpecker CI pipelines

Woodpecker (`.woodpecker/*.yaml`) runs this repository's PR gates. GitHub Actions keeps only the jobs that need macOS/Windows runners or the GitHub API. Which pipelines exist, which checks are required, and how `ci:pause` behaves are covered in [`CONTRIBUTING.md`](../../../CONTRIBUTING.md), under *Continuous Integration*. This skill is about *how* to edit the Woodpecker side.

## The multi-file convention

Each file under `.woodpecker/` is its own pipeline and its own required GitHub check, `ci/woodpecker/pr/<file>`. Woodpecker discovers files on its own, so nothing needs registering. Every file has the same two-level `when`:

```yaml
when:                       # top level: event + ci:pause only, never a path
  - event: push
    branch: main
  - event: pull_request
    evaluate: 'not ("ci:pause" in split(CI_COMMIT_PULL_REQUEST_LABELS, ","))'

steps:
  - name: first
    when: &x-when           # step level: the path filter, anchored once
      - event: push
        branch: main
        path:
          include: &x-paths ['src/**', 'Cargo.toml', 'Cargo.lock', '.woodpecker/x.yaml']
      - event: pull_request
        path:
          include: *x-paths
  - name: second
    when: *x-when
```

Always include the pipeline's own file in its paths, so editing the pipeline re-runs it. To add a check, copy the closest file and change the steps and paths. A new file is a new required check, so it also needs adding to the ruleset (see *Renaming* in `CONTRIBUTING.md`).

Steps in one file run **sequentially** and share the workspace. Adding `depends_on` to any step switches the whole file to DAG mode, so don't add it to express order.

## Pinning

- **Images**: pin every step image to a digest (`tag@sha256:…`). Resolve it with `crane digest <image>` (or `docker manifest inspect`) and check it is multi-arch, since the agents are both amd64 and arm64. Renovate's `woodpecker` manager keeps digests current. The `rust` image tag follows `mise.toml`'s `rust` and is bumped in the same group.
- **Downloaded tools** (`scripts/ci-fetch-tools.sh`): versions live in step `environment:` vars, each under a `# renovate: datasource=… depName=…` comment, which a regex custom manager in `renovate.json` reads. A tool added without that comment goes untracked.
- **MSRV** is never pinned in CI. `msrv.yaml` reads `rust-version` from `Cargo.toml` at run time.

## Gotchas

- **Never put `path:` on the top-level `when` of a required pipeline.** A top-level miss means no run is ever created, so no status is posted. GitHub shows an absent required check as "Expected" forever, which blocks the merge. A step-level miss just skips the step, and the pipeline still reports success. The one deliberate exception is the top-level `ci:pause` `evaluate`, where withholding the status *is* the point.
- **`when.branch` matches a PR's *target* branch**, never its head. To filter by head branch, use `evaluate` over `CI_COMMIT_SOURCE_BRANCH`.
- **An `evaluate` runs on every event**, even ones its entry's `event:` doesn't match. PR-only variables (`CI_COMMIT_PULL_REQUEST_LABELS`, `CI_COMMIT_SOURCE_BRANCH`) then read as `""`. Write expressions so that the empty case gives the right answer for pushes too.
- **Woodpecker substitutes variables in the config before the shell sees it.** A `$NAME` in `commands:` that refers to a shell variable set inside the step may be rewritten first (`$$NAME` escapes it). Avoid the question: use `$(…)`, a temp file, or a `scripts/` file, which Woodpecker never rewrites (see `msrv.yaml` and `deps.yaml`).
- **`volumes:` need the repository marked *trusted*** on the Woodpecker server. That is an admin setting, and without it the cache mounts fail. Caches (`/var/lib/woodpecker-cache/…`) are **per agent**, so a cold cache on another agent is normal. Each pipeline gets its own `CARGO_TARGET_DIR` mount, so pipelines don't wait on each other's build lock.
- **The cargo cache is all of `CARGO_HOME`.** Mount the whole of it (`/cache/cargo-home`), not just `registry/`: cargo's package-cache lock lives at its root, and without it pipelines starting together unpack crates over each other. rustup then refuses to run (`rustup is not installed at '/cache/cargo-home'`), so prefix rustup commands with `CARGO_HOME=/usr/local/cargo`, and select a toolchain with `RUSTUP_TOOLCHAIN=…`, not `cargo +…`.
- **`rust:*-slim` is minimal.** It has no `rustfmt`, `clippy`, `git` or `curl`. Add components with `rustup component add`, and packages with `apt-get` and `DEBIAN_FRONTEND=noninteractive`.
- **Woodpecker runs only Linux containers.** Code behind `#[cfg(target_os = …)]` is linted by cross-target clippy in `lint.yaml`. Native macOS/Windows tests live in `.github/workflows/cross-platform.yml`.
- **Release-please PRs are not excluded.** GitHub Actions never runs on them (they come from `GITHUB_TOKEN`), but Woodpecker does, and that is the only CI the version bump in `Cargo.toml`/`Cargo.lock` gets.

Validate locally with `woodpecker-cli lint .woodpecker/`, for example `mise x 'github:woodpecker-ci/woodpecker[bin=woodpecker-cli]@3.19.0' -- woodpecker-cli lint .woodpecker/`.
