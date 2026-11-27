# sh → ush migration: handoff (2026-11-26, about 21:30 JST)

## Update 2026-11-27, about 14:00 JST (supersedes the matching items below)

- ush #198 and the v0.20.1 release (#199, tag `v0.20.1`, assets and `sha256sums.txt` published) are done.
- vize #13729 has two more commits:
  - `bc3bb8a417`: the generator refuses a build that calls the optional ush runtime (marker `__ush_std_host() {`), because such a build exits 127 without ush; tests are in `tests/tooling/ush-programs.test.ts`;
  - `5489c4fa43`: a `tools/README.md` note on the portable std.
- #13729's red shard (`vm-programs.test.ts` on `editors/vscode/test/workspace-trust.test.ts`) comes from main's #13745. The fix is #13773 (auto-merge). After it merges, merge main into `ci/ush-foundation` so CI runs on a fresh merge ref.
- `ci/ush-05-06` (`3df67a33f5`) now has:
  - the v0.20.1 pin commit `9423dff00f` (flake, lock and setup-ush sums);
  - 05-lsp-vs-volar `plan`/`measure` in ush. Under stub commands they match the old argv, output, summary and wrapper byte for byte.
  - Cherry-pick `9423dff00f` onto `ci/ush-08` and `ci/ush-07-11` when rebasing them.
- Pitfall: `std::int::sub` and other runtime-backed std functions compile fine but need ush when the build runs. Use `n + -1`.
- The `vize-ush-07-11` and `vize-ush-08` worktrees were removed (branches pushed); recreate them from the branches.

## The goal (the user's words, still in force)

> Retire the sh and ts scripts entirely and migrate to ubugeeei-prod/ush (phase 1: sh only). Write real ush, never `shell """`. Commit the compiled `.sh` under `./**/compiled/` so outside developers need no extra setup. If ush lacks a feature, commit to ush and take it through a release. Make setup one nix command. Soundness of design first, and readability everywhere.

On 11-26 the user added: carry every PR through to **merge**, not just open it.

Conventions (from `tools/README.md` § "Writing scripts in ush"):
- one purpose-comment line at the top of each program;
- an explicit `use $bin::{…}` list, never `$bin::*`;
- no `shell`/`$ …` escapes;
- `std::path::from_source` for sibling files, not `$VIZE_CI_FILES`;
- shared helpers live in `lib/` modules.

## ush (github.com/ubugeeei-prod/ush)

**Released (all tagged by me through `cut`, i.e. push the tag on the release merge commit):**
- v0.17.0: `} else {`, `$"…"`, `$env::`, the List API, glob.
- v0.18.0 (released by the owner): `let x = match …`.
- v0.19.0: #188, a trailing ` \` continues a command; #190, `from_source` returns a real path instead of leaking `__ush_pathref__:`.
- v0.20.0:
  - #191: a List argument spreads into argv;
  - #192: a parenthesis-unwrap miscompile of `(a) && (b)` fixed; `&&`/`||` in a value is now an error;
  - #193: `std::path::script`;
  - #194: the `detach cmd` statement;
  - #195: words like `300s` pass to programs as written;
  - #196: lists and groupings span lines.

**Open PR #198 `fix/argument-words`** (the next patch release, v0.20.1):
- the contracts pass now accepts a List argument; it failed in any file with a `fn` or `mod`;
- only canonical integers are `Int` words, so `-0`, `0755`, `007` and `+1` stay text.

Steps:
1. Merge #198 when green.
2. Release v0.20.1 the same way as #197: bump `Cargo.toml`, `Cargo.lock` (`cargo update -w`), `editors/vscode/package.json` and its lock (two spots), and date the CHANGELOG section; merge; tag the merge commit; push the tag.

**Gates before every push**, all of them:
- `cargo +stable fmt --all --check`;
- clippy with `-D warnings`;
- `cargo check -p ush_compiler --no-default-features` (no_std; I forgot this once);
- the workspace tests, debug and release;
- rustdoc;
- `sh scripts/check_rs_line_limit.sh` (250-line cap);
- `sh scripts/check_generated.sh`.

Run them as a real background task, not a plain `&`: backgrounded jobs ignore SIGINT and fail `ush_shell`'s signal test.

**CHANGELOG conflicts** between parallel PRs: `.handoff/ush-migration/merge_changelog.py CHANGELOG.md` merges the whole `## [Unreleased]` section of both sides and also normalizes duplicate headings. To land several PRs in one CI round, stack them, push them all, then squash-merge in order.

**Remaining ush gaps, lower priority:**
- **For the owner to decide:**
  - integer subtraction: `a - b` is ambiguous with the CLI word `-` (`cat -`); `count + -1` works today;
  - a function without `-> ()` whose last line is a command becomes String-valued, so its last output is captured and loses the trailing newline.
- **Smaller fixes:**
  - Boolean `&&`/`||` as values;
  - a portable `byte_length` (`wc -c`);
  - match arms that hold statements (print, then exit);
  - `std::process::exit` printing an extra "command failed" line.

## vize

### Foundation PR #13729 (`ci/ush-foundation`): auto-merge is ON

It adds:
- the flake input `ush` pinned to v0.19.0, built in the dev shell with `doCheck = false`;
- `tools/support/ci/ush-programs.ts`, the compile and freshness check inside `repository-config.ts`, which requires `ush --version` to equal the flake pin;
- the `.github/actions/setup-ush` installer, written in ush, holding a copy of the release's whole `sha256sums.txt`;
- the `Actions.pkl` helpers `workflowProgram`, `actionProgram` and `wslProgram`;
- the `11-docs-build` pilot (4 programs) and `tools/ci/lib/github.ush`;
- `stage.ts`, which now copies `compiled/` subdirectories;
- the contract-test helper `tests/tooling/support/automation-source.ts`: a `sh …/compiled/x.sh` step is inlined as its `.ush` source, and `automationShellPath` maps that text back to the build.

The last push is `5aa9c27c95`, which re-pins the custody closures of the four workflows whose programs include `stage.ts`: 04-content-mapper-conformance, native-beta-diagnostic-dto, typescript-plugin and typescript-provider-vsix. Before that, every "Affected tooling" failure was `<owner>: executable source changed`, from that closure.

- **If CI still fails:** look for other digest lists that pin `stage.ts` (`rust-cache-docs-wasm-consumers.mjs` producerHashes already has its new digest).
- **Generator check:** run `node tools/commands/generate/repository-config.ts --check` with the pinned ush on PATH. The release binary from `gh release download v0.19.0 -R ubugeeei-prod/ush` works; verify it with `shasum -a 256 -c`.
- **Regenerating a changed workflow YAML:** run `vp fmt` on it, because the generator re-renders the whole file in its own layout.

**Collision with J1b #13726** (from vize-18):
- #13726 renames native-beta-diagnostic-dto to `05-native-diagnostic-dto` and moves the closure readers to the new name, with a fallback to the old recorded name.
- If #13726 merges before #13729, the native-beta-diagnostic-dto re-pin in #13729 conflicts or goes stale. Rebase #13729 onto main, then re-pin that closure under `05-native-diagnostic-dto`: run `refresh-history` for that owner in `tools/support/ci/workflow-program-history.json`, and update the custody rows.
- typescript-plugin and typescript-provider-vsix aren't renamed yet (J1c is deferred), so their re-pins stand.
- Afterwards, check that every closure in `workflow-program-history.json` matches the files on the branch. The merge queue rebases, so check again after #13726 lands.

### Conversion branches: pushed, no PRs yet

They are based on the OLD foundation head `cdab43e8a4`. Open their PRs only after #13729 merges, because stacked PRs become UNMERGEABLE in the merge queue. Then rebase each onto main, run the generator `--check` and the related `tests/tooling` tests, open the PR and turn on auto-merge.

| Branch | Converted | Notes |
| --- | --- | --- |
| `ci/ush-08` | 34 of 34 | Includes the v0.20.0 pin cherry-pick, the apt hook via `std::path::script`, and babel's `sha256sum $tarballs`. |
| `ci/ush-05-06` | 41 | Includes the pin cherry-pick. 06-native-smoke (9) is skipped for custody: its closure is in `workflow-program-history.json`. 05-lsp-vs-volar `plan` and `measure` wait on #198. |
| `ci/ush-07-11` | 40 of 48 | Includes the pin cherry-pick and 07-canonical-corpus's prebuild via `detach`. Seven 10-app-e2e programs are skipped for custody (`rust-cache-docs-wasm-consumers.mjs` and `byte-custody.tsv`). |
| `ci/ush-pin-0.20` | — | Only the v0.20.0 pin commit, `c864b7cc33`. All three conversion branches already contain it. |

Every program was compared old against new under stub commands. The agents' commit bodies record each behaviour choice; the main ones:
- `fail` with a message instead of a silent `test`;
- Node expands globs itself, since ush doesn't;
- the `spawn` and `defer` cleanup replaces the old trap.

The agents also quoted `"-0"` and `"0755"` as a stopgap before #198. Both forms stay correct after it.

### After those merge

1. Make a follow-up PR:
   - pin v0.20.1;
   - move `.handoff/ush-migration/plan.ush` and `measure.ush` into `tools/ci/workflows/05-lsp-vs-volar/scripts/`;
   - delete their `.sh.pkl`/`.sh`;
   - change the steps to `ci.workflowProgram`;
   - update the measure assertions in `tests/tooling/lsp-vs-volar-evidence.test.ts`.
2. **Groups 03 and 04** (03 has 128 programs, many custody-bound) are next. Brief the agents with `.handoff/ush-migration/conversion-brief.md`: one agent per group, 3 at a time, local commits only, no PRs until the base merges.
3. **Custody-bound programs need the user's decision:** 06-native-smoke, 10-app-e2e ×7, and much of 03/04.
4. Then the nix shellHook scripts, `tests/tooling/programs`, `.github/actions` scripts and `crates/*/tests` `.sh`. Finally remove `ShellScript.pkl` and the Pkl "shell" kind.

### Coordination

- vize-18 (the cleanup/J session) has gone quiet.
- J1b #13726 is in the merge queue; check it with `gh pr view 13726 --json state`.
- J2, branch `ci/move-rename-workflows-j2`, renames 19 workflows in groups 01, 03 and 07; its PR comes after J1b. Convert a group only after its J move has merged.
- vize-18's handoff: branch `handoff/cleanup-2026-11-26`, file `.handoff/cleanup-2026-11-26.md`.

### Pitfalls I hit

- **Shared git stash:** never use a bare `git stash`. I did once, in an automated loop, then verified that entry against its pushed branch and dropped it.
- **Shared disk:** keep at least 30 GB free. On 11-26 the disk reached 100% because of other worktrees' `target/` directories.
- **CI queue:** after an overload someone bulk-cancelled runs; re-run once.
- **Merge commits from the owner:** the owner sometimes merges `main` into a branch on GitHub. Fetch before you push.
