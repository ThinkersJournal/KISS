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
