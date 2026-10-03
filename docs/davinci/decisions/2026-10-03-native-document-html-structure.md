# Native explicit-envelope HTML element structure

This is a bounded Stage 1 continuation of [#6835](https://github.com/ubugeeei-prod/vize/issues/6835)
and [#6843](https://github.com/ubugeeei-prod/vize/issues/6843), after the
[original Document lexical owner](./2026-10-03-native-document-lexical-owner.md).
Both roadmap issues remain open. The issue comment and central markup paragraph
record this same decision; neither product nor general Document completion is
claimed.

## Actual native producer

`NativeDocument::html_structure` constructs `DocumentHtmlStructure` directly
from the original owner's sole retained event tape. It never reparses the source,
accepts external token vectors, wraps a Component surface or uses Armature.
The lexical owner privately retains its original allocator; both element and
open-stack storage use that exact arena. Immutable element, parent and child
views borrow the sealed structure and genuine original owner. Names, openings
and full closing frames retain checked original UTF-8 source coordinates.

The existing L1 event-to-structure construction step stays in L1; no serialized
transport or added level stage is introduced. Original Component and legacy
construction paths retain their bytes and policies. No Cargo manifest, lockfile
or normal legacy dependency changes are needed.

## Implemented structural meaning

Admission requires an authored, explicit no-quirks envelope:
`<!DOCTYPE html><html><head>...</head><body>...</body></html>`.
Names match without ASCII case. Every nonvoid element needs its matching
explicit end, even when its original opening ends in `/>`. A slash cannot close
a nonvoid HTML element: a subsequent `span` inside `<DIV/>... </DIV>` has the
actual native `div` parent. True void elements are inserted without pushing an
open frame. Direct-child adjacency and original start-tag order are retained.

Head elements are restricted to `base`, `link` and `meta`. Body elements are
restricted to `div`, `span`, `br`, `hr`, `img` and `input`. Static attributes retain
the whole original opening slice but confer no DOM attribute-value semantics.
Text and comments are not nodes in this element-only ancestry projection.

The admission modes enforce explicit `html`, `head`, then `body` ordering. Only
HTML TAB/LF/FF/CR/SPACE text is admitted outside the body interval. Entity events
outside body refuse because the original lexical recorder did not retain their
decoded whitespace classification. Closing frames require original immediate
`</`, a genuine `>` and only HTML whitespace between name and terminator; legacy
`</ div>` recovery cannot become browser structure. Comment observations must
have complete original `<!--...-->` framing without nested/recovered markers.

## Refusals and remaining work

The provider first requires original normal lexical completion. Lexical errors,
pending lexical states and declaration recovery retain their original typed
refusals. Incomplete or mismatched structural frames cannot mint a structure.
Missing/repeated/out-of-order envelope members and nonspace text outside body
refuse rather than inventing browser-created wrappers.

Tables and implied-end families report their unfinished policy. Foreign content,
raw-text elements, formatting/adoption rules, templates and other elements remain
unsupported. Interpolation and directive events refuse: Vue lexer interpolation
can hide literal HTML tags, and dynamic directive arguments can consume a `>`
which browser HTML tokenization treats structurally. Static `v-scope` is likewise
outside this provider's admission, even though the lexical owner retains it.

All four general `DocumentTreePolicy` entries stay unfinished. This restricted
structure supplies no general HTML tree completion, decoded text/attributes,
DOM root selection, petite-vue scope/effect L2 admission, Descriptor, File or
native product completion. The original pinned petite-vue SVG document passes
lexical completion but explicitly refuses this bounded structure. #6843 and
the lint/LSP fix-history and default-path gates remain open.

Future work is genuine browser insertion modes, implicit envelopes/end tags,
table foster/wrapper construction, namespaces/raw text, DOM value semantics and
petite-vue dialect/provider admission. No policy constants alone close those
TODOs.

## Independent laws and acceptance

The shared eight-row original-source census covers casefolded names, actual
nonvoid slash ancestry, true void insertion, sibling adjacency, generic nesting,
quoted markup/unquoted slash attribute bytes, entities/comments and head metadata.
Native Rust laws compare the produced element names/parents with that census.
The existing Chromium browser suite independently computes DOMParser element
names, HTML namespaces and actual parent start order from the exact same sources.
Browser rules are grounded in the [HTML tree construction standard](https://html.spec.whatwg.org/multipage/parsing.html#tree-construction).

Additional Rust laws cover exact original closing spans, source/arena custody
after owner moves, insertion-mode and Unicode/entity refusals, Vue syntax,
unfinished families, malformed/end recovery, pending structure and the original
pinned upstream refusal. Private construction and non-Clone compile failures
protect the retained structure capability. Independent source review found no
accepted-source ancestry counterexample in this admitted subset.

Initial local dependency setup failed with ENOSPC. A started focused cache-backed
run completed eight of nine laws before the team-wide local-build stop; the ninth
incorrectly expected a Vue refusal where original lexical recovery takes
precedence. Its assertion is corrected to the actual typed lexical refusal.
Local browser startup failed on missing package installation. Neither failure
is validation credit, and no further local builds/installations are run. Pure
format/source/storage checks and exact-head hosted Rust/browser Actions,
protected full/instruction candidate acceptance and actual merge decide delivery.
