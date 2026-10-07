# Component model modifier diagnostic ranges

Issue: [#7950](https://github.com/ubugeeei-prod/vize/issues/7950).

`defineModel` publishes its modifier union correctly, but component template
checks synthesize an object such as `{ "bogus": true }`. The generated value
has no authored initializer in the `v-model` attribute, so the old whole-attribute
mapping clamped TS2353 to the byte after its expression. The existing CLI test
also asserted that incorrect position.

## Decision

Collect the authored modifier AST spans once with the existing per-file Canon
check tables, skipping the AST walk for templates whose source contains no `v-model`. Keep these private to emission rather than adding a field to
Croquis's public `PassedProp` struct. Both the individual prop assertion and the
generic whole-props literal map each generated key to its own modifier token.
String contents and quoted keys retain separate sub-spans for editor requests
and diagnostics. Match both the synthesized prop name and object value before
using the map: an ordinary model expression or explicit modifier prop must keep
its own mapping.

The new corpus is
[`tests/fixtures/typechecker/model-modifier-ranges`](../../../tests/fixtures/typechecker/model-modifier-ranges).
It covers default and named models, multiple valid modifiers around an invalid
one, multiline attributes, clean explicit modifier props, native modifiers,
and repair. The generator contract covers CRLF, Unicode, nonzero SFC template
offsets and exact start/end byte ranges in both generated check forms.

## External RED evidence

The immutable published `vize@0.432.0` CLI, TypeScript `7.0.2`, and Vue `3.5.43`
were run on the exact corpus inputs before evaluating the fix:

- `App.vue`: one TS2353 at `9:30`; the authored `bogus` token starts at `9:18`.
- `NamedApp.vue`: two TS2353 rows at `10:39` and `10:42`; the one authored
  `bogus` token starts at `10:24`. Giving both generated checks the same exact
  authored span also permits existing authored diagnostic deduplication.

The current source baseline still contains the same coarse synthetic value
mapping and the CLI assertion at the byte after the attribute. Terminal
exact-head Actions and protected merge-queue validation are required before
completion. This change makes no performance improvement claim: the private
mapping table adds no backend process, serialized representation or pipeline
stage. It does not replace Canon with Davinci, close #6879, or complete the
project-wide #3984 / production-readiness #3957 contracts.

First source run `37589353338` rejected the new unit test's stale
`VirtualTsOutput.mappings` field name before execution. Use the existing
`output.mapping.spans()` accessor without changing any range assertions.
The same run's production npm audit separately rejected the newly published
`@modelcontextprotocol/sdk` advisory `GHSA-6qxp-vccf-f47h` (requires 1.31.0);
that shared dependency repair is owned by the root delivery lane. Retain both
failed logs and require fresh exact-head source/security proof.

Successor tooling rejected stale Canon consumer inventories and the direct
`Croquis.component_usages` access used solely to skip collection. Use a cheap
`v-model` presence guard on the existing template source instead, retaining
every demand-census gate and adding no Croquis consumer. Regenerate only the
Canon consumption and typechecker migration shards for the new private
collector/test imports. All production typechecking and regression assertions
remain unchanged; fresh source/security Actions are still required.

Ordinary PR Rust shards explicitly disable native typechecker execution. Run
the modifier CLI corpus in the existing native-phase PR step with
`VIZE_TEST_REQUIRE_TSGO=1` and an unset disable flag; its exact diagnostic and
edit/repair assertions must actually execute before acceptance. Extend the
workflow paths to cover standalone CLI/corpus changes. Generator byte-range
assertions remain in the ordinary Rust shards.
