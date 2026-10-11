# Japanese Vue RFC guidance

Issues: [#8372](https://github.com/ubugeeei-prod/vize/issues/8372) and
[#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

Review the complete Japanese RFC detail page as authored documentation after
the actual #8661 merge, rather than treating a translated heading or generated
paragraph as a complete reader review. Explain how to enable each independent
flag, which processing surfaces use it, which diagnostics it adds, and which
parts remain unsupported. Keep the original implementation boundaries;
this change implements no compiler, checker or editor capability.

The complete sixteen fenced examples at source `24cd18c34626` remain byte-exact,
including invalid placements, the parenthesized fallback and slot cardinality
examples. Preserve all seven ordered destinations, all 109 distinct inline
syntax/field/type/value tokens and ten old heading bookmarks. The reviewed
translation marker and real regeneration path retain the complete authored
page. Ordinary Markdown paragraphs use one source line per paragraph; the
page remains below the existing 350-row source limit without a budget change.

Retain disabled defaults, independent opt-in, direct-plugin precedence,
unsupported aliases in native fields, error versus warning behavior, source
mapping limits, the shared parser and helper contract, native and WASM type
checking boundaries, unfinished Content Mapper and full editor acceptance,
in-tag comment placement, exact `<Self>` case and recursive termination, and
strict slot child cardinality and permissive-type limits. Upstream RFCs remain
read-only references; this review claims no upstream acceptance.

The existing original table-glyph checks remain byte-exact. In the four
original native table-cell controls, change only the Japanese reference's
label and complete explanation to their authored Japanese text; retain the
complete pattern literal, three native cells and all English and guide
controls. Existing implementation-boundary assertions retain their same
obligations with readable Japanese witnesses. A complete preservation and
regeneration test rejects the original generated page and qualifies the
reviewed page. Formatter normalization of original example syntax is rejected
by the whole-example digest; keep those original examples exact.

Obtain exact-source Actions, the original full Docs source/font/link/render
campaigns and an actual protected merge. Actual published-source custody and
desktop/mobile reading remain required before public acceptance. The complete
#8372 and #8365 reviews remain open; this is one finished authored slice.
