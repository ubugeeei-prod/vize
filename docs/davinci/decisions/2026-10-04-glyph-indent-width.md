# Preserve configured byte indentation (#7793)

Issue: [#7793](https://github.com/ubugeeei-prod/vize/issues/7793).
Configuration roadmap: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).

After verified v0.432.0 publication and explicit global thaw, fresh wt main
`caee1656927e1232f4aaabff2328246decb8bc58` contains the already delivered newline,
raw-body, Node bridge, quoteProps and JSX fixes. The existing tabWidth option is
an integer in public schema and a u8 in Glyph; the schema does not enforce Oxc's
0..=24 domain. Pinned Oxc IndentWidth accepts every value in that domain, and
Glyph's indent_string produces that many spaces. indent_bytes supports only
1/2/4/8 and falls back to two for other values, including the valid value three.
Template formatting and SFC block indentation consume that inconsistent helper.
This is source evidence; executed pre-fix output is not claimed.

Retain the default two-space branch and tab branch, project each valid width
onto a borrowed static space buffer, and retain the existing two-space fallback
outside 0..=24. The checked slice has no panic or allocation. Do not change
indent_string, schema validation, Oxc fallback, stages, public APIs, quotes,
newlines or source-map policies. Prior correct widths keep identical bytes.
The 349-line owner remains within its 350-line ceiling.

The original minimal input is 34 bytes (SHA7cf9fb3e); the complete independent
three-space reference is 39 bytes (SHA6c97fc04), including its final newline.
Add it as the twelfth legacy configured CLI case. Preserve all previous eleven
case objects, original captures, 300 API plans, native nine plans/25 calls,
source-law witnesses and native migration-credit boundaries.

Public Rust tests prove all valid widths, existing out-of-domain fallback,
tabs, Auto/LF/CRLF/CR layout, Vue 2/2.7/3, raw-body byte fidelity and complete
three-pass fixed points. Inputs/options remain immutable; changed compares the
whole source to the whole reference, including the already canonical zero-width
raw control. No source-map or broad option-combination acceptance follows.
Fresh exact-source Actions and all 104 unchanged instruction ceilings precede
root review and independent auto-squash. The actual protected candidate must
pass full suites and merge with a valid signature. Verify source/PR reporter
credit and actual final credit under the existing normalization policy; release
publication is separately verified. No large artifacts or cold Cargo build run
locally while disk space is constrained.

TODO: broad formatter profiles/option combinations/ranges/source maps remain
unfinished under #6098. Any future change to validation or widths outside the
shared established domain is a separate decision, not part of this fix.
