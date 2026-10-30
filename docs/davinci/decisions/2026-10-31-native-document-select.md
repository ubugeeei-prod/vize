# Bounded native Document select structure (#6835)

`NativeDocument::html_structure` now admits `select`, `option` and `optgroup`
through the existing original event-tape builder. It adds no parser, event/node
storage, decoded buffer, synthesized element, pipeline stage or product route.
`Profile::SELECT_CONTENT_MODEL` declares this host difference beside the existing
name, self-closing, implied-end and table policies. Component retains authored
nesting; Document retains HTML stack behavior.

The current [HTML in-body rules](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody)
handle select-family starts without a separate select insertion mode. In this
bounded subset, options close prior options, groups close prior groups/options,
and `hr` becomes a select child after implied closes. Ends retain their authored
frames, while implied ends retain no authored closing span. Text/comments and
static attributes use the existing original owner-bound cursors unchanged.

Nested select and input starts require browser recovery, so native construction
refuses with `IgnoredTag` and `ImpliedEnd`. Other children of select refuse as
`UnsupportedSelectContent`; they are not falsely classified as ignored by modern
HTML parsing. Table foster/insertion errors, malformed ends, foreign/raw/template
and Vue syntax keep their current typed refusals. The original outside-select
option/optgroup policy remains unchanged, including nested optgroups.

Thirteen complete original packets in `html-select.json` bind native and actual
Chromium DOMParser ancestry, attributes and direct child-node arrays. Ten admit
native construction; three characterize complete browser results while native
construction refuses the exact offending token. Native assertions also retain
every original start/end-tag callback and whole authored UTF-8 frame ranges,
including CRLF, literal NULL, decoded references and multibyte text boundaries.
All prior document fixtures remain byte-identical. Both profile behaviors and
cross-table/malformed refusals have separate native laws.

Local HeadlessChrome 152 compares all thirteen complete browser packets against
independently authored expectations. This is local characterization, not hosted
source or delivery credit. Exact-head hosted Rust/browser/source checks and
protected candidate validation remain required. No default compiler, linter,
formatter, typechecker or LSP route changes; #6835 and #6843 remain open for the
larger native document/petite-vue/product acceptance scope.
