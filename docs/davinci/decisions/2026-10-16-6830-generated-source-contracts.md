# Preserve authored rule changes in generated contracts

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
This repair includes main through
[`678ea68ca34ad88206670ed00749c6448cdf1b9c`](https://github.com/ubugeeei-prod/vize/commit/678ea68ca34ad88206670ed00749c6448cdf1b9c).
Full tooling qualification exposed generated documentation drift and assertions
that still described earlier product behavior.

The `valid-v-on` examples and custom modifier settings, numeric-key selection
and fixes, exact-listener overlap rules, and lexical template-shadow boundaries
had been corrected directly in reference pages. Preserve those corrections in
the existing authored examples, explanations and locale notes, then run the
official rule-page generator. English and Japanese references and all five
localized category pages now retain the current examples and support limits.
Generated implementation links and fixability also follow current Rust metadata.
The whole-page, bilingual source and deterministic-generation guards remain.

Historical preset packets keep their original bytes and hashes. The existing
composition test still compares complete packets after reversing the seven
approved byte/UTF-16 range changes. It now also verifies the intentional
[#9213](https://github.com/ubugeeei-prod/vize/pull/9213) changes: five numeric-key
cases have exact named-key fixes in both default and essential diagnostics,
and the combined numeric case reports only the first numeric modifier in its
native and LSP results. Fix text, edit ranges, messages and diagnostic counts
are checked before the inverse; other fields and mutation refusals stay intact.
Neither historical nor current fixture packets were rewritten for this repair.

Semantic assertion repairs inspect the actual diagnostic file, code, message,
range and count, complete LSP hover/result packets, the native test result, and
the authored span selected by a provider diagnostic. They retain the original
runtime scenarios and strengthen the previous substring assertions. The stage
alias remains the route for L0 storage. These runtime expectations still need
qualification by the final combined Actions run.

The accompanying [v-on corpus repair](./2026-10-16-6830-v-on-corpus-source-sync.md)
regenerates the existing per-area inventories after the authored sources settle.
It preserves natural occurrences, exact inventory equality and the two-entry
storage bounds. This batch does not apply a blanket golden update, add an
assertion-lint exception, or remove a product guard.

The following checks passed locally: 20 existing docs/preset contract tests and
the official generator's consistency check. The docs-all test itself is unchanged.

```sh
node docs/scripts/generate-patina-rules-page.ts --check
node --test tests/tooling/docs-all-lint-rules.test.ts \
  tests/tooling/docs-rule-options.test.ts \
  tests/tooling/lint-deprecation-source-composition.test.mjs
```

The corpus decision records its separate 15-test verification. These local
checks establish source consistency; the final combined Actions qualification
remains the acceptance boundary for the runtime assertions and integrated tree.

The first integrated [native Canon job](https://github.com/ubugeeei-prod/vize/actions/runs/38924018440/job/117050971900)
disproved the new empty-stderr assumption. Its complete syntax/semantic corpus
passed, but native TypeScript 7.0.2 emitted `error handling method 'exit': EOF`
for each of its two diagnostic-session exits. A separate direct native LSP
initialize/hover/shutdown/exit replay also exited zero with that exact line;
Corsa inherits the child's stderr. The handoff assertion now permits only that
complete provider line, at most once per broken/repaired session. Unknown
output and every Vize degradation warning still fail. Successful execution and
the one-test, zero-failure result remain mandatory; no product diagnostics,
transport failure checks or runtime behavior changed.
