# Vapor static attribute force modifiers (#7888)

The reported original App loses `input[value]` because retained Vapor lowering
forgets `.attr`, and static attributes after an object lose their authored merge
position whenever a force modifier prevents native admission.

Keep forced attribute keys in the runtime's existing `^key` representation and
consume that prefix before selecting `setAttr`. `.prop` retains `setDOMProp` and
wins when both force modifiers occur. The public IR layout stays unchanged.
Keep the retained static camel-key conversion byte-for-byte, including trailing
hyphens and punctuation; the force-key helper reuses the already computed camel
and prop flags. Replacing that conversion with the general utility would change
unrelated keys, so its original implementation remains in this slice.
For a DOM element that mixes an object with static `.attr`/`.prop` bindings,
retain every attribute in the existing ordered merged-prop operation; forced
keys use the runtime's `^key`/`.key` vocabulary. The selected native emitter
admits these static modifiers and emits the same operations from its owned
operands. No retained-level fallback or new pipeline stage is introduced.

The whole original App and a reactive control are retained in
`tests/_fixtures/differential/compiler/vapor-attribute-modifiers/`. The latter
checks both source orders, property versus attribute writes, updates and DOM
identity. Existing Actions Rust workers execute development/production and
requested inline/separate SFC rows against both Vize and the locked official
Vue Vapor compiler/runtime, with independently authored DOM expectations and
raw failed-process capture. Native tests require actual admission and matching
retained operation bytes. The pinned runtime currently uses Vue 3.6.0-rc.9;
the reporter used rc.10, so later runtime qualification remains separate.

Computed modifier keys, component force modifiers, `.camel` mixed with objects,
whole native SFC migration, browsers, SSR/hydration and performance are outside
this static reported slice. Their unsupported admission routes remain explicit.
Source Actions, merge queue instruction ceilings, actual merge and installed
release verification remain pending. No historical green or fixture authoring
is treated as a successful runtime execution.

Initial source `3192942496` Check `37589873592` rejected identical forced-attribute
and SVG class/style Clippy branches before Rust execution. The correction joins
the conditions with the same single `setAttr` body; fixtures and runtime
expectations stay unchanged. Shared security advisories remain a separate gate.

The existing PR now composes actual signed main
`8c7727613de6213e0a52cde96eeb295d01c9bef5` from its real
`22bdf5f276f9e28afeb18f189a7db473cde7d042` source head. Every owned product,
test, original fixture and runtime observer byte remains unchanged; the shared
SFC consumption inventory retains both sides' entries. The coordinated decision
record preserves the full slice clause alongside all incoming decisions. Earlier
source success does not qualify this composition: fresh exact-head Actions,
protected gates and actual merge remain required, with the rc.10 runtime and
computed/component boundaries above still unfinished.

After the protected preceding cohort actually merged, the same local source
chain incorporates signed main `3ed1cc90908c88016310b90a855500c94a156f1f`.
That incoming main changes no owned Vapor or SFC modifier source or fixture;
the complete owned byte census remains exact. The approved common 350-line
record is identical across the prepared backlog. This genuine main union still
requires fresh exact-head Actions and protected actual delivery; projected
merge analysis is never a source ancestor or execution witness.

The unchanged original module/runtime controls and source Actions passed on
`1f6926e10255b6de55bb4ca92df71b1ca6f6395a`, but protected candidate
`8f54a4b864850a482e1171fef72f28c9d0098c23` exceeded the existing Vapor
lowering ceiling (77,308 > 76,975) and interpolation ceiling
(5,340,840 > 5,340,048), alongside three Croquis ceilings. The candidate was
removed from the queue; these measured failures remain historical evidence.
The corrective source composes genuine signed main
`eed471b4424b922b878bc35cac765dc7274e5736` without compiler product conflicts
and retains the approved complete 350-line record and every original fixture.
Empty retained modifier lists now avoid static-key flag and prefix work; the
existing camel and force helper remains exact. Empty loop and slot stacks with
absent binding metadata return the same `None` through a small inline wrapper;
every other scope uses the unchanged resolution body. These paths use existing
metadata without changing outputs or raising ceilings. Fresh
Actions and protected measurements must establish the resulting counts before
this PR can be delivered; previous source success does not qualify the repair.

## Actual original source delivery

PR #8172 signed-merged as `1476c1921c0dd4026a64260dda26cbe89a9a1429` at 2026-10-07 16:27:51 UTC. Protected Check 37648954615 passed all four actual Rust workers, the required aggregate, canonical comparisons and original 104 instruction ceilings. The exact 192-byte original App and 434-byte Reactive source run eight whole compiled modules across both production and inline modes. The locked rc.9 oracle agrees on the reported value/title attributes, later static id precedence and authored reverse-order object precedence; full ordered attribute/property vectors, updates, node identity, empty diagnostics and unmount pass. These attribute observations do not claim recursive HTML or decoded-map accuracy.

The complete original fenced source matches its retained issue body, manifest byte count and SHA at the actual signed merge. Combined original-source custody SHA-256 is `72b65a0913013914e274e1ce8abf5e6e980815895f0929962f872773e3d3a2c9`. The required law also passes later exact 2e96, whose own qualification remains distinct. The original source defect is [closed as completed](https://github.com/ubugeeei-prod/vize/issues/7888#issuecomment-6046652952).

Earlier OPEN statements retain historical source/queue scope. Reported rc.10/Node26/installed public replay, browser/performance/release delivery and broader computed/component/filename-case/native custom-directive/SSR/hydration/native-only graduation remain unfinished; they are not new gates for the whole original source Expected contract and receive no acceptance from this closure.
