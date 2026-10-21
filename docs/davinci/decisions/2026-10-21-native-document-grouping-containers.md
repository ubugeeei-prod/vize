# Native Document grouping body containers (#6835)

The original Document-profile structure provider already retains one completed
lexer event tape and admits a strict explicit no-quirks `html` / `head` / `body`
envelope. This slice admits six further HTML body containers in that existing
insertion loop: `address`, `blockquote`, `figure`, `figcaption`, `hgroup` and
`search`.

These names use the same source-owned element, attribute and direct-child views
as the previously admitted `div`, `span` and semantic containers. ASCII name
folding, authored opening and closing spans, ignored nonvoid self-closing
slashes, duplicate-attribute selection and once-decoded values are unchanged.
No event, element, node or decoded-value storage is added, and no source is
lexed or decoded a second time.

HTML's in-body rules can close an existing `p` before several of these start
tags and generate implied ends while processing container closes. This bounded
provider still refuses `p` and every implied-end family. It admits only an
explicit close matching the current top of the open-element stack, so it does
not claim the unfinished implied-end policy. Table modes, foreign content,
raw-text and formatting elements, templates, Vue syntax and implicit envelopes
remain explicit refusals.

Eight complete packets are shared by native Rust and actual Chromium
`DOMParser`: one mixed-case, ignored-slash and duplicate-attribute source per
new name, one all-sibling source and one fully nested source. The packets pin
whole element ancestry, effective attributes and direct text/comment/element
children. Native tests additionally pin every authored opening/closing frame
and retain wrong-mode, unmatched-close, prefix, Vue, implied-end, table,
foreign, raw and formatting refusals.

This is independent of the SFC container language projection in #10131: it
consumes a complete Document root rather than a Vue SFC descriptor. It neither
switches a product route nor completes general HTML or petite-vue semantics.
#6835 and #6843 remain open. Exact-head Actions, protected unchanged
instruction ceilings and the actual signed merge decide acceptance.
