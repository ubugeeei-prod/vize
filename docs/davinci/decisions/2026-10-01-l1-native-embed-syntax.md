# L1 retained native JS/TS syntax provider

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836), following
the [typed-embed design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929).
This is a genuine dependent consumer of the
[checked embedded source API](./2026-10-01-l1-embed-source.md), not a product
switch or a completed container/dialect integration.

`embed::syntax::parse_once` consumes caller-selected `Grammar` and validated
`EmbedSource`. Expr and Program use the pinned OXC parser once in L0's shared
arena, retaining the actual AST, comments and full diagnostic metadata.
Program parses the original source directly. Expr uses a private `(` and
newline wrapper because OXC's expression entry point discards its comment
list. Removing only the generated outer parentheses preserves authored
parentheses; an escaping wrapper or multiple statement shape cannot become a
successful expression. No wrapper Program or wrapper text is exposed for Expr.

AST spans use exact checked wrapper correction and the source map before they
become authored coordinates. Comments expose corrected spans and decoded bytes.
Parser diagnostics remain available even when a local syntax hole hides the
recovery AST; message, severity, code, help, note, URL and label metadata are
retained rather than replaced with a generic parse failure.

Diagnostic labels are a covering view, separately from exact edit selection.
A label intersecting generated delimiters is intersected with real source;
wrapper-only labels anchor to the corresponding source start/end point.
Labels splitting a UTF-8 scalar cover that scalar, then entity-interior
boundaries conservatively cover the complete authored reference. Exact AST
locations still reject wrapper bytes, non-UTF-8 boundaries and partial
entities. A diagnostic highlight must never select rewrite bytes.

This initial API is explicitly bounded: the complete parser input admits at
most 31 conservative word/punctuation units, including generated delimiters,
strings and comments. ASCII identifier/numeric runs count once; every other
non-whitespace byte counts once. Whitespace ends a word run and non-ASCII UTF-8
bytes overcount. Larger valid snippets return `TokenBudget`, preserving source.
This avoids pretending that L0's expression nesting guard bounds unbraced
statement recursion: OXC also recurses on nested if/loops, labels, new, class
heritage, assignments/arrows and TS syntax without deep brackets. A complete,
less conservative recursion admission provider remains unfinished.

Other shapes, Flow and input rejected by L0's existing OXC safety guard return
distinct typed local holes. Both admission checks see actual wrapped input.
Unadmitted inputs are not parsed and do not manufacture parser diagnostics.
Accepted malformed syntax retains OXC's real diagnostics and comments. OXC
syntax acceptance is not semantic validation or identifier resolution.

Normal dependencies are pinned workspace OXC libraries and L0. There is no
normal/build dependency on a legacy crate, extra pipeline stage, inter-level
serialization, new node allocator or root markup/product route change. The
source API and this genuine child use verified native GitHub Stack #7349;
parent-base links or prose alone are insufficient. Root review authorized
publication after the preceding configuration Stack actually merged. Protected
queue and actual merge proof are required before reporting integration as merged.

TODO:

- [HandlerBody and SlotParams](./2026-10-01-l1-native-embed-shapes.md) extend
  this provider with corrected wrapper laws; ForHead and FilterChain still
  require real dialect/language pieces.
- Replace conservative whole-input admission only after complete JS/TS parser
  recursion is bounded, including statement/type/operator nesting and escapes.
- Admit JSX/TSX explicitly rather than infer them from current Js/Ts enums.
- Resolve language once per file, diagnose script/setup mismatch, and attach
  artifacts to existing L0 node identities in the actual markup/container path.
- Use concrete dialect shape decomposition plus lexical diagnostics; malformed
  recovered directive arguments do not alone admit a grammar.
- Connect actual L2/L4 consumers without reparsing. Preserve held v-pre and
  product fix-history gates before replacing production routes.

Thirteen consumer/admission laws cover actual JS/TS programs and ASTs, expression comments
and authored parentheses, decoded entity input, exact edit projection,
retained failure diagnostics, UTF-8/wrapper diagnostic covering, unsupported
shape and safety holes, language admission, checked parser lengths and the
conservative recursion budget. Four admission laws pass under a lightweight
rustc harness using the actual source/tests with a minimal formatter shell.
Exact-head Actions subsequently passed all twenty-five source/OXC/admission
laws, crate compile, Clippy, wasm32-wasip2 default and no-default-feature
libraries, full Check and strict100. Fresh-main replay preserves the merged
directive provider's dev-only oracle and removes L1's now-zero skeleton baseline;
new exact-head Actions must pass before protected queue submission. Formatting
and offline locked Cargo metadata pass locally without Cargo builds. Source-derived
Croquis inventory is regenerated without classifier changes. Instruction100
ceilings are unchanged; protected queue validation remains required.

The initial published [full Check](https://github.com/ubugeeei-prod/vize/actions/runs/36852534851)
failed the exact per-file storage preflight. The reviewed child rows record its
temporary L0 wrapper String (one import/one bound use) and four test-owned
comment/diagnostic collection sites (one import/four bound uses, analysis
category). Parent map/source rows are inherited through the actual dependency.
No storage types, policy classifier, gates or instruction ceilings change.
