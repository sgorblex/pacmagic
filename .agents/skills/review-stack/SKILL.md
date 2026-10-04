---
name: review-stack
description: Use before submitting a branch — or a whole Graphite stack — as PRs in this repository, or when asked to review the current branch's or stack's own changes for quality, architecture, naming, bugs, convention adherence, and scope, then apply the safe fixes. Covers per-branch diff scoping across a stack, parallel per-branch review, this repo's review dimensions and doc-update obligations, and the bottom-up mise verification sweep.
---

# Review and fix a branch or stack (pre-PR pass)

Self-review of **the branches' own changes**, run once implementation feels done and before any PR exists. Works on one branch or an N-branch **Graphite** stack. Report first, then fix what is safe to fix.

## 1. Pick the branches

```bash
gt log short -sr    # this stack, trunk-first (-s = ancestors+descendants of HEAD, -r = reversed)
```

A harness-generated **`worktree-…`** branch is **untracked**, and every `gt` operation on it errors, `-s` included — move the work onto a real branch first (`git switch -c <type>/<short_name>`, then `gt track --parent main`; see [`start-work`](../start-work/SKILL.md)), or fall back to `gt log short -r` and pick the stack by hand. Ordered trunk → tip:

```bash
gt log short -sr | sed -E 's/\([^)]*\)//g; s/[^A-Za-z0-9._\/-]+/ /g' | awk 'NF{print $NF}'
```

Default target: every branch above `main` in that list. If only part of it is this session's work, say which branches you are reviewing and confirm with the user before sweeping the rest. A `(name)` annotation means that branch is checked out in **another worktree**: you can diff and read it, but you cannot check it out or amend it.

## 2. Scope each branch

"The diff" is against the branch's **parent**, not `main`. In a trunk-first list the parent is the previous entry (`main` for the first). No checkout needed — diff the refs:

```bash
git merge-base --is-ancestor "$PARENT" "$BRANCH"   # must pass; if not, wrong parent or unrestacked
git log --oneline "$PARENT..$BRANCH"               # this branch's own commits
git diff --stat "$PARENT...$BRANCH"                # blast radius
git diff "$PARENT...$BRANCH"                       # the review surface
git status --short                                 # uncommitted work (checked-out branch only)
```

Three-dot, so an unrestacked parent does not drag its own changes in — and if the ancestor check fails you have either paired the wrong parent (a fork in the flattened `-r` list) or the stack needs `gt restack` — resolve that before reviewing, or every finding from the diff is suspect. The working tree sits at **one** branch: read context for any other with **`git show <branch>:<path>`**, never from disk. Then read whatever the diff raises questions about — callers, an existing helper that makes the new code redundant, the other adapters that should have changed alongside this one.

## 3. Review dimensions

Per branch:

- **Bugs and functionality** — does it do what the branch claims, including edge cases, error paths, and exit codes of the underlying package manager.
- **Architecture** — right layer: CLI parsing and flag plumbing in `main.rs`, cross-adapter policy in `PacMagic::execute` (`lib.rs`), native command mapping in the adapter. PacMagic abstracts **commands, not output**: an adapter forwards the package manager's own output rather than reformatting it, unless PacMagic itself consumes the result (e.g. `list_installed` for completion and `--install-behavior` checks).
- **Security boundary** — any new path from user input to an adapter goes through `core::validation`; commands run through `CommandExecutor`, never `Command::output()`/`status()` directly.
- **Adapter consistency** — every built-in adapter (`apt`, `pacman`, `paru`, `brew`, `winget`) was considered, `supports_operation()` matches what is implemented, and a trait change updated the test mock in `src/adapters/mod.rs`.
- **Reuse and convention** — an existing helper, executor method, or adapter pattern already covers this.
- **Naming** — clarity over abbreviation; purely functional methods named as **nouns, not verbs**.
- **Readability** and **internal consistency**.
- **Restraint** — comments that restate the code, and documentation added beyond what the change requires.
- **Size** — a function or file that should be split or extracted further.
- **Tests** — inline `#[cfg(test)] mod tests` in the same file; new behaviour that is testable without a real package manager is tested.
- **Doc obligations** — walk the *After you change the repository* table in [`AGENTS.md`](../../../AGENTS.md), row by row, against this branch's diff. It is the list; do not work from memory of it.
- **Public API** — a changed public signature (`PackageManager` trait, `OperationOptions`, re-exports in `lib.rs`) breaks library users: decide deliberately whether the squash title needs a `!`.
- **Branch ships as one unit** — every commit here squash-merges into one commit on `main`. If an earlier commit in this **unmerged** branch added something a later commit makes obsolete, fix it **at the source** with `gt modify`, never with a follow-up revert commit.
- **Branch name and commits** — `<type>/<short_name>`, snake_case, no scope segment; Conventional Commits throughout; the type matches what the branch actually squashes to, since **release-please** reads it to bump the version and write `CHANGELOG.md`.
- **mise discipline** — new instructions, scripts, and docs route through **`mise run`** tasks rather than restating raw `cargo` invocations.

Across the stack (only visible from the whole sweep — do this pass yourself, after the per-branch ones land):

- **Granularity** — judge the stack that actually resulted, not the one that was planned: branches get added along the way, and that is normal. The test is whether it reads as a **sequence of changes each worth reviewing on its own** — every branch squashing to one coherent commit that reverts on its own, nothing riding along that belongs in its own PR, and no branch so slight that reviewing it separately tells you less than reading it inside its neighbour. Where it does not, **`gt fold`** (into the parent, restacking descendants; **`--close`** if it has an open PR) or **`gt split --by-commit`** / **`--by-file <pathspec>`**. Expect to fold more often than you split, but that is a prior, not a quota.
- **Churn** — a later branch rewriting, renaming, or deleting what an earlier one added. Fix it at the source in the earlier branch when both are unmerged and unsubmitted; **flag instead of amending** when the earlier branch already has an open PR.
- **Duplication** — the same helper, type, or test fixture invented independently in two branches (two adapters growing the same flag-mapping helper is the usual case).
- **Doc attribution** — the README line, `ARCHITECTURE.md` update, or config-template entry lands in the branch that introduces the behaviour, not at the tip.
- **Scaffolding** — a shim or temporary flag added low in the stack and never removed by the branch that made it redundant.
- Anything else worth flagging.

## 4. Parallelise the reads, serialise the writes

Per-branch review is independent — for a stack of three or more, dispatch **one read-only subagent per branch** in a single message: give it the branch and parent refs, the per-branch dimension list above, and the rule that it may only run `git diff` / `git log` / `git show` and read files, never check out, edit, or run `gt`. Collect their findings, then do the cross-branch pass yourself. One or two branches: just read them inline.

Everything that writes — fixes, `gt modify`, verification — stays **serial**, because there is one working tree.

## 5. Report, then triage

Report **before** touching anything: findings grouped by branch, then the cross-branch section, each as `file:line` plus one line of why, ordered by severity. Nothing worth raising is a result too — say so explicitly rather than going silent.

Then, per finding:

- **Easy, clearly an improvement, and in scope for the branch that owns it** → apply it.
- **Restructuring the stack** (a fold or a split) → **propose it, never apply it silently.** It rewrites branches and invalidates any open PR, so it needs the same explicit go-ahead as a design decision.
- **A real improvement but needs an architectural or design decision** → stop and ask.
- **Out of scope for the stack's topic** → do not propose it at all.

## 6. Apply the fixes

Bottom-up, one branch at a time:

```bash
gt checkout <branch>   # skip: branches another worktree holds
# apply that branch's fixes
git status --short     # nothing unrelated about to be staged
gt modify --all        # amends the branch commit; auto-restacks descendants
```

Amending is right rather than adding a commit — the branch squash-merges as one unit. If the tree held unrelated pre-existing changes, `git add` only the reviewed paths and run `gt modify` without `--all`. Finish with `gt log short -sr` to confirm nothing is left needing a restack.

Never `git stash` here: the stash stack is shared with every other worktree on this machine. Set work aside with a WIP commit instead.

## 7. Verify the sweep

Bottom-up, on each branch, run what its paths touch:

| Changed | Run |
| --- | --- |
| anything in `src/` | `mise run fmt-check`, `mise run lint`, `mise run test`, `mise run doc-check` |
| `Cargo.toml` / `Cargo.lock` | additionally `mise run deny`, `mise run machete`, `mise run msrv` |
| an adapter's command building | a dry-run smoke test against the real package manager where it is installed: `mise run dev -- -n -p <pm> <command> …` prints the exact command |

`mise run check-all` covers the lot and mirrors CI; prefer it at the tip of the stack. Traps:

- **Judge by exit status, not by grepping the output.** `mise.toml` sets `CARGO_TERM_COLOR=always`, so a `grep '^error'` over task output matches nothing and a failed build looks clean.
- **Formatting failures stop a `&&` chain** before clippy or tests run — check every step actually ran.
- CI also runs tests on **macOS and Windows**; `#[cfg(target_os)]` code and path handling you cannot run locally deserve a closer read instead.

Report a branch as verified only once its checks have **actually run**, and say plainly which ones did not.

## Never

**No `git push`, `gt submit`, `gt push`, `gh pr create`, merge, or PR comment** — this skill stops one step short of the PR. Only the user's explicit request in that same turn changes that; see [`AGENTS.md`](../../../AGENTS.md) — *Git and Graphite*.
