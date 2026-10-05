# Reuse the existing whitespace plan under `v-pre`

This records the bounded provider correction for
[#7939](https://github.com/ubugeeei-prod/vize/issues/7939). The separate
[canonical divergence record](https://github.com/ubugeeei-prod/vize/blob/69f95bbf69a5742384d60993aeabb757f54bd0b3/docs/davinci/decisions/2026-10-05-canonical-v-pre-divergence.md)
owns the authenticated original 147-gitlink failure and expanded registration.
That source and failed execution remain historical; this correction does not
relabel them as accepted.

The original complete 228-byte SFC is committed unchanged in
`tests/_fixtures/differential/compiler/v-pre-whitespace/format-v-pre-content.vue.txt`.
Its SHA-256 is `40f6f59fd7ccdc7e13c33d23c3d643a0bd75758993844a9ae1d306712676dcb0`.
The upstream revision, Git blob and historical output limitations are recorded
beside it. No complete historical DOM oracle is invented from difference windows.

## Cause and correction

`lower_children` already computes one complete `plan_whitespace` for its child
list. Ordinary text lowering reads that plan. The `v-pre` text-run branch did
not receive it, so it collapsed the three removable newline gaps into spaces
instead of removing them. This accounts for the observed extra DOM text helper
and SSR spaces around the original two child spans.

Forward that same plan to `lower_v_pre_text_run`. After the existing contiguous literal-run scan, when a one-member authored text
run is planned `Drop`, retain the existing `condense.drop-whitespace` record
with its exact consumed bytes/span and no produced node, then advance once.
A run containing literal interpolation delimiters remains one complete text unit,
so its leading whitespace survives the normal condense. Other existing literal-run lowering stays intact. The correction adds no
pipeline stage, serialization, legacy dependency or separate whitespace policy.
`pre` still disables condensing, inline spaces and non-ASCII whitespace survive,
and literal braces/directive names remain frozen only in their original subtree.

## Bounded verification and delivery

The default L1-to-L2 tests pin the complete authored three-element folio and
all three removed-whitespace records, then verify source/span, side-table and
post-transform soundness. Independent controls retain inline/NBSP/em spaces,
`pre` newlines, literal text and an active interpolation outside `v-pre`.

Feature-enabled DOM tests require an accepted native stage capture and compare
complete code, preamble, map and empty diagnostic vectors. Feature-enabled SSR tests compare the entire serialized SFC
result, including code/CSS/map/bindings/errors/warnings/macros, and require the
selected provider to report exactly `s4`. Both use the whole original input
and six authored controls without trimming, output normalization or recapture.
The existing protected differential action explicitly executes both test binaries.

TODO: require exact-source Actions, genuine full DOM/SSR execution, unchanged
protected 104 instruction ceilings and actual signed merge. The canonical
coordinator must then execute the complete unchanged 147-gitlink DOM/SSR/Pug
matrix on the accepted composition. Source inspection provides no hosted runtime
credit. No comparator, error allowlist, original corpus, default routing,
fix-history denominator or instruction budget changes. #6880 and whole Davinci
migration remain unfinished; no performance or 10x claim is made.

The actual issue author is the existing primary maintainer, ubugeeei (public
GitHub ID 71201308), not a distinct external reporter. Meaningful source and PR
metadata carry the verified public noreply Co-authored-by trailer; the actual
squash footer is verified separately, without inventing a normalization cause.

The first publication omitted the generated DOM test/dev row from its staged
inventory although the local generator had updated it. The successor commits
that exact row alongside this receipt; both DOM and SSR inventory rows match
the actual test imports. No production or fixture bytes change. First-source
Actions remain historical and cannot qualify this corrected source.

Exact source `1c849a1a7c` Check `37260721463` genuinely fails before queue
admission. Existing full SSR parity exposes a leading-newline literal run:
dropping its first Surface whitespace before grouping loses the authored
leading space around `{{ variable }}`. Apply the existing Drop only after
the existing run scan establishes a single text member; keep complete fused
literal text unchanged. An authored L2 leading-newline control and both whole
DOM/SSR controls pin this counterexample. No second scan/stage is added.

The authored DOM controls also used non-void HTML self-closing syntax, which
the genuine native compiler reports as two ExtendPoint diagnostics. Make
these authored controls well-formed with explicit end tags; retain the exact
original 228-byte SFC and every whole comparison without filtering errors.
The storage inventory separately records the reviewed existing empty-after
provenance String occurrence, from 1/8 to 1/9. The failed raw jobs and all
other original inventories, diagnostics, input bytes and budgets stay intact.
Fresh source/protected runtime must qualify this meaningful successor.
