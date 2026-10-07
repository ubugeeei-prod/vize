# Retained slot helper registration

## Concrete failures

Exact source `99d4839bde65afe26cc4afc89ec3f4a71a28894c` failed canonical
DOM comparison with 285 divergences and production reach with 233 divergences
in each DOM mode. The existing samplers retained 20 DOM windows and ten unique
reach paths in both modes. Every retained window differs by the adjacent
`createElementVNode`/`renderSlot` helper order; this does not classify the
unrecorded 265 DOM or 223 reach inputs per mode, or prove full module equality.

The complete failed Actions log is run `37612981066`, job `112764617628`.
The L2 step's displayed success comes from `continue-on-error`; its raw test
and the strict finalizer failed. Source `6a37d1d27006f75a6b414367513633d5c48cc2df`
retains that production code. Native successes on either head grant no
canonical or complete adoption credit.

## Source decision

Legacy `crates/vize_atelier_core/src/lane/element.rs` registers `RenderSlot`
while processing any authored `v-slot` props, before visiting its children.
Original n8n `FormInput.vue` has an earlier conditional component with a
named `#content` template without an outlet, followed by a branch containing
a wrapped outlet. That carrier registers the helper before the later VNode.
The original airi wrapped outlet has no such carrier.

The earlier repair incorrectly descended through every ordinary element to
choose the first helper. Restore the ordinary element's VNode marker, and
register the slot helper immediately for the retained slot-template role:
`tag == "template"` and an existing `BindingOp::SlotContent` binding.
There is no added parse, pass, source scan, or serialized product state.

## Independent evidence and obligations

`tests/_fixtures/differential/compiler/n8n-helper-props/slot-registration-controls.json`
retains 17 complete source/module/map/diagnostic packets from checksum-pinned
Vue 3.5.26, including both complete original SFCs and their licenses, eleven
carrier chronology controls, and all four unchanged authored controls.
Its exact SHA256 is
`9d3bf28ef2a02ab0dcf3647b1ec17372eadd83f44b235286ef0648211eea69a0`.
Stock ordering stays independent of Vize's legacy registration contract;
the recorded legacy ordering predictions are source inference.

The existing `davinci_dom_n8n_helper_props` target compares whole native and
forced-legacy modules for these 17 inputs in all three shipped option lanes.
Local checks authenticate frozen bytes and source/module/license hashes.
No local Vize Rust compilation or hosted candidate result is claimed here.

The existing four-target invocation can set `VIZE_N8N_CONTROL_PACKET_DIR`
inside its already uploaded custody directory. An optional test-only writer
then retains each eligible whole old/native comparison, complete legacy
code/map/error fields, native complete Debug/code, original SFC/template
bytes, exact options and SHA256 identity of the actual test executable.
Refused native results are tagged `Err` and receive no successful output
credit. Default corpus execution leaves the writer disabled; its existing
sampling limits, counters and strict gates remain unchanged. The separately
diagnosed original retains its full independent before/after custody packet.

Acceptance requires the unchanged entire canonical corpus and production
reach to report zero divergences, plus source, native, original custody,
runtime, and every explicitly feature-enabled authored target to pass on the
final head. Historical unrecorded failures remain unclassified even after
a successful fresh complete sweep. Queue admission remains blocked until
those terminal results exist. Paired issue decisions belong to Vize #8142;
the n8n upstream and its pinned original fixture bytes remain untouched.
