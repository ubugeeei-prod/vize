# Authored letter case in documentation

Issues: [#8372](https://github.com/ubugeeei-prod/vize/issues/8372) and
[#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

The maintainer requests natural letter case rather than forced uppercase in
documentation. Remove the six active uppercase declarations from the landing
page's feature index, actions and mobile description, and from blog category
and metadata labels. Preserve every other layout, spacing, colour, font and
interaction declaration, along with authored wording and acronyms.

Actual Chrome exposes eight additional uppercase rules from the default
framework theme: sidebar and contents titles, mobile footer labels, search
scope labels, callout titles and three levels of API section headings. Set
only their text transformation to `none` with finite selectors in the Vize
theme. Keep the readable shared rule in `letter-case.css`, imported by the
existing stylesheet: source Actions rejected growing the over-limit
`style.css` from 1088 to 1095 lines. Preserve that failure and the unchanged
growth gate; the corrected large file shrinks. Preserve DOM tag-name conversion
in the test support; it is a separate HTML contract and does not change displayed letter case.

Two literal French and Portuguese product headings read `ESBUILD`; use the
existing English and Japanese brand spelling `esbuild`. Their native heading
anchor remains `esbuild`, and complete examples, destinations and policy remain
exact. Acronyms such as HTML, SFC, JSX, TSX and CI/CD remain authored as written.

The original public cascade is observed at source
`d55b8238f89bab4418a1a961bd8db1c84911cb42`. Ten EN/JA/FR landing and English
blog desktop/mobile views demonstrate that removing only the six declarations
preserves authored text, fonts and layout properties and exposes natural case.
Keep their original and CSSOM comparison receipts and PNGs; this controlled
observation is not built or deployed candidate acceptance. Extend physical
reading verification to the default navigation, contents and footer labels.

Obtain fresh exact-source Actions, the existing full Docs
render/source/font/link campaigns and protected actual merge. Publication and
public reading acceptance require the actual deployed source and custody;
historical screenshots or queue admission do not establish delivery. No new
implementation-mirroring CSS test substitutes for these existing browser gates.
Both whole-site documentation reviews retain their unfinished acceptance.
