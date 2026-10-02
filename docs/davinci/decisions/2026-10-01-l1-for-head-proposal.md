# Native ForHead: reviewed first bounded slice

The selected first slice for
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836) is a dense alias block,
parsed once as SlotParams, plus one collection Expr parse. The concrete source
and ownership contract is recorded in
[the dense provider decision](./2026-10-01-l1-native-dense-for-head.md).

The original [embed design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929)
requires one L1 parse per language piece, authoritative source and OXC syntax.
The existing strict repository splitter selects the first whitespace-delimited
`in`/`of` before JS parsing. The original-expression and slot-binding handoffs
retain actual arena roots, comments, complete owned diagnostics and checked
source maps. Their consuming APIs provide the genuine prerequisites for this
composite provider.

The earlier proposal parsed individual comma-separated positions after a
bounded text scan. That approach was not selected: one actual formal-parameter
AST determines dense positions, so commas in regexes, templates, comments and
TS generic types require no partition heuristic or additional parse. One to
three dense formals are admitted. Empty/sparse, extra and rest positions keep
the whole checked head and typed refusal rather than an invented binding.

Full sparse grammar, contextual coverage and unrestricted JS/TS admission remain
separate unfinished work. A native ForOp consumer must use these retained roots
and sources without another language parse. No legacy-backed provider or product
replacement is authorized by this source slice.
