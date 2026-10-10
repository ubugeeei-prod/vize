# Slot parameter entity output history (#8142)

This additive corpus records the intentional legacy DOM output improvement for
`<Panel v-slot="&#32;{ item }">{{ item }}</Panel>` and a literal-space control.
The complete original input bytes, parent whole codegen/error packets and producer
whole packets are retained separately. Both Function/no-prefix and Module/prefix
options run on selected and legacy DOM: eight packets per source state.

Parent source: `943702629e503d8b10b4ff9a953c597a268482c0`.
Observed producer: `a80191dfb8dcc7575ea77477513bdf25f7e2a506`.
The legacy entity output replaces physical `&#32;` with decoded whitespace;
code/preamble/map/errors otherwise match. All four literal-control packets and
both selected entity packets remain byte-identical. This records a deliberate
legacy fix, not universal compiler byte preservation.

`runtime.expected.json` is independently authored from a real Vue custom renderer
and a Panel default slot supplying a reactive `item`: `entity-slot-value`, then
`updated-slot-value`. Parent entity output fails with SyntaxError in all four
conditions; producer legacy output renders/updates in both conditions, while
producer selected output still fails. Full observed trees, host calls, exceptions,
evaluation sources, native/compiler hashes, Rust/Cargo/Node and pinned Vue 3.5.26
provider custody are retained in the bounded source comparison:

`/private/tmp/vize-slot-entity-probe-20261010/comparison-audit.json`
SHA256 `61a9071e7fcbf834e5f9338d904dd088f78c4ce83eb489a5d91503cb27f80dd6`.

Croquis declares `item` at 18 on both source states; the authored entity input's
physical declaration is 22. The four-byte decoded-to-physical gap is preexisting
and unfinished. The literal-space declaration is correctly 18. The test records
that existing gap explicitly; it does not assert mapping correctness.

The Rust integration test writes complete actual code/preamble/map/error
packets and Croquis/AST observations to a workspace-root differential directory
before asserting whole goldens. It contains one test to serialize observations.
No original 96 parameter packets, 245 consumers, or existing historical fixtures
are modified. The old parent goldens remain immutable history.

The Rust test now also invokes the existing checksum-pinned full Vue loader.
It executes all eight preserved historical modules and all eight actual current
modules, retaining complete evaluation bytes, exceptions, host calls, trees,
Node/native hashes and actual process status before asserting the sixteen
authored runtime laws. Historical modules are explicitly historical packets;
this consumer execution is not a fresh baseline Rust build. Selected entity
SyntaxErrors remain explicit failures of the product, with no success credit.

The paired issue/canonical decision records this bounded change. Fresh actual
source Actions remain required. Scratch before/after proof and local connected
runtime checks do not replace an exact-head CI/native/canonical/merge/release
gate, or close all #8142 work.
