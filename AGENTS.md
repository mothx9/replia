# Repository instructions

These rules apply to the entire REPLAI repository. Follow the
[development method](docs/development.md) for the operational workflow and
[documentation map](docs/README.md) for information ownership.

- Before changing files or committing, inspect the current branch, HEAD,
  staged/unstaged diff and untracked files. Preserve concurrent work; do not
  reset, discard, stash or overwrite another contributor's changes to simplify
  a task. Do not create branches or worktrees contrary to the user's scope.
- Keep generic terminal mechanisms here and application semantics in hosts.
  No sibling application checkout may be required to build or test REPLAI.
  Do not modify consumer repositories without task authorization.
- Preserve `unsafe_code = "forbid"` in the implementation crate. FFI unsafety
  belongs only in the separate binding, with local safety arguments and no
  fabricated lifetimes or accesses to private implementation modules.
- Define behavior and failure atomicity before extending public API. Keep one
  editor and renderer behind Rust and C; do not create fallback implementations.
- Run checks relevant to the changed boundary, including negative evidence.
  Report source identity, commands, observed properties and unqualified limits.
  Distinguish local checks from published CI and consumer/runtime qualification.
- Update the owning document when a contract changes. ROADMAP alone owns
  current project status; completed work belongs in Git history. Do not add
  archive directories, shadow status documents or speculative release claims.
- Preserve published history and use ordinary pushes. Inspect the final diff
  and remote identity before claiming delivery. Do not publish a release or
  begin a consumer migration as an incidental consequence of another task.
