# Release reach accounting (2026-09-29)

This decision belongs to the [central level record](./2026-09-27-level-restructure.md)
and the [#6830 decision comment](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-5884874692).

The [v0.429.2 Real Project Matrix run](https://github.com/ubugeeei-prod/vize/actions/runs/36527068101/job/109272577203)
failed its production reach step for `vue2-elm/src/components/common/shoplist.vue`.
That pinned upstream SFC has a normal `<script>` and repeated `class` on one
template element. Since #7166, the SFC duplicate validator rejects this
template before DOM lowering. The normal-script product path retains that
`TEMPLATE_ERROR` in `Ok(SfcCompileResult.errors)` alongside its script-only
output, so no DOM backend ran and no selection counter exists. The reach gate
previously recognized only `Err(SfcError)` as an SFC error.

For an `Ok` result with a `TEMPLATE_ERROR` and no backend selection, count one
SFC error instead of an unrecorded DOM compile. Compare the complete result
with the forced legacy lane, including output bytes and diagnostics. Do not
synthesize a selection counter or credit L2 acceptance. An error-free compile
without a DOM selection still fails the gate. The focused committed fixture
uses a normal script and the same duplicate-class shape; no product emitter,
legacy output, or #7166 validation behavior changes in this correction.

The earlier [successful matrix](https://github.com/ubugeeei-prod/vize/actions/runs/36330400946/job/108651106540)
predated #7166. This correction requires fresh exact-head Actions evidence;
local formatting and source checks alone do not establish release readiness.
