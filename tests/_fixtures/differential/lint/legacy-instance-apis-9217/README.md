# Legacy instance API diagnostic contracts (#9217)

[Issue #9217](https://github.com/ubugeeei-prod/vize/issues/9217) came from the
Real Project Matrix at `d2506ca6b5e31252973f05981829c0367cacf26b` and remains
open as the receiver-accuracy acceptance tracker.

Parent PR #9222 fixes exact deprecated script-property spans and JSX-bearing
JavaScript SFC parsing. Missing template `$listeners` diagnostics are owned by
[Draft child #9251](https://github.com/ubugeeei-prod/vize/pull/9251), which is not
accepted until retained AST, authored coordinates and lexical ownership are
qualified. The existing broad
script warning policy is preserved, including unknown receivers, ordinary
objects, bare script references, registered components and custom factories.
It does not restrict diagnostics to the pinned provider's receiver/factory
heuristics. TypeScript-wrapped and optional calls retain their warnings.
`$emit` and ordinary external-emitter `on`/`off`/`once` remain valid.

`independent.json` retains 70 authored SFCs, source SHA256s and complete unchanged
ESLint 10.4.1 / eslint-plugin-vue 10.9.2 / vue-eslint-parser 10.4.1 /
@typescript-eslint/parser 8.65.0 packets. Both rule implementations and the shared
utility implementation are pinned by hash. `native.json` contains separate
complete native expectations; `provider-gaps.json` explicitly records the 35
authored cases where the historical native script policy is broader than the
provider. A provider omission is not automatically a native false positive.
No budgets or reported ranges are adjusted to make the tools agree.

All 70 controls remain categorized in `template-ownership.json`: 53 belong to
the script parent and all 17 template controls belong to child #9251. The full
`native.json` expectations are unchanged in that Draft. The parent consumes the explicit `script-native.json` projection; its
template-owned entries expect no new script diagnostic, without deleting or
rewriting any full template expectation. The Node replay verifies packet hashes
and both ownership inventories. The child now uses the complete native inventory
and actual CLI, with additional boundary packets; exact-source CI and source
review remain required before it leaves Draft.

The public script regression is
`crates/vize_patina/tests/legacy_instance_apis.rs`; the provider/provenance/runtime
replay is `tests/tooling/patina-legacy-instance-api-reference.test.mjs`.
Native acceptance of all original projects remains the Matrix's responsibility,
separately from the authored native contract and independent provider replay.

`originals/manifest.json` identifies 18 immutable project revisions and 147
complete affected sources. `originals/sources.json.gz` preserves their bytes,
URLs and hashes. Every `*-lint-divergence.json.gz` is the whole original Matrix
report, including unrelated rules and failures; `originals/independent.json.gz`
retains the complete two-rule provider packet for every captured source.
Original source licenses remain those of the linked repositories; these are
regression evidence rather than production dependencies.

[The receiver provenance audit](./provenance.md) explains all 239 original
false-positive labels without conflating provider silence and product truth.
`provenance.json.gz` retains all row classifications, full primary source
packets and hashes. Known Vue instances and unknown receivers keep their
existing warnings. One lexically proven tiny-emitter call and eight unresolved
receiver rows remain visible as the acceptance frontier; this PR does not claim
to resolve them. The scoped-slots rule remains a separate scope in
`expanded-audit.json`.

The pinned Vue 3 runtime packet in `runtime.json` and its executable control
verify public `$root`/`$parent` identity, component Options returned by a custom
factory, missing removed APIs after TypeScript erasure, silent optional calls,
and the still-working `$emit`. A real foreign emitter control still works,
which documents the remaining distinction the broad script rule cannot prove.

The child retains the intended scriptless/dual-script, loop, slot, escaped-name,
dynamic-argument and handler behavior. Its canonical replacement consumes the
original retained grammar owners and authored entity ranges, with bounded
lexical evidence for modern slots, collection ordering and If conditions. Legacy
slot attributes are scopes only with an explicit Vue 2 dialect; default Vue 3
keeps them as unevaluated plain attributes. A separate review-correction overlay
retains the six original historical legacy expectations and adds actual Vue 3
compiler/runtime proof and current native/CLI expectations without rewriting them.
`template-boundaries.json` retains 42 historical whole packets, originally labelled
27 admitted, five provider blind spots and ten refusals. The current Vue 3
overlay corrects these derived labels to 23 admitted, 11 product-accuracy cases
and eight refusals. `cli70.json` and
`template-cli.json` preserve all 112 source/config/status/formatter packets.
The remaining refusal frontier and qualification requirements are recorded in
`template-authority-plan.md`; no synthetic consumer reparsing is accepted.

Two separate append cohorts preserve positive component values without changing
those frozen 112 inputs. `template-component-bindings.json` keeps 30 complete
provider/compiler packets (15 listener controls and 15 scoped-slots research
controls); `template-component-declaration-bindings.json` keeps 40 more, including
value declarations, erased types, lexical negatives and full compiler/runtime
contrasts for bare identifiers versus explicit `this` references. Their native
and CLI projections retain 15 and 20 listener observations. The latter explicitly
marks `template-this` as observation-only, not qualified acceptance. The four CLI
tests therefore retain all 147 whole observations; their native execution is
still pending exact-source Actions. Scoped-slots behavior is not expanded here.

Five original Element SFCs require JSX script parsing; explicit TypeScript keeps
its angle-assertion grammar while TSX opts into JSX.

Replay with the pinned benchmark dependencies installed:

```sh
node --test tests/tooling/patina-legacy-instance-api-reference.test.mjs
cargo test -p vize_patina --test legacy_instance_apis
```

For separately installed dependencies, set `VIZE_LEGACY_API_PROVIDER_MANIFEST`,
`VIZE_LEGACY_API_SCRIPT_PARSER_MANIFEST` and `VIZE_LEGACY_API_VUE_MANIFEST` to their
package manifests. Versions and implementation hashes are verified; providers
are never silently substituted.

`template-normal-setup-refusals.json` adds a separate bounded completeness
frontier: three original normal-setup research cases retain their historical
whole packets while a derived overlay refuses unsupported absence claims.
Four counterlaws preserve complete script warnings. The existing 147 CLI cases
still execute; none of these refusal records earns provider-parity credit.
