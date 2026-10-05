# Current-source lint attribution probe

Paired issue: [#7856](https://github.com/ubugeeei-prod/vize/issues/7856).
This consumes the read-only metadata parent #7860 in a native Stack. The
typechecker instrumentation follows this layer; no layer is auto-merged.

## Frozen authority

The public benchmark at `5489aee433cd1054b9d72973457498544da7c467` uses Vize
0.429.1. Both CLI thread profiles cover 200/200 files but fail all eleven
mandatory lint validity pairs. The two displayed notes are a truncated
summary. These observations concern the old publication, not current Vize.

Use the exact hydrated submodule, full-source SHA-256 pins for the plants,
private child, CLI definitions, fixture generator, ANSI helper and MIT license,
and an unchanged, byte-exact extraction between `function cliDiagnostics(raw)`
and `async function runEntrypoint`. The extracted 1,413 bytes have SHA-256
`634d5b0faebd8cf8c51fd14b1c588b9eddc4be60bc129f11bd0b67f6ad1a4a1a`.
Only this parser is executed from the child; its other tool entrypoints and
timing helpers are not executed. The four directly imported upstream modules
depend only on Node builtins. Preserve the original suite `2026-09-12.2`, hash
`3a09ab6b2b314ce5be7d60bf17fa06941f895f25fe0e305adf88c0a1b0ee06a2`,
all eleven dirty/clean strings, attribution rules and judge failures.

## Same work and complete evidence

The untouched `prepareLintDir` generates the original `n200` configuration
from an empty input directory. Independently authored complete config bytes
are pinned in the contract before current runtime execution. Copy exactly
`eslint.config.mjs`, `biome.json`, `.oxlintrc.json` and `package.json` into
fresh dirty/clean work roots with the original empty `.git` marker. No Vize
configuration is introduced. Inputs are the eleven original nested plants;
this is a correctness probe, not a rerun of the 200-file timing benchmark.

For `vize-lint-1t`, execute `vize lint .` with `RAYON_NUM_THREADS=1`; for
`vize-lint-max`, use the default pool. Use the receipted current source-built
CLI, never a global/package binary. Preserve complete stdout, stderr, exit,
signal, source revision and binary hash for each dirty/clean invocation.
Run supplemental `vize lint . --format json` in each same cwd under the same
environment. Retain its entire ordered file/message arrays, full ranges and
counts. Record additional discovered config files explicitly. Check all
eleven file rows and every input/config byte before and after each run.

Apply the untouched original judge separately to the original human parser
results and the machine diagnostics. JSON does not replace the benchmark
validator. Preserve failures, including an invalid structured path, wrong
file/line/concept and a clean twin retaining the concept on another line.
An authored graphical grammar example tests the private parser boundary;
it is not a runtime observation or reference collected from current output.

## Delivery and remaining work

Existing source Check and protected-queue tooling hydrate only this exact
submodule when the new test is selected. Existing full Check also hydrates it.
They reuse their own source-built executable and existing differential
artifact retention. Missing source, receipt, helper or output fails closed.
No local Cargo build, upstream installation or mutating old-65c ensurer runs.

Current runtime, adapter cause, actual merge and publication remain pending.
An observation can retain failing judges without claiming rank qualification.
Keep all published ranks, old 150-case typecheck replay, legacy fix-history,
native migration credit and instruction budgets unchanged. Composition API
computed side-effect coverage is a separate narrow correctness slice; a
parser mismatch alone cannot prove missing rules or justify changing output.

The first source execution at `16b36435` failed instrumentation: the unchanged
parser's authored graphical example captures `╭─[nested/...`, and the first
CLI call creates `node_modules/.vize/vize.config.schema.json`. All eleven
inputs and four configs retained their original bytes. Keep that failed run
as historical evidence. Correct the example from the fixed regex contract;
compare the generated schema with the source file included by
`crates/vize/src/config.rs`, never with captured output. Retain every complete
before/after inventory, require that one exact generated file after each
call, retain its complete actual bytes separately for every call and reject
all other additions or changes. Repaired-head runtime is
pending; neither this repair nor JSON grants rank or native migration credit.
