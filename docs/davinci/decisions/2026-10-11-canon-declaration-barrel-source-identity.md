# Declaration barrels keep projected Vue source identity

Related: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879), and
the [original App provider baseline audit](./2026-10-11-app-check-provider-baseline-audit.md).

## Reproduction and scope

The full retained Nuxt UI `useToast.ts`, `useFormField.ts`, and
`useContentSearch.ts` import named types from existing `.vue` components.
The prepared `.nuxt/imports.d.ts` re-exports these authored TypeScript modules.
The original App capture establishes their bytes and membership, but does not
prove the diagnosing native configuration or loaded-file identities.

An independent native 7.0.2 literal replay uses the complete unchanged hosted
minimal SFC projection (SHA256
`b906d5211ea2a279c504058c4b327ce5c12fb81f6844048b4eed1eae7db78353`),
the source-defined shared helper text, installed Vue, and independently authored
TypeScript callers. Four complete same-config CLI vectors demonstrate the
mechanism. An unprojected declaration barrel loads the authored TypeScript
module, reports TS2307 for its existing Vue target, and loses the wrong-number
TS2322. Projecting the declaration barrel resolves the existing projection:
the repaired caller is clean and the wrong-number caller reports exactly TS2322.
The replay retains the full stdout/stderr, config, native executable hash,
argv, and `--listFiles` response. It is a literal topology replay, not a run of
the local Vize factory and not acceptance of the original Nuxt App row.

The hosted normal-factory [run 38144542913](https://github.com/ubugeeei-prod/vize/actions/runs/38144542913/job/114485171697)
at immutable source `223e4831c4b572fc4af10ba609cdc2ae0f6348b3` compiled,
listed, and executed the required native law. All four outcomes were retained
before its failure. Both direct cases used projected TS: the repaired vector
contained only the absent neighbor TS2307 and the broken vector additionally
contained TS2322 at authored App line 3, column 12 (zero based). Both declaration
barrel cases loaded authored TS instead and contained only the absent neighbor
TS2307, losing the wrong-number TS2322. The same-client source-file query
succeeded and normalized strict was true. The law failed on this concrete
authored-source identity leak; it was not skipped or accepted as green.

## Decision

Source-backed first-party declaration barrels join the existing reachability
walk. Both exact alias discovery and reached-declaration policy must agree:
the alias fast path previously rejected concrete declaration targets before
the walk could inspect their source edges. A per-walk cache determines whether
a declaration graph reaches supported first-party source, handles cycles, and
honors unsaved declaration overlays. Registration, final declaration rewriting,
and materialization then use the existing graph and source maps.
The cache reuses classified children and stores every visited node only after a
complete negative closure. Early success classifies only the queried root, so
unproved branches never acquire a positive answer. Each new walk starts fresh
and observes its own overlays; repeated aliases of a negative declaration cycle
do not repeatedly parse the entire cycle.

Terminal ambient declarations keep their authored identity and existing
tsconfig authority. Published declarations inside `node_modules` keep the
package resolver path. This preserves the declaration identity law from #2047;
the change does not add another pipeline stage or modify authored files.

Production scope is `batch/virtual_project/dependency_scan.rs` plus
`batch/virtual_project/dependency_scan/declaration_sources.rs`. The existing
`import_rewriter.rs` and `dependency_scan/resolution.rs` are unchanged.

The tracked whole-project native corpus compares direct TypeScript imports and
an initially unscanned declaration barrel, each with a wrong-number caller and
its repair. The normal public batch and actual generated Corsa project preserve
complete raw/mapped/public vectors, installed Vue identity, selected config,
normalized options, roots, full projections/source maps, and the same diagnosing
process's `getSourceFileNames` request and response. Its absent Vue neighbor must
retain TS2307. A repair may remove only TS2322, and a registered source's authored
identity must not leak into native through the barrel.

## Qualification pending

TODO: preserve the hosted normal-factory red-before artifact and bind a passing
green-after packet to its immutable source index before granting native correctness credit.
Keep the original App diagnostic goldens unchanged. Even a passing minimal
native law still requires the full prepared Nuxt App row and provider/config
authority to classify the original three diagnostics.
