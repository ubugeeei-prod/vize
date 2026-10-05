# First newline in pre and textarea — #7889

The original [report](https://github.com/ubugeeei-prod/vize/issues/7889)
contains a complete script setup SFC. Preserve the exact bytes produced by its
printf command as `pre-textarea-first-newline/App.vue.txt` in the compiler
differential corpus; independently authored controls cannot replace that input.

Remove one initial LF, or one CRLF pair, from the first decoded text child of
an HTML `pre` or `textarea`. Keep its complete authored source location.
A preceding preserved comment, interpolation, element or space prevents removal.
A second newline remains. Decode entities once, including numeric LF/CR and
`&NewLine;`; an escaped entity spelling remains literal text.

The existing parser whitespace walk owns this normalization for the retained
compiler. Textarea RCDATA preserves the remaining whitespace instead of
condensing it into a space. Pre still normalizes CRLF in descendant text.
Native normalization recognizes both literal and encoded CRLF pairs while
preserving other entity spellings until the normal decoding boundary.
Nested pre owners apply their own first-newline rule; ordinary descendant
elements retain their initial whitespace.

The native L1 to L2 lowering applies the same rule in its existing text-run
consumption. Owner state is restored after every region, source spans retain
all authored bytes, and the existing branch, comment, compound and id laws
remain in force. No pipeline stage, serialization or selector fallback is added.

Direct native lowering tests cover the correction and authored ranges. The
whole original SFC and controls also run through a source-built CLI in DOM,
SSR, Vapor and Vapor-requested SSR modes. Chromium compares actual mounted
pre text, live textarea values, complete SSR HTML and hydration with the
repository-pinned Vue 3.6.0-rc.9 compiler/runtime. The report used rc.10;
the fixture metadata records that difference explicitly.

The reporter must mount and hydrate without warnings and retain its original
elements. Controls compare complete observations against the independent Vue
oracle, including any upstream hydration diagnostics for remaining leading
newlines. This correction does not expand SSR behavior beyond that oracle.

Public CLI behavior and direct native text-rule tests do not establish whole
native/default SFC or native Vapor SSR acceptance. The differential manifest
keeps that distinction explicit. Source Actions, peer review, protected queue
checks and the actual merge are required before recording delivery.
