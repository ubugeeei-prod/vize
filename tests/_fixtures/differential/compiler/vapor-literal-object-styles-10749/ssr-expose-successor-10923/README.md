# SSR script-setup expose successor

The original complete `base-sfc.json` and `head-sfc.json` bytes remain unchanged.
Only SSR indices 1 and 3 admit the single default-expose code edit introduced by
main PR #10923 (`9157b9fa83dbdc98100be084be288f21d0d1bc18`). Every other output
field, including bindings, map, diagnostics, CSS and macro artifacts, must match.
Fresh baseline and head SSR outputs must equal the entire archived output.

The archive was produced on an unmerged JSX branch. Its source/tree/run/artifact
custody is retained in `producer-capture.json` and pinned by `admission.json`;
this registration adds no JSX feature dependency or runtime/native parity credit.
The dedicated workflow supplies the actual current main source proof. Index 0
keeps its existing equality check; index 2 keeps its existing style-fold difference
check and is never admitted by this helper.
