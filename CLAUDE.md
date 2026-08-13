# Working in this repo (agent conventions)

Multiple Claude Code agents frequently work on the KISS suite **at the same time**,
and several of them are launched in the *same* working directory
(`C:\Projects\KISS`). That means they share one working tree and one `HEAD`. A
branch switch or an uncommitted edit in one session is therefore visible to — and
can clobber — every other session in that directory. This has caused real
mid-operation churn (a session's `HEAD` moving underneath another's commit).

To prevent that, follow the worktree convention below.

## The rule

**The shared checkout at `C:\Projects\KISS` is a stable anchor. Do not switch its
`HEAD`.** Treat it as a read / coordination surface: reading files, running
read-only tools, coordinating over the claude-peers network. Never run
`git checkout <branch>` / `git switch` / `git checkout -b` in the shared
directory — that changes the branch for every session sharing it.

**If your session will *write* — commit, edit tracked files, or create a branch —
do it in your own git worktree**, not in the shared directory:

```sh
# from C:\Projects\KISS, off the intended base (usually main)
git worktree add -b <your-branch> C:/Projects/kiss-<task> main
# ... edit / commit / push / open a PR inside C:/Projects/kiss-<task> ...
git worktree remove C:/Projects/kiss-<task>      # when done
```

A worktree is a separate working directory with its own `HEAD` and index, sharing
the same `.git` object store — cheap to create, fully isolated. Your commits,
branch switches, and uncommitted edits cannot disturb any other session.

**Read-only / advisory sessions** (answering questions, reviewing, coordinating)
may stay in the shared directory without a worktree. The convention is scoped to
sessions that mutate the repo, not to every session.

## The anchor is stale, by design — do not measure `main` in it

The shared checkout stays where it is. That is the point, and it is also a trap:
**every read from it answers with the state at its `HEAD`, confidently, with no
marker that the answer is old.** Nothing prints a warning. A lint says CLEAN, a
grep finds nothing, a coverage figure comes back — all correct about a tree
nobody is working on.

**To measure `main`, read `main`:** `git show origin/main:<path>`, or a detached
worktree at `origin/main` (`git worktree add --detach <tmp> origin/main`). If you
measure in the shared tree anyway, **state its `HEAD` alongside every number** —
a figure without the commit it was taken at is not a measurement of anything.

This is not hypothetical. Four instances in one day, across three people:

- A cross-project dtype divergence computed off the anchor — wrong in **both**
  directions (`c32`/`c64` vs `c64`/`c128`, `s4` vs `i4`, wrong surplus count).
- A feature/cfg audit reporting **zero** `cfg(windows)` gates and no `harness/`
  module — clean, plausible, and describing a tree twelve commits behind.
- A `cargo test -- --list` capture taken from a pre-merge worktree, producing
  **eleven false absences**, all from one PR.
- Coverage read in the anchor at 38 commits behind: `330/902 (36.6%)`,
  `RESULT: VIOLATIONS FOUND`, and a red test that had already been fixed.

**Three of the four produced a clean-looking answer**, which is the direction
that does not get questioned — a false positive gets investigated, a false
negative gets filed. The fourth was caught *because it contradicted a result
someone had already verified*, not because it looked wrong. **Contradiction with
an independent finding is the only signal that reliably fires on a stale read.**

## Discipline

- **Base off `main`** (or the correct integration branch) so branches start clean.
  Note local `main` may be ahead of `origin/main`; check before you push, and do
  not publish another session's unpushed commits without the maintainer's go.
- **Announce shared-ref moves.** Before any operation that moves a ref other
  sessions depend on (a push to `main`, a `git branch -f`), post it on the
  claude-peers network and get a tree-clear from co-located sessions first.
- **Clean up.** `git worktree remove` your worktree when the work lands; don't
  leave stale worktree directories accumulating.
- **Worktrees prevent live clobbering, not logical conflicts.** Two branches that
  edit the same lines still conflict at merge time — worktrees make the *process*
  safe, they don't make the *changes* independent.

## Coordinate

Sessions discover and message each other over the **claude-peers** network
(`list_peers`, `send_message`, `set_summary`). Set a summary describing what you're
doing so co-located sessions know before they touch the shared tree.

See `CONTRIBUTING.md` for the human-facing contribution and review process.
