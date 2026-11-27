# Vue Language Tools diagnostic corpus

The complete `test-workspace/tsc` source corpus and its shared config are copied
from [vuejs/language-tools](https://github.com/vuejs/language-tools/tree/88e8500c1e5f1b29d80f42c8ca065cc9cbd56899),
revision `88e8500c1e5f1b29d80f42c8ca065cc9cbd56899`, under the included MIT license.
The original `packages/tsc/tests/typecheck.spec.ts` supplies the expected diagnostic
codes and authored locations. `manifest.json` records all 230 projects, 599 files
and their SHA-256 hashes, and all 39 expected diagnostics. Error directives in
the original sources are preserved.

`tests/tooling/canon-upstream-diagnostics.test.ts` checks the inventory and runs
every project independently through the actual Vize CLI. It rejects missing and
unexpected errors; no fixture is excluded or assigned a passing baseline.
Compatibility work is tracked in [#6253](https://github.com/ubugeeei-prod/vize/issues/6253).

`documented-differences.json` lists the diagnostics Vize reports where the
oracle reports none, each under a reviewed reason and its tracking issue. They
are added to the exact expectation of their project rather than ignored: a
documented diagnostic that moves, disappears or gains a neighbour fails the
corpus, so the ledger can only shrink through a change that also edits it. It
currently holds one difference: the six `exactType(Component, defineComponent(...))`
identity assertions of `components` and `defineModel`, which require the SFC
default export to be the very type vue-tsc generates, where Vize exports its own
component contract.

The fixture workspace uses the upstream Vue `3.6.0-rc.6` and
`vue-component-type-helpers` `3.3.11` dependencies, pinned separately from the
application fixtures. The runner copies each workspace into an isolated temporary
directory and runs at most four compiler processes concurrently.

Before accepting corpus results, a mutation test verifies that the shared
`exactType` declaration rejects an intentionally wrong type. Deleting the shared
declaration must also report TS2307. This prevents failed module resolution from
turning assertions into `any` and producing a false passing score. Real LSP tests
cover dependency edits and restoration on close, plus diagnostic directive
ownership, repairs, unused expectations, UTF-16 positions and document versions.

Undocumented compatibility failures remain hard failures in the full runner. The
source inventory test and mutation test can pass while the full corpus still
fails; that is not evidence of full upstream compatibility.
