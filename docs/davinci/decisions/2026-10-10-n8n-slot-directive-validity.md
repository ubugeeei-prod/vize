# Bounded n8n slot-directive validity (2026-10-10)

Tracking: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
Baseline: actual signed main `26031a4fbb30a1e86511919bafc7035b08440ef3`.
This is local source preparation; hosted checks, protected delivery and public
installed acceptance remain required. It does not complete the adoption umbrella.

The reviewed correction is replayed on signed release-metadata main
`4c2bebb9a586e3eaab698b41e5b58ed5f8181852`. Its incoming metadata is preserved.
Original C observations and controls remain original evidence; hosted results
must identify the current source rather than treating those packets as a new build.

## Reproduction and decision

Current-source `Linter::lint_sfc` silently accepts these independently authored
inputs while pinned eslint-plugin-vue reports `vue/valid-v-slot`:

```vue
<FancyPanel v-slot>ready</FancyPanel>
<FancyPanel v-slot="">ready</FancyPanel>
<FancyPanel #default>ready</FancyPanel>
<FancyPanel #default="">ready</FancyPanel>
<FancyPanel v-slot.foo="slot">{{ slot }}</FancyPanel>
```

Require a value for a default slot on a component. Reject modifiers when the
slot directive has no argument. Both checks use the existing directive node in
the current rule callback. They add no parse, semantic demand, pipeline stage,
serialization, cache or cross-level dependency. Ordinary named-slot validation
keeps its existing path and diagnostics; slot-name classification is not repeated.
New diagnostics have complete directive byte spans and EN/JA/ZH messages and help.

Whitespace-only values, `{}` patterns, value-bearing component slots, and child
`<template #default>` or named templates without values remain unchanged. Existing
dotted named slots such as Vuetify's `#item.memo` remain supported. The independent
provider's default modifier policy differs from this established compatibility
choice; its separate `allowModifiers: true` observations are retained as controls,
not substituted for the current n8n contract's default configuration.

## Complete corpus and controls

The [47-input corpus](../../../tests/_fixtures/differential/lint/slot-directive-validity-8142/cases.json)
retains scripted and scriptless SFCs, Unicode and LF/CRLF coordinates, empty and
absent values, valid values, destructuring, child templates, named and dotted
slots, native-element location errors, mixed owner/child slots, duplicates,
conditional slot groups, dynamic arguments and neighboring scope controls.
The unchanged explicit n8n projection selects all 51 identities at error severity
and preserves its three literal options. Whole packets retain every foreign
finding, diagnostic span, message, label, help, fix, count and provider metadata.

The fresh isolated baseline and corrected source each execute every input twice.
Exactly eight packets gain one declared diagnostic, including scriptless and
Unicode/CRLF missing-value controls. The remaining 39 whole packets are identical.
Removing only each independently specified new finding restores the complete
baseline packet; the regression checks also address the literal authored bytes.
Original before/after packets, original observer bytes, complete baseline Git-tree
inventory and distinct executable hashes are retained. No stale shared-target
executable supplies the baseline or corrected observation.

The independent provider executes both default and allowed-argument-modifier
profiles twice: 188 complete observations. Versions are ESLint 10.4.1,
eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1 and TypeScript parser 8.65.0;
entry bytes are authenticated before replay. The official Vue base processor and
its comment/JSX infrastructure are preserved. Recording changes only the exact
ephemeral physical filename. Provider failures and incomplete execution remain
failures, with raw observations retained before assertions.

These are authored library API and pinned provider observations. Selecting all
51 rules here does not establish independent positive/negative semantics for all
51, native-product routing, real n8n package execution, installed CLI adoption,
whole-monorepo correctness or comparative speed.

## Explicitly unfinished neighboring defects

Fresh current-source execution confirms that a dynamic slot argument can still
reference bindings introduced by that same slot without a validity diagnostic:

```vue
<FancyPanel><template #[slot.name]="slot">{{ slot.name }}</template></FancyPanel>
<FancyPanel><template #[item]="{ item }">{{ item }}</template></FancyPanel>
<FancyPanel><template #[key]="{ source: key }">{{ key }}</template></FancyPanel>
```

Direct identifiers, renamed and nested bindings also have retained before
observations. Property keys, strings and outer-scope controls remain distinct.
Arrow-expression controls retain the current parser's complete diagnostics and
the independent provider's complete packets; they are not silently filtered.

A separate dynamic-binding producer fix must first establish a provider that
reuses retained argument reads and slot-binding ownership without requiring full
Croquis analysis for this syntax-only rule or reparsing every slot. The original
51-rule contract, legacy history, source corpus and instruction-count budgets
remain mandatory. This patch does not claim to repair that defect or the broader
parser and modifier-policy differences.

## Delivery

Post the paired concise decision on owned #8142 before opening the reviewed PR.
Keep upstream strictly read-only: no branch execution, vendoring or upstream
comments or changes. Publish a separate conventional PR after the emergency
third-party release; exact-head Actions, protected queue checks, actual signed
merge and released consumer verification are still required. Leave #8142 open.

Official semantics: [valid-v-slot](https://eslint.vuejs.org/rules/valid-v-slot.html).
