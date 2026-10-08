# Suffix path alias reachability

Issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).
Base: actual main `9d974d27b6823772d36a6855557cbbc27b3c1cd1`.

An App-only Canon scan missed an imported Vue child when tsconfig used
`"@/*.vue": ["src/*.vue"]`. The existing import-rewrite fixture correctly
preserved the authored `.vue` spelling but did not exercise dependency
registration. The prefilter searched for the literal `@/*.vue`, and the
resolver substituted only a terminal `*` in both key and target.

## Decision

Use the prefix before the wildcard for the existing dependency prefilter.
For a single wildcard, match both the key prefix and suffix and substitute
only the captured middle into the target, retaining its suffix. Thus
`@/components/Child.vue` reaches `src/components/Child.vue`, and a target
such as `src/*/index.ts` retains `/index.ts`.

Rank a wildcard by its prefix length plus the wildcard marker. A newly
supported generic suffix pattern must not displace an existing longer
trailing-wildcard prefix. This retains the exact previous scores of every
literal and trailing-wildcard key, including their tie and fallback policies.

This changes no pipeline stage, cache, public API, package boundary, candidate
probing, or target-array order. Existing literal and trailing wildcard
mappings retain their routes. Published package prefilters, physical
workspace identity, declaration-root policies and `.vue.ts` folding remain
in place. Multiple-wildcard patterns are outside this valid single-wildcard
slice.

## Regression evidence

The real native-feature Canon test compiled on Rust 1.99.0 against unchanged
base production code. Extending the existing
`batch_sfc_keeps_vue_suffixed_alias_key_import_authored` fixture to register
reachable dependencies failed because `DirectiveTable.vue` was missing:
**0 passed, 1 failed, 1755 filtered**, 0.02 s. The authored resolver laws
also failed before the fix: **2 passed, 3 failed, 1757 filtered**, 0.01 s.
The existing literal/trailing-wildcard control passed in that same run.

Independent review caught a mixed-pattern regression in the initial suffix
implementation: `@/*.vue` displaced the existing, more-specific `@/x/*`
because the suffix increased the generic key's full length. With both authored
targets present, the new law reproduced that wrong-child selection:
**5 passed, 1 failed, 1757 filtered**, 0.01 s. The corrected score must keep
the specific child in either alias input order.

The new resolver controls cover suffix matching, target suffixes, empty and
Unicode captures, unchanged target fallback order and JavaScript extension
substitution. The original alias fixture keeps its import-spelling assertions
and adds App-only child membership.

After the corrected score, all **6 resolver laws pass**, including both input
orders for overlapping patterns. All **8 existing alias-rewrite tests pass**,
including the extended child-registration law. These real Canon unit results
do not substitute for the required native diagnostic run.

The maintained consumer inventory initially exposed the new top-of-file test
module reclassifying a production Carton reference as test code. Moving only
that declaration below the production functions restores its classification.
The genuine new native-fixture test reference is recorded in the existing
typechecker shard; the derived **19-file inventory check passes**. The
configured production Canon Clippy check also passes with warnings denied.

The [authored corpus](../../../tests/_fixtures/differential/typecheck/suffix-path-aliases-3984/README.md)
drives the existing `scan_paths`/`check_project` APIs. It requires one authored
child TS2322, the exact child file and `(1, 6)` location, then no diagnostics
after repairing only that child and repeating the App-only cold scan with a
fresh checker. Persistent disk invalidation is outside this suffix law. A local
required-native attempt refused an unavailable Corsa executable; it is not a
native diagnostic qualification. Natural Actions with the existing
`VIZE_TEST_REQUIRE_TSGO=1` remains mandatory before queue admission.

## Remaining scope

#3984 remains open for its broader project, tsconfig, mixed-language and
backend requirements. This bounded fix does not qualify every supported
configuration shape. The existing exact-key versus equal-prefix wildcard tie
policy and fallback across competing patterns remain separate bounded TODOs
on #3984; this fix preserves those earlier routes.
The current frozen v0.437 release candidate excludes this change. Source
delivery and later publication must be recorded independently.

Reference: [TypeScript's `paths` contract](https://www.typescriptlang.org/docs/handbook/modules/reference#paths).
