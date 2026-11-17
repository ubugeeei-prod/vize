# Handoff: davinci-foundation-patina-3 (#6881 linter fix history, batch 4)

Stacked on davinci-foundation-patina-2 (PR #13244, in the merge queue at handoff). The branch has two wip commits: the raw registration and v1/v2 with pins.
Locally: current-api grows 693 -> 832 cases (planned 863). Ledger: 193 oracle-pinned, 27 no-lint-behavior, 38 not-admitted. Linter tooling tests: 34 passed, 0 failed.
Native-handled pins went from 23 to 28: the five h14-definition-name-*-939 cases are handled by the native component-name rule.

## Next
1. After #13244 merges, rebuild on origin/main: `BASE=origin/main patina-pipeline.sh <counts.json> linter-specs4`. The specs and counts were in the scratchpad and are lost, so recover them from the cohort on this branch, or squash the two wip commits and rebase, resolving conflicts in the pinned files by re-running the finalize step.
2. Squash into one commit, e.g. "test(patina): pin N more linter fix-history commits as current-API oracles", and open a PR.
3. Parked rows: bd6f865e26 (form-control-has-label cases are also observed by the native input-label contract, which admits only six wires) and c3f6780cb9 (native observer mismatch on bound access-key/autofocus).

### patina-pipeline.sh
```bash
#!/bin/bash
# usage: patina-pipeline.sh <counts.json> <specdir>...
# Rebuilds the linter history registration from ${BASE:-origin/main} state: register
# with observed counts, regenerate snapshots, finalize pins.
set -e
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad
cd /Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-ac0b79bf685e68587
counts=$1; shift
specs=()
for d in "$@"; do for f in "$S/$d"/*.json; do specs+=("$f"); done; done
# put main's cohort/oracle pins back into the contract test before resetting
node -e '
const fs=require("fs");const {execFileSync}=require("child_process");
const cur=JSON.parse(fs.readFileSync("tests/_fixtures/differential/linter/manifest.json","utf8"));
const main=JSON.parse(execFileSync("git",["show",(process.env.BASE||"origin/main")+":tests/_fixtures/differential/linter/manifest.json"],{encoding:"utf8",maxBuffer:1<<28}));
const pick=m=>[m.cohorts.find(c=>c.id==="current-api").cases.sha256,m.oracles.sha256];
const [a,b]=[pick(cur),pick(main)];const f="tests/tooling/differential-linter-sfc-contract.test.ts";
let t=fs.readFileSync(f,"utf8");t=t.replaceAll(a[0],b[0]).replaceAll(a[1],b[1]);fs.writeFileSync(f,t);'
git checkout ${BASE:-origin/main} -- crates/vize_patina/tests/fixtures/lint-history/cases.json crates/vize_patina/tests/lint_history.rs tests/_fixtures/differential/linter/manifest.json tests/_fixtures/differential/linter/oracle-index.json tests/tooling/differential-linter-api.test.ts
git ls-files --others --exclude-standard crates/vize_patina/tests/snapshots | xargs rm -f
for f in $(git diff --name-only ${BASE:-origin/main} -- crates/vize_patina/tests/snapshots); do git checkout ${BASE:-origin/main} -- "$f" 2>/dev/null || rm -f "$f"; done
if [ -n "${BASE:-}" ]; then
  git checkout "$BASE" -- docs/davinci/plan/linter-fix-history.tsv tests/_fixtures/differential/linter/history-native-reasons.json crates/vize_patina/src/rules/a11y/form_control_has_label/native/tests/support.rs crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option/native/tests/support.rs
else
  cp "$S/linter-fix-history.tsv" docs/davinci/plan/linter-fix-history.tsv
  echo '{}' > tests/_fixtures/differential/linter/history-native-reasons.json
fi
# the contract test's pins are recomputed by finalize from the reset manifest
git show ${BASE:-origin/main}:tests/tooling/differential-linter-sfc-contract.test.ts > /dev/null
node tools/commands/fixtures/linter-fix-history-case.ts register --counts "$counts" "${specs[@]}"
INSTA_UPDATE=always CARGO_BUILD_JOBS=4 nix develop --command bash -c 'cargo test -p vize_patina --test integration lint_history -- --test-threads 4' > "$S/lint-pipeline.log" 2>&1
grep -E "^test result" "$S/lint-pipeline.log"
nix develop --command node tools/commands/fixtures/linter-fix-history-case.ts finalize "${specs[@]}" 2>&1 | tail -1
vp fmt tools/commands/fixtures/linter-fix-history-case.ts tests/tooling/differential-linter-sfc-contract.test.ts tests/_fixtures/differential/linter/history-native-reasons.json > /dev/null
```

### lintdump.py (local only)
```python
#!/usr/bin/env python3
# usage: lintdump.py apply|remove  (local-only: record observed counts instead of asserting)
import sys
p = '/Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-ac0b79bf685e68587/crates/vize_patina/tests/lint_history.rs'
s = open(p).read()
pairs = [
    ('    assert_eq!(initial.diagnostics.len(), case.diagnostics, "{}", case.id);\n',
     '    let localdump = std::env::var_os("VIZE_LOCAL_LINT_COUNTS");\n'
     '    if localdump.is_none() { assert_eq!(initial.diagnostics.len(), case.diagnostics, "{}", case.id); }\n'),
    ('    assert_eq!(applications.len(), case.fixes, "{}", case.id);\n',
     '    if let Some(dir) = localdump {\n'
     '        std::fs::create_dir_all(&dir).unwrap();\n'
     '        std::fs::write(std::path::Path::new(&dir).join(format!("{}.json", case.id)), format!("{{\\"diagnostics\\":{},\\"fixes\\":{}}}", initial.diagnostics.len(), applications.len())).unwrap();\n'
     '    } else { assert_eq!(applications.len(), case.fixes, "{}", case.id); }\n'),
]
if sys.argv[1] == 'apply':
    for a, b in pairs:
        assert a in s, a
        s = s.replace(a, b)
    # the registered-count pin and snapshots are skipped while dumping
    s = s.replace('        insta::assert_snapshot!(case.id.as_str(), first_bytes);\n',
                  '        if std::env::var_os("VIZE_LOCAL_LINT_COUNTS").is_none() { insta::assert_snapshot!(case.id.as_str(), first_bytes); }\n')
else:
    for a, b in pairs:
        assert b in s
        s = s.replace(b, a)
    s = s.replace('        if std::env::var_os("VIZE_LOCAL_LINT_COUNTS").is_none() { insta::assert_snapshot!(case.id.as_str(), first_bytes); }\n',
                  '        insta::assert_snapshot!(case.id.as_str(), first_bytes);\n')
open(p, 'w').write(s)
print(sys.argv[1], 'ok')
```

## Spec-agent brief (linter-spec-brief.md)
# Brief: prepare linter (Patina) fix-history case specs (#6881)

You prepare input data only. Do NOT edit the repository, run cargo, commit, push, or comment on GitHub.

Repo (read-only): /Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-ac0b79bf685e68587
Output dir (only place you write): /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/linter-specs/

Shell note: commands mixing `git` with `$(...)`, subshells or pipes containing the long path may be refused by a guard. Run git as plain commands from the repo dir (`cd <repo> && git show <sha>`), no command substitution.

## Background

Issue #6881: every historical Patina (linter) fix must be pinned as an executable fixture with its complete diagnostics and autofix behavior. The pinning mechanism is the existing "current-API" history cohort: `crates/vize_patina/tests/fixtures/lint-history/cases.json`, executed by `crates/vize_patina/tests/lint_history.rs`. Read both files and a few snapshots in `crates/vize_patina/tests/snapshots/integration__lint_history__*.snap` first.

Each case is ONE input linted with ONE rule:
```
Linter::with_preset(LintPreset::Incremental)
  .with_enabled_rules(Some(vec![case.rule]))
  .with_locale(Locale::En).with_help_level(HelpLevel::Full)
  .with_vue_version(vue_version)   // null, "2" or "3"
  .with_vapor_mode(vapor)          // null, true or false
```
then `lint_template(source, filename)` / `lint_script(...)` / `lint_sfc(...)` according to `entry` ("template" = the raw template markup without a <template> wrapper, "script" = a JS/TS module, "sfc" = a whole .vue file). Every offered fix is applied independently and re-linted. The complete Debug output is snapshotted, so a case pins exact messages, spans, help text, fixes and applied bytes.

## Your task

You get a TSV of ledger rows: `full sha <TAB> subject`. For EACH row:

1. `git show <sha> --stat` and `git show <sha>`. Find the behavior the fix changed and the regression tests it added (Rust tests under `crates/vize_patina/`, snapshot files, fixture files, CLI tests under `crates/vize/tests/`).
2. Express it as cases. Use the ORIGINAL test inputs where they exist (copy exact bytes; unescape Rust string literals precisely). Typical shape:
   - a case where the fix REMOVED a false positive → expect 0 diagnostics for that rule;
   - a case where the fix ADDED/CORRECTED a report (or its span, message, fix) → expect >= 1 diagnostic;
   - always at least one case that DOES report (a control proving the rule runs on this kind of input), even if the fix was a false-positive fix — use the historical test's "still reports" input or author a minimal one.
   Prefer `entry: "template"` or `"script"` when the rule works there; use `"sfc"` when the rule needs script+template context.
3. Write `<outdir>/<short-sha>.json`:
```json
{
  "commit": "<full 40-char sha>",
  "cases": [
    { "id": "kebab-id-<issue or topic>", "source": "<exact input>", "filename": "History.vue", "entry": "template", "rule": "vue/require-v-for-key", "vue_version": null, "vapor": null }
  ]
}
```
   - `id`: `^[a-z0-9]+(-[a-z0-9]+)*$`, globally unique: must not equal any id in `lint-history/cases.json` or any other spec in the output dir. End ids with the issue number when there is one.
   - `rule`: the exact Patina rule name (`category/name`, e.g. `a11y/alt-text`, `vue/valid-v-for`, `script/no-next-tick`); look it up in `crates/vize_patina/src/rules/` (the rule's `name`/meta) — a wrong name silently lints nothing.
   - `filename`: `History.vue` unless the rule depends on the name/extension (use `History.ts` for script entries in TS).
   - Write the JSON with a real serializer so bytes are exact.
4. Write `<outdir>/<short-sha>.notes.md`: the requirement (1-2 lines); per case the expected diagnostic count and fix count with what they should say (from the historical test), and where the input bytes came from (file + test name at the sha) or "authored".

If a row has no lint-observable behavior (CLI plumbing, config loading, output formatting, LSP-only, perf, refactor, test-only), write `<outdir>/NOBEHAVIOR-<short-sha>.md` listing what the commit changed and which existing test at current HEAD covers it (path + test name), if any. If the behavior exists but needs something this cohort cannot express (type-aware project, multiple files, config file, cross-file), write `<outdir>/SKIP-<short-sha>.md` with the reason. Don't skip merely because it is hard.

## Addendum (batch 2)

- Batch 2 writes into `/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/linter-specs2/` (not `linter-specs/`). Ids must also be unique against `linter-specs/*.json`.
- Type-aware (`type/...`) rules or anything needing Corsa/tsgo: SKIP (not deterministic in this cohort).
- When a later commit reversed the historical expectation, pin the current behavior only if it still exercises the fix's code path, and say so in the notes; otherwise leave that input out.

## Addendum (batch 3)

- Batch 3 writes into `/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/linter-specs3/`. Ids must be unique against `linter-specs/` and `linter-specs2/` too.
- Lesson from batch 1: an SFC whose `<script>` tag carries a non-standard attribute (e.g. `vapor`) makes the native observer refuse the descriptor; avoid such attributes unless the fix is about them (then say so in the notes).

## Addendum (batch 4)

- Batch 4 writes into `/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/linter-specs4/`. Ids must be unique against `linter-specs/`, `linter-specs2/`, `linter-specs3/` and the repo's `lint-history/cases.json`.
- Lessons so far: `script/prefer-use-template-ref` with `entry: "sfc"` cannot be registered yet (use `script` entry or SKIP); a control that is expected to report must really report under today's rule — read the rule's current code at HEAD, not only the historical test.

Reply at the end with a compact list: short sha -> case ids + expected counts, and NOBEHAVIOR/SKIP rows with one-line reasons.
