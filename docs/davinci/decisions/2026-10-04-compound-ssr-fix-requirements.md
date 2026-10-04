# Compound SSR fix requirements (2026-10-04)

For [#6880](https://github.com/ubugeeei-prod/vize/issues/6880), classify the
requirements of a compound fix separately before granting whole-fix coverage.
The pinned population remains 626 candidate subjects, 30 fixture links and 596
subjects awaiting semantic review; the original issue's 609 fixes remain its
historical population. This audit changes neither denominator nor runtime code.

Fix `63d11c41a88db4cead31dbbb512e3a47c8ac29e6` ([#983](https://github.com/ubugeeei-prod/vize/pull/983))
contains two behavior changes from [#962](https://github.com/ubugeeei-prod/vize/issues/962):

| Requirement                                                       | Original authority                                                                                                                                                                         | Fixture status                                                                                                                                              |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Textarea model renders its bound value as escaped text            | `test_ssr_v_model_textarea_renders_bound_value`, exact `<textarea v-model="x"></textarea>` through default `compile_ssr`, empty diagnostics and `_ssrInterpolate(_ctx.x)`                  | Existing `textarea-model` preserves that input and default options, with the complete code/preamble/map/diagnostics reference and source-qualified capture. |
| Direct input processing uses the dynamic model helper for `:type` | The same fix adds `get_dynamic_bind_exp` and the `_ssrRenderDynamicModel(type, model, null)` branch in `process_v_model_on_element`; its issue/PR describes the `inherit_attrs=false` path | No original complete dynamic-input test/options or full-result capture was added by that fix. The textarea fixture does not exercise this requirement.      |

The existing TSV state `fixture-linked;target-and-semantic-review-pending`
remains accurate. Its textarea link supplies one requirement witness and grants
no whole-fix or native credit. The merged-attrs fallback and select-model work
explicitly deferred by #983 are separate requirements of #962 and later fixes.

## Next witness

Find a source-authored complete direct-input witness and its effective options
before registering it as original history. The original issue's dynamic-input
sketch omits the model expression, and the fix adds no such test. A new bounded
control such as `<div><input :type="t" v-model="x"></div>` must carry its own
authorship and source proof; it cannot be labelled an original test from #983.
`inherit_attrs` is an internal insertion-path argument, not a public option to
invent in the default compiler contract. Prove which original branch executes.

Freeze complete code, preamble, map and diagnostics with actual source/binary,
options, status, stderr and repeated capture identity. Preserve the existing
five SSR rows and their captured packet byte-for-byte. Any shared registration
extension needs a separately authenticated cohort and actual default execution;
native remains unavailable until a genuine complete product path supplies the
same output contract. Text/checkbox/radio rendering and hydration evidence must
also remain distinct from byte-output capture. #6880 stays open.
