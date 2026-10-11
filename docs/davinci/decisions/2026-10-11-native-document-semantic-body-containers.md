# Native Document semantic body containers (#6835)

The original native Document HTML structure admitted only `div` and `span` as
nonvoid body containers. Ordinary semantic HTML such as an explicit `main` or
`section` returned `UnsupportedElement`, preventing their original parents,
effective attributes and direct child-node views from being produced.

This slice admits `article`, `aside`, `footer`, `header`, `main`, `nav` and
`section` in the existing strict explicit no-quirks body mode. Native ASCII
name folding, opening/closing admission and sealed child-node parents use the
same original event tape and existing element adjacency. These elements push
the existing open-element stack, require explicit matching ends and ignore a
self-closing slash as HTML nonvoid elements. No new storage, decoding, lexer,
pipeline stage or level serialization is introduced.

The [HTML in-body rules](https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody)
insert these names after closing a `p` in button scope and generate implied
ends when processing their close tags. The bounded provider already refuses
`p` and every implied-end family; accepting only exact top-of-stack closes
therefore preserves the supported subset without modeling either operation.
General HTML insertion policies remain unfinished.

Ten complete independently authored packets are shared by native Rust and
actual Chromium. Seven sources cover each mixed-case name with an ignored
slash, duplicate/Boolean attributes, harmless `xmlns`, Unicode, reference and
literal CRLF/NUL boundaries, children and a following sibling. Three whole
sources cover all seven siblings, all seven nested parents and unchanged old
tags. Each packet includes every element parent, effective attribute and direct
body-subtree child. Exact authored opening/closing and ignored-slash frames are independently
recorded for whole native comparison. Both native laws fail against the unchanged parent source
and pass after the implementation; the same browser packets pass.

Forty typed refusal controls retain wrong-head mode, omitted or misnested
closes, name prefixes, Vue syntax, implied-`p` and table policies, foreign
namespace, raw and formatting element boundaries. Existing envelope limits,
source inventories and immutable 104 instruction ceilings remain unchanged.

[#6835](https://github.com/ubugeeei-prod/vize/issues/6835) and
[#6843](https://github.com/ubugeeei-prod/vize/issues/6843) remain open. This is a
bounded native L1 provider slice: general Document, petite-vue meaning, all Vue
dialects, downstream product routes and released acceptance are unfinished.
Exact-head Actions, actual protected merge and release evidence are required.
