# Include natural casing in the compiled theme

This decision pairs with #8372 and #8365 and completes the packing correction
for #8674. The authored words, acronyms, code, layout and eight existing
`letter-case.css` overrides remain unchanged.

## Actual public counterexample

Source `94cbc15a8c7fd91aa9071912d2fc151bf9d0dd61` passed its complete 119-route
Docs campaign. Its actual merge `7674532c230e10115c3c21ca050bfd06725f4a5d`
also passed official Docs run `38158802004`. Neither proved that the theme
bundle applied the intended casing.

The authenticated publication of `a9b23c513d42cf2ff442cc2c2d2d8f9edb265f13`
had manifest SHA-256
`6229860ca3694d90909deff317f7024cd14258dd83cbee872886f343ecdbac08`.
Its native `/blog/` observation found `Start`, `Build an app` and
`On this page` computed as `text-transform: uppercase`. The physical
42,267-byte theme stylesheet SHA-256 was
`7fec68910fec36e0629977fa4f2ab686ea93b1e3551a6c48856396f3b19ef6af`.
It contained `@import "./letter-case.css";` at byte offset 1547 after six
qualified rules and contained no `text-transform` declaration. The core
stylesheet retained its eight default uppercase declarations. A separate
read-only peer confirmed these exact bytes and native observations.

The theme builder concatenates explicitly listed CSS files as raw strings.
`letter-case.css` was missing from that list; its late relative import was
therefore unprocessed and invalid in the emitted stylesheet. This is a real
packing failure, independently of grouped CSSOM selector serialization.

## Correction and retained gates

Include the unchanged sidecar last in the existing raw CSS array, after all
other custom theme rules, and remove the invalid import from `style.css`.
No extra CSS transform, animation, capture stage or browser campaign is added.

The existing native theme-readability gate now examines the loaded CSSOM for
the same six owned selectors and eight theme selectors. It splits grouped
selectors, requires all eight compiled overrides, and checks the computed
transform of every actual matching element or pseudo-element. Absent API
elements remain absent; no DOM or CSS is injected to manufacture witnesses.
Unreadable cross-origin font styles do not excuse missing owned rules.
The original native annotation, focus, contrast, source, font, link, capture,
scroll checkpoint, route, timeout and concurrency authorities remain intact.
The existing isolated-import fixtures retain their zero-browser and
zero-native-transform assertions while copying the new assertion dependency.

Root and the independent packing peer cleared the concrete correction.
The 11 existing navigation and locale-preflight tests pass, including both
whole partial receipts and isolated zero-native imports. Fresh exact-head
Actions, the complete original Docs campaign, protected actual merge and
stable public CSSOM/computed casing must still qualify this repair. The old successful builds and authentic public RED
remain separate evidence; source review or queue admission grants no public
completion. Broader Japanese and More-guide issues remain open.

## Fresh native counterexample and provider dependency

The exact assertion helper from `36259d861ea23c20249c45b68102f8f4603ba17b`
(SHA-256 `50afef71699666db5d18e728ed4faa9ce1648696000c3a1fb9bb19e5fad10ecd`)
also rejects the actual public `/` and `/blog/` at source
`a6633fb68b110d07abd5ee5a6049bc15f20f9c7f`: the compiled `.nav-title`
override is missing and the actual authored navigation computes uppercase.
The complete manifest stays identical before and after both observations
(`d7c1a60f6eec182695301dc4715659d0170b6ed351b3435d3c7d1893440480f7`).
No DOM or CSS is injected. The unchanged source is authenticated by official
Docs `38164712296`, publisher `38169153131` and deployment `7000783391`.

The first repair head's native phase run `38178712837`, job `114586493422`,
fails before the casing observer: the inherited custom-directive source guard
compares the entire current tree against historical `67e3d0e` and rejects
unrelated incoming main changes. The raw failure is retained. #8707 owns its
provider repair; this child genuinely branches from its reviewed real head
`13cd6c9cd1e5bd709875bcd355e04ce4501a538b`. The native GitHub Stack's ready
parent prefix may proceed without waiting for this child's manual Docs
campaign. All source, full Docs, protected and public evidence must qualify
the successor head anew; old checks are not transferred.

The successor `294050d2` tooling run `38179266804` preserves a separate
integration failure: job `114589072320` rejects the existing concurrency
fixture's bare HTML because it has no theme stylesheet. Its four original
desktop/mobile `/a/b` and `/a-b` jobs cannot supply the mandatory owned
`.entry-page .features::before` rule. The dependent release child retains the
same genuine failure separately in job `114589191555`.

Repair only that fixture's dependency setup: serve byte-exact production
`style.css`, `features.css` and `letter-case.css` over its real HTTP server and
link them before the unchanged original body. These three files contain the
complete finite CSS ownership required by the new gate. No rules or elements
are invented, and the production gate receives no route or scope exception.
The concurrency fixture is not a substitute for actual built-site packing
or public acceptance. An independent peer reverses this narrow addition to
the exact original whole driver and verifies all routes, workers, deadline,
serial/parallel equality, failed-page custody, links and capture controls
remain intact. The unchanged real-Chromium law passes with zero skips in
36.94 seconds; this fixture measurement grants no full-site speed claim.
All seven production/sidecar/import-dependency blobs remain exact `294050d2`.
Fresh exact successor Actions, protected delivery and actual-source official
Docs/public native casing remain required; all earlier campaigns and failed
or cancelled source runs stay separately identified.
