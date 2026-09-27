# Binding traversal instruction regression

For [#6868](https://github.com/ubugeeei-prod/vize/issues/6868), the protected
physical L3 candidate `60f67f86b9ae28b0466b65ec53bf84facb849a92` failed
[Check 36319177343](https://github.com/ubugeeei-prod/vize/actions/runs/36319177343).
All 100 probes repeated identically three times. Independent raw inspection
reconciled all 300 client-request totals, 36 zero termination dumps and 36
allocator setup receipts; fixture hashes, windows and the fixed guest
environment match the accepted reference. Three ceilings failed:

| Probe | Accepted ceiling | Candidate |
| --- | ---: | ---: |
| `s1_to_s2_lower_vfor_three_aliases` | 148047 | 150340 |
| `patina_jsx_markup_one_root` | 45385 | 45731 |
| `patina_s2_markup_one_root` | 539107 | 540157 |

The Patina increases are concentrated in `walk_items`: +334 of the JSX
increase of 346, and +811 of the template increase of 1050. Callback counts
are unchanged. The selected production optimization specializes owners with
no attached binding ops. JSX visits their static attributes directly; Vue
still resolves each authored attribute by exact span and restores consumed
or dropped surface items using the existing spelling policy. The nonempty
binding path retains its original authored-order merge and callback payloads.
This adds no allocation, pipeline stage or serialization.

New tests compare complete hook traces to the existing legacy projection:
static attribute values and spans, slot names, conditional keys, three
iteration aliases and carriers, mixed dynamic bindings, and a JSX tree.
The existing committed differential battery remains required. Local Rust
formatting, syntax parsing, assertion scanning and source growth checks pass;
typed tests, corpus and the unchanged Linux 100-by-three gate are pending.
No benchmark input, window, protocol or accepted ceiling changes.

The iteration probe has a distinct cause: all Vize/OXC costs and calls are
unchanged; its entire increase of 2293 is one extra main-thread memory-map
line read by libc `pthread_getattr_np` during stacker's first stack-bound
query. The exact mapped object is unknown because no map snapshot or binary
was uploaded. Stack protection and its cold work stay within the existing
window. A production optimization of lowering work must be measured before
claiming this separate failure resolved; later passing prefixes do not replace
the earliest candidate's protected validation.
