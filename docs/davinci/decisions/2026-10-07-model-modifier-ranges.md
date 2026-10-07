# Component model modifier diagnostic ranges

Issue: [#7950](https://github.com/ubugeeei-prod/vize/issues/7950).

`defineModel` publishes its modifier union correctly, but component template
checks synthesize an object such as `{ "bogus": true }`. The generated value
has no authored initializer in the `v-model` attribute, so the old whole-attribute
mapping clamped TS2353 to the byte after its expression. The existing CLI test
also asserted that incorrect position.

## Decision

Collect the authored modifier AST spans once with the existing per-file Canon
check tables. Keep these private to emission rather than adding a field to
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
