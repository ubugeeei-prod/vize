# Selected linter history requirements through the current public API

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).
Source parent: `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`.

The issue's 499 touch commits / 255 fixes is the historical design-session
inventory, not a completion counter. The later read-only full-history audit
has 503 nonmerge touch rows, including behavior changes without fix titles,
uncertain compound changes and separately reviewed merge deltas. That audit
and all existing prepared differential packs remain separate from this pack.
No complete-history or native acceptance credit is granted by these cases.

## Decision

Add complete current public API snapshots for selected missing or partial
historical requirements, preserving the existing semantic assertions. A
diagnostic-count witness alone does not cover messages, help, authored spans,
labels, offered edits or the result after application.

`crates/vize_patina/tests/lint_history.rs` exercises the real public
`Linter::with_preset(Incremental).with_enabled_rules(...)` entry point with
English locale and full help. Its strict JSON cases retain full historical
SHAs, filenames, complete UTF8 source bytes, entry points and actual public
Vue-version / Vapor settings. Snapshot the complete `Case` and `LintResult`
Debug output without filtering, sorting, position construction or message
normalization. The API identifies diagnostics by `rule_name`; there is no
additional diagnostic code to invent.

Every genuine offered `Fix::apply` runs independently on the original input,
followed by the same public entry point and options. A case with no offer
actually queries its unchanged bytes again. A panic, count mismatch, missing
case or snapshot mismatch fails. A second complete observation must equal
the first before it can match the frozen golden.

| Historical commit | Selected scope | Control |
| --- | --- | --- |
| `cba6fd5bbb7519696059416fd77aa1ba3192cba3` | Typed and untyped `ref(null)` text inside strings, paired with static template ref names | A real typed `ref()` call with the same template name reports |
| `731cce813a41207950923d89099ac2c87047c0b6` | `v-for="items"` through the public template entry | A valid `item in items` expression remains clean |
| `f9fa82f7867e3a9373a8d0ee30162c6947c7c113` | Explicitly enabled slot/shorthand rules respect Vue 2 compatibility; explicitly enabled nextTick respects Vapor false | Vue 3 slot/shorthand and unspecified Vapor report real findings |
| `634f636969b451c805ad4874c0567d64c8e8f850` | Empty static-class expression; genuine fix with a Japanese/emoji/CRLF prefix | Complete applied source and subsequent full lint result |

The string cases preserve the historical text and additionally supply an SFC
template name, because the current rule requires template pairing. They are
current successor requirements, not a replay of the obsolete script-only
eligibility policy. The static-class control strengthens current fix/offset
evidence; it does not certify every CSS, Musea or opinionated panic path in
that compound commit. The no-separator case does not certify every production
path changed by its compound workspace panic repair.

Input lives inside JSON, rather than new `.vue` files, to avoid unintended
changes to every global fixture walker. These are real authored source bytes
executed by the Rust integration test. Shared differential registration must
point to this input and exact oracle; it must not invent additional duplicate
eligible cases or claim native handling from the current product path.

## Actual local capture

The exact locked, offline command was:

```sh
CARGO_TARGET_DIR=/tmp/vize-patina-history-target-20260927 CARGO_INCREMENTAL=0 cargo test --locked --offline -p vize_patina --test lint_history --no-run --message-format=json-render-diagnostics
```

Rust 1.98.0 / aarch64-apple-darwin built the initial selected test executable
in 51.18 seconds. A lint annotation correction rebuilt that target in 2.52
seconds; the receipt and matching executions bind this final source and binary.
The target is task-owned and seeded by explicit APFS
`clonefile` calls that fail on error, with no physical-copy fallback or shared
target writes. The first actual execution froze 13 full snapshots; a separate
execution with snapshot updates disabled matched every golden. Each execution
also compares two complete observations per case. No test or case was ignored.
The [capture receipt](./2026-09-27-lint-history-current-api-receipt.json)
binds the parent, prepared source, exact UTF8 inputs, executable, Cargo target /
profile / features and raw local build/execution logs by full SHA256.

**The static-class control exposed an existing autofix bug.** Its real offer
replaces the SFC binding through one byte beyond the closing quote, consuming
`>` and producing `<div class="static-class"</div>`. The subsequent public
lint result contains three parser errors. The snapshot records the actual
offered edit and broken output; it does not describe a successful repair.
This static-class requirement is **unsatisfied**, never covered or accepted.
Keep this observation immutable in the history pack. [#6920](https://github.com/ubugeeei-prod/vize/issues/6920)
tracks a separate behavior-fix
change needs its own corpus fixture and a reviewed valid post-fix contract;
this preparation changes no production code.

## Remaining work

- Account for every actual historical behavior requirement, including
  non-fix titles, merge resolutions, renames, superseded requirements and
  uncertain compound branches. Keep partial fields and unrun witnesses visible.
- Register complete input/config/oracle identities with the shared #6891
  runner and retain the independent prepared raw lint captures. A committed
  current API snapshot is useful evidence, not an automatic whole-row closure.
- Repair the static-class autofix closing-tag boundary separately, including
  bare template and SFC authored offset controls and exact resulting bytes.
- Verify a fresh source-built Actions run and merge queue result before
  publication or product-path replacement. Run full native comparisons only
  when a genuine native adapter exists.
- Keep #6881 open until its whole-history condition is actually satisfied.

No production behavior, parser, dependency, native adapter or product path is
changed by this preparation.
