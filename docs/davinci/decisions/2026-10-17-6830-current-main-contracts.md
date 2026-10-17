# Preserve current source contracts after the main merge

Refs [#6830](https://github.com/ubugeeei-prod/vize/issues/6830),
[#9200](https://github.com/ubugeeei-prod/vize/pull/9200).

Full Check [38993084178](https://github.com/ubugeeei-prod/vize/actions/runs/38993084178)
at `2b0cf6c084ea0d4ab0a3dce33cfb9ebf0e7c8db2` exposed three contract
failures after main added the #9312 build evidence checks and #9251 retained
template analysis. All three failures reproduce locally. Repair their source
contracts without changing workflow selection, product fixtures or budgets.

The planner contract now compares the complete five-test command as argument
tokens, including the existing build evidence test. It still requires the same
tooling-input condition and install-free Node/offline metadata preparation.

Move the two retained assignment/scope walkers byte-for-byte in a separate
commit, then use ordinary `retained/{assignments,scopes}.rs` module discovery.
Their Rust module names, parent visibility and implementations stay identical.
Update both source-custody includes; the module-layout guard remains unchanged.

The declaration CLI contract compares all 19 accepted IDs in original order
against the complete 20-case primary sequence, excluding only the explicitly
unqualified template-this observation. Its actual CLI runner, complete JSON
packets, stderr/status checks and source/config custody remain unchanged.
All original fixture files are retained byte-for-byte.

The existing bounded parser-entry source guard now reports the complete
`(module, forbidden spelling, original byte offset)` violation vector and
requires it to be empty across the same six collection/walker files. Import,
direct-call and fallback controls require exact vectors; spaces/newlines cannot
hide a forbidden spelling. The dynamic parse counters and original AST-pointer
checks remain unchanged. This textual boundary still does not claim to prove
an arbitrary future call graph. The assertion linter and allowlist are unchanged.

All 14 focused workflow/layout/assertion contracts and all 37 tests in the
actual five-test planner command pass. The exact source-guard functions and
their six real inputs also compile and pass in a dependency-free Rust probe;
this is not a full product build. Formatting and the focused TypeScript lint
pass. Fresh combined-source Actions must execute the retained Rust ownership
and complete CLI observations before qualification is complete.

Merging frozen main `8df6e1fe3e8e91fe2a0adbb4633d1de38e81e506`
adds #9324's preset-warning source audit. Its historical nextest hash rejects
the independent PR scheduling priority override: the merged configuration is
5,562 bytes, whereas the audited original is 5,213 bytes. Keep both current
overrides. Archive main's exact original configuration as
`tests/_fixtures/lint-preset-default-warnings/nextest-8df6e1.toml`, whose SHA-256
is `d7aca4d0556a5ab2a4ef81c16d022883aaf2e17ddc48de3235e9cbe028596e86`.

Only that nextest row now audits the authenticated historical fixture. Its
existing finite inverse, before/after identities and mutation controls remain
unchanged. The successor manifest, seven other live source inputs, membership,
corpora and complete authored CLI packets remain byte-identical. No current
source is stripped or rewritten to claim historical identity.

The separate live TOML assertion requires exactly one matching warning-output
override, its complete original package/binary/test filter, and the boolean
`junit.store-success-output = true`. Ten mutation controls reject a missing or
duplicate override, broad or changed package/binary/test filters, false/string/
missing retention values, and moving the override out of the default profile.
Existing PR/full selection controls continue to check the actual configuration.

All seven preset source tests and twelve existing planner, full aggregation and
PR/full runtime-envelope controls pass. Formatting and focused lint pass. The
native preset tests and CLI execution remain unchanged; these local source
checks do not replace their fresh combined-source Actions qualification.
