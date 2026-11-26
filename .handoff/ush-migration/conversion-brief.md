# Convert a group of vize workflow scripts from shell (Pkl `.sh.pkl`) to ush

You are converting the shell programs of some vize CI workflows to ush
(https://github.com/ubugeeei-prod/ush), a typed language that compiles to POSIX sh.
The user's conditions apply to every line you write:

- Write real ush. Never embed shell with `shell "…"`, `$ …` inline shell, `extern`, or a
  `"""` block of shell text.
- The compiled `.sh` is committed under `compiled/` next to each program, so outside
  developers run plain `sh`.
- If ush lacks a feature, do NOT work around it. Stop that program and report the exact
  gap: what the shell did, and what ush construct is missing. The coordinator fixes ush
  upstream and releases it.
- Soundness of design comes first. Be meticulous about readability everywhere: names,
  comments and layout.

## Setup

- Repository: `/Users/nishimura/Source/github.com/ubugeeei-prod/vize`. Create your own
  worktree from `origin/ci/ush-foundation`:
  `git worktree add ../vize-ush-<group> -b ci/ush-<group> origin/ci/ush-foundation`.
  Work only there. Never use bare `git stash`.
- ush 0.19.0 is pinned in `flake.nix`. `nix develop --command …` provides `ush`, `node`
  and `pkl`, but the Nix store is often busy. A faster route that gives identical output
  is the release binary `ush-aarch64-apple-darwin.tar.gz` from
  `gh release download v0.19.0 -R ubugeeei-prod/ush`: verify it with `shasum -a 256 -c`,
  then put it first on PATH. Use the system `node` (v26) for the generator and tests.
- Read first:
  - `tools/README.md`, section "Writing scripts in ush": the conventions;
  - `tools/ci/workflows/11-docs-build/`: the pilot, with 4 programs, their `compiled/`
    builds and the `workflow.pkl` calls;
  - `tools/ci/lib/github.ush`: the shared module for GITHUB_OUTPUT, GITHUB_ENV,
    GITHUB_PATH and the job summary;
  - `.github/actions/setup-ush/scripts/install-ush.ush`: another example;
  - in the ush checkout `~/Source/github.com/ubugeeei-prod/ush-wt-else` (read only):
    `docs/shell-language.md`, `docs/native-scripting.md` and `docs/stdlib.md`.

## Per program

For each `tools/ci/workflows/<wf>/scripts/<name>.sh.pkl`:

1. Read the `.sh.pkl` and its generated `.sh`. Understand what it does and why, including
   strict mode, `interpreter = "bash"` and bash-isms.
2. Write `scripts/<name>.ush`:
   - open with one comment line saying what the program does and why;
   - name every external program in `use $bin::{…}`, never `$bin::*`; quote a name that is
     not an identifier and alias it: `"apt-get" as apt_get`;
   - use CLI-style words: a bare word is a string, `$name` reads a binding, `$env::NAME`
     reads a required environment variable (`std::env::get_or` for an optional one),
     `$"…"` interpolates;
   - break a long call with a trailing ` \`, one flag per line;
   - use pipelines `|` and `|&`, which have pipefail, the `std::io` stages (`to_file`,
     `append_to`, `from_file`, `discard_stderr`, `stderr_to`), and `cd dir { }` and
     `export N = v { }` blocks;
   - use `if <command>` as a condition, `status_of`, `allow_fail`, `spawn` and `defer`,
     `match` and `let x = match …`, `for x in [...]`, and the List methods;
   - reach files next to the program with `std::path::from_source "x.ts"`, never
     `$VIZE_CI_FILES`;
   - write step outputs, env, PATH and the summary through `mod "../../../lib/github.ush"
     as github`; it frames multi-line values safely;
   - keep behaviour identical: same commands, arguments, files, exit status and failure
     modes. ush already fails fast like `set -euo pipefail`.
3. Delete the `.sh.pkl` and its generated `.sh`.
4. In `workflow.pkl`, replace `ci.workflowScript("<name>.sh")` with
   `ci.workflowProgram("<name>")`. Use `ci.wslProgram` for WSL steps. A step that sourced
   a script for its side effects in the same shell (exported variables, `cd`) needs
   thought. Report it if it isn't obvious.
5. Run `node tools/commands/generate/repository-config.ts`. It compiles every `.ush` into
   `compiled/` and regenerates the workflow YAML. Then run `vp fmt` on each regenerated
   `.github/workflows/*.yml`: the generator re-renders a changed file in its own layout,
   but main keeps the formatter's layout. Then run
   `node tools/commands/generate/repository-config.ts --check`.
6. Run `dash -n` on every build (`nix shell nixpkgs#dash`). Run a program with stub
   commands on PATH where that is practical.

## Tests and custody

- Contract tests under `tests/tooling/` read step text through
  `tests/tooling/support/automation-source.ts`. A step that runs `sh …/compiled/x.sh`
  inlines the `.ush` source, and `automationShellPath(text)` returns the build for tests
  that execute it.
- Find the tests that mention your workflows (`git grep -l '<wf>' tests tools`) and run
  them with `node --test`. Update expectations that pinned the old shell text so they
  assert the equivalent ush text, keeping what each assertion protects. Never weaken an
  assertion.
- Some programs are pinned by content digest:
  - `tools/support/ci/workflow-program-history.json`;
  - `tools/config/byte-custody.tsv`;
  - the `producerHashes` in `tests/tooling/support/rust-cache-docs-wasm-consumers.mjs`;
  - historical step bodies in the generator (`historicalStepBodies`).
  Do NOT convert a program pinned there, or one that `restoreWorkflowNames`/frozen-record
  tests compare byte for byte. List it in your report instead; it needs a custody
  decision.
- Run `node tools/ci/commands/source-file-lengths.ts` and `vp fmt --check`.

## Commits and report

- One commit per workflow:
  `ci: write <wf>'s programs in ush`, with a body explaining any behaviour-relevant
  choice. End each message with
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Push the branch, but do NOT open a PR. The coordinator opens PRs after the foundation
  PR (#13729) merges.
- Report:
  - the workflows converted;
  - the programs skipped, with why: custody or an ush gap;
  - the ush gaps, each with a minimal example;
  - the tests you changed and why;
  - the verification you ran.
