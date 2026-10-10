# Selected alias collector

This additive [#3984](https://github.com/ubugeeei-prod/vize/issues/3984) corpus
qualifies the source CLI authored-file collector. The original
`path-alias-precedence-3984` eight-case inputs and oracles remain separate.

`selected-exact-missing` preserves all seven historical input files byte for
byte, including the 73-byte independently broken `wildcard.tsx`. The retained
historical source CLI reports App TS2307 and an unrelated wildcard TS2322;
stock TypeScript 6.0.3 and native Corsa 7.0.2 report only App TS2307. The four process records under `historical-red/` are explicitly labeled
portable derivatives of the original historical reports. Only three
machine-root prefixes are replaced with `__SOURCE_WORKSPACE__`,
`__NODE_RUNTIMES__` and `__TEMP_ROOT__`; executable, package, case and version
suffixes, hashes, input sizes, diagnostics and return values remain literal.
`historical-red/portable-correspondence.json` binds each original and derivative
by SHA-256 and byte length, lists replacement counts and records byte-exact
inverse-substitution verification against originals retained outside Git.
The derivatives are historical evidence, not byte-identical raw reports or
a new source run.

The eight literal cases require exact-key priority, selected-pattern-only
target probing, within-pattern target-array fallback and longest-prefix
selection. Broken exact and selected wildcard cases are repaired at the same
physical project path and rechecked by fresh CLI processes. The selected
wildcard control requires both App and producer TS2322 diagnostics; an
unselected broken generic must remain absent. Repairs claim only cold CLI
rechecks, not persistent sessions or invalidation.

Each `expected-report.json` is an authored complete stdout oracle, including
ordered files and diagnostics, program files/options/root and all counts.
Only its `__VIZE_ALIAS_ROOT__` compiler-path values bind to the actual project
root. The integration test retains all eight processes and every input before
it checks exit codes, empty stderr and complete stdout/report equality.
Original source and input bytes must remain unchanged by each invocation.

Run through the required-native Actions lane:

```sh
VIZE_TEST_REQUIRE_TSGO=1 cargo test --locked -p vize \
  --test check_canon_selected_alias_cli -- --nocapture
```

`VIZE_CANON_ALIAS_CAPTURE` chooses the external capture parent. This corpus
adds no parser/provider fallback and makes no installed-package, source-map,
default-mode, performance, or full #3984 completion claim.
