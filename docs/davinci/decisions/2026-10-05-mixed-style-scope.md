# Full-SFC mixed style scope

Issue [#7862](https://github.com/ubugeeei-prod/vize/issues/7862), within
[#7856](https://github.com/ubugeeei-prod/vize/issues/7856).

At audited protected main `d1a25ec1da2efca98534520ff8ebe84d34708e3d`,
NAPI single/file-batch/result-batch and WASM adapters provide the component's
aggregate scoped flag as the base style option. The shared full-SFC compiler
ORs that flag with every original block's own flag, incorrectly scoping
ordinary siblings. A full SFC now uses each descriptor block's authored
`scoped` value. Standalone style compilation retains its explicit option.

The complete original 229-byte `multiple-style-blocks` plant comes from
[upstream5489](https://github.com/pikax/vue-benchmarks/blob/5489aee433cd1054b9d72973457498544da7c467/scripts/lib/style-feature-gates.mjs#L15),
blob `c74492077bfc08adb29415b03c718a91eb5cafcd`, source SHA256
`78c44890f7d99f3716c9291712f962bf5fcf4b09efbe2f020a92b3cd6778d35b`.
Its declaration cascade, important flag, custom property, media rule and
ordinary tail stay exact. Source custody and the upstream MIT notice are
retained beside the registered regression; existing fixture bytes stay exact.

The source-built regression emits complete CSS for the original, reversed
block order, ordinary-only and scoped-only inputs across DOM/SSR/Vapor and
both aggregate flags: 24 fixed observations. An independent pinned official
stable Vue compiler compiles each original style block separately. The child
compares every CSS rule's selector/order/media constraint and actual
happy-dom computed values for scoped and ordinary DOM nodes, retaining raw
actual/reference CSS and observations. A separate law keeps standalone
explicit scoped compilation. Source Actions must actually execute both laws;
the unchanged merge queue must execute the full suites and 104 ceilings.

This CSS/DOM observation is not Chromium, Vue component hydration, direct
Vapor SSR or all-style parity. The shared existing compiler is used; native
migration credit stays zero. The published upstream result was generated
2026-09-29 at `8a8848276c52956d7e54e262e5846e41fc922288` with Vize0.429.1.
That failed historical snapshot is not current-source proof. All seventeen
original CSS validators and thirty-three runtime plants in every API/backend/
mode remain mandatory for complete #7856 acceptance. Reference-invalid and
missing-lockfile corpus limitations remain explicit; no timing or ranking
claim follows from this focused fix.

TODO: exact source Actions, root peer review, protected candidate full
execution/instruction ratchet, signed actual main and reporter trailer,
then an authenticated existing-product release. #7856 remains open.

The initial source cc67 contained a stale digest of the new expected-metadata
file after formatting. Update that exact reference and require the runtime
child to authenticate every corpus input/reference before observation. The
original source and production fix remain exact; fresh source Actions are
required, without acceptance transferred from the superseded head.

Fresh source Check 37255740116 at b33 failed: the CSS oracle treated every truthy cssRules as media, although happy-dom CSSStyleRule inherits an empty cssRules collection; it crashed before comparing CSS. Discriminate actual MEDIA_RULE/STYLE_RULE and refuse other/nested rules. Replace the standalone partial selector check with complete authored CSS equality and regenerate the one added test/dev L0 inventory row. Full original source, production correction, all 24 identities, official expected CSS and all ceilings stay unchanged. The source Nuxt 3 lane 37255739543 succeeded separately, without curing the failed Check. Fresh successor source Actions remain required.
