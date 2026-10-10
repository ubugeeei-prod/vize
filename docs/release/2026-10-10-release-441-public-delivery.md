# v0.441.0 public delivery

v0.441.0 delivers the focused inline-Art component-binding repair from
[#8462](https://github.com/ubugeeei-prod/vize/pull/8462). The original public
0.440 Musea replay stopped at the default preview because it rendered a literal
`museacomponent`. The same public gallery and globals laws now pass against the
actual 0.441 npm packages in Chromium.

Related acceptance: [#8329](https://github.com/ubugeeei-prod/vize/issues/8329)
and [#8337](https://github.com/ubugeeei-prod/vize/issues/8337). The previous
[0.440 delivery record](./2026-10-10-release-440-public-delivery.md) retains its
failed public Musea result and successful native report acceptance.

## Immutable source and protected promotion

| Receipt                                | Exact identity                                                                                        |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Cut C and actual #8462 merge           | `61e0e3309a85dcb319fd5133d36c7d7f4f44964f`                                                            |
| Frozen source H                        | `bb83dd9817837ab4a87b09b32646389fb570a705`                                                            |
| Source PR                              | [#8473](https://github.com/ubugeeei-prod/vize/pull/8473), closed unmerged after external verification |
| Preserved pin                          | `fb4200b04db96c85d7dc2a138b6dd14da967fb81`                                                            |
| Metadata PR                            | [#8474](https://github.com/ubugeeei-prod/vize/pull/8474), merged 09:53:42 UTC                         |
| Qualified metadata head                | `ac47785748767314ca34e60b7758d928b566bab7`                                                            |
| Protected candidate and signed merge M | `9c9ffb8bea95f75bfc7d148e63d5da76b02e7a33`                                                            |
| Actual M parent                        | `edaf93334291c31e79c0af27ecdec7ec7a3ec9a1`                                                            |
| Annotated tag object T                 | `b151dee0aae7a6e69df459ee34092eb61122ed61`                                                            |
| Tag target                             | `v0.441.0` points to exact H                                                                          |
| Original Release R                     | [38037918678](https://github.com/ubugeeei-prod/vize/actions/runs/38037918678), attempt 2              |
| Same-source resume operator            | [38041013822](https://github.com/ubugeeei-prod/vize/actions/runs/38041013822)                         |

H has the sole parent C. The generated metadata was integrated on newer main,
so M has a different source tree from H. The complete official publication
catalogs at H, the refreshed metadata head and the actual protected candidate
are byte-equal: 492,530 bytes, SHA256
`bc7190730cf5e4fe782a994385af4619c57319b230d8ad32ba669566891d98bb`.
All 24 raw public-plan authority blobs also match. The original version-only
change and its refreshed form have the exact same additions and deletions
across all 37 paths.

The official operator created T at 09:54:33 UTC. Its annotation binds C/H/R,
both PRs, the qualified metadata head, M, M's parent and the protected candidate.
The actual metadata commit has a verified valid signature. The annotated tag
is unsigned; these are distinct observations.

## Qualification and retained recovery

The ordinary source checks and every required full gate passed at H:

| Full gate           | Actual successful run                                                                    |
| ------------------- | ---------------------------------------------------------------------------------------- |
| Check               | [38038027304](https://github.com/ubugeeei-prod/vize/actions/runs/38038027304), attempt 2 |
| Miri                | [38038028535](https://github.com/ubugeeei-prod/vize/actions/runs/38038028535)            |
| Docs build          | [38038029851](https://github.com/ubugeeei-prod/vize/actions/runs/38038029851)            |
| Fuzz                | [38038031177](https://github.com/ubugeeei-prod/vize/actions/runs/38038031177)            |
| Real Project Matrix | [38038032356](https://github.com/ubugeeei-prod/vize/actions/runs/38038032356)            |

The first full Check failed while downloading pinned VSCode 1.107.1, before the
editor tests: `ETIMEDOUT` and `ENETUNREACH`. Release preflight correctly refused
the failed gate. The original operator ended without a tag, retaining the
source and artifacts. One unchanged failed-job rerun downloaded the same pinned
editor and passed the actual VSCode, Neovim, Vim, Emacs, Zed and Helix tests.
The same-source official resume reused R, H, both PRs and the original artifacts.
The earlier failed receipts remain historical evidence.

Metadata event snapshots also correctly refused stale PR body/base custody and
incorrect synthetic parents. Recovery used one reviewed generated-version-only
refresh onto actual main. It preserved the immutable source, pin, publication
catalog and R, then passed fresh source/native checks and the actual protected
[Check 38041635888](https://github.com/ubugeeei-prod/vize/actions/runs/38041635888)
and all four required integration workflows. No tag movement, manual candidate
merge, gate waiver or red-candidate admission was used.

## Terminal public publication

R completed successfully at 10:27:21 UTC: 47 successful jobs and the intentional
crates.io handoff skip, with no failed or pending jobs. The resumed operator
completed successfully at 10:28:31 UTC. The stable, non-draft
[GitHub Release](https://github.com/ubugeeei-prod/vize/releases/tag/v0.441.0)
was published at 10:27:17 UTC with the exact 78 expected uploaded assets, each
having nonzero size and server SHA256 metadata.

Independent invocation of the official public verifier checked the raw-H plan:
all 26 npm archives against their public SHA512 SRI, all 26 non-yanked crates
against actual archive SHA256, Marketplace 0.441.0 and the actual Zed archive
against its public checksum. Publication channels and installed behavior are
separate proofs; server asset digests alone do not claim independent body or
cryptographic signature verification for every GitHub asset.

The separate optional
[Open VSX workflow 38045161373](https://github.com/ubugeeei-prod/vize/actions/runs/38045161373)
also succeeded. Its actual public 0.441 VSIX body is byte-equal to the frozen
GitHub Release VSIX: 116,751 bytes, SHA256
`f8b37f6ef341a048eeb8dbcf42e980d8f4ead4730cfb0af31b78b52a2b64117b`.
The registry reports no signature; no cryptographic signature claim is made.

## Original installed public acceptance

The official fresh Linux npm consumer
[38045154612](https://github.com/ubugeeei-prod/vize/actions/runs/38045154612)
passed on its first attempt. Its collector ran on actual main M, with all 11
TypeScript authority files byte-equal to the protected merged observer from
[#8464](https://github.com/ubugeeei-prod/vize/pull/8464). Its 13 example fixtures
were authenticated against raw H, and its lock, package versions, public SRI,
loaded native path and successful return were retained. The actual loaded public
Linux native SHA256 is
`9b20e1b403e5057be6265394ce60ac56b6631e941d7b75a2ce3b335adf2754cc`.

The combined seal contains all eight original whole-result native DOM/SSR cases
and ordinary-is controls, plus every original Musea browser phase. Its 65 frame
observations cover six real previews, default globals and all three toolbar
changes without replacing their Documents or rerunning setup. New-tab globals
arrive before setup. Delivered sibling and foreign commands are refused. Art
navigation, a copied URL in a clean context, a real same-URL reload that fails
the Document identity oracle, and all six unconfigured one-argument hooks pass.
There are no browser errors. The rendered gallery screenshot contains the actual
button rather than the unresolved inline-Art host.

The retained
[artifact 11666543111](https://github.com/ubugeeei-prod/vize/actions/runs/38045154612/artifacts/11666543111)
contains 104 files, including 89 authenticated HTTP response bodies. Its actual
920,532-byte archive matches the Actions SHA256. The complete seal embeds the
unchanged native, Musea, preparation and publication receipts and the exact
C/H/tag/source-PR/R identity. This proves the bounded original gallery/globals
and native paths; hosted production audits, VRT and props-editor completion
remain separately qualified work.

A second independent review reconstructed every complete frame observation,
rehashed all 104 files and joined the raw-H fixtures, collector authority and
public archive receipts. After that review, #8329 and #8337 were actually closed
with concise release and installed-acceptance evidence. All old failed release
and public-browser receipts remain retained.

| Retained proof                             | SHA256                                                             |
| ------------------------------------------ | ------------------------------------------------------------------ |
| Joined canonical source/publication review | `f0f2f6b0059db36174611ca53c8b99e222c86f39da54341ec44024c340e6922b` |
| Mandatory public archive/channel receipt   | `9ac5b6a36f0ebdb83451108ba8806ecaa9cba2e1245a880039866f9ce31426d4` |
| Actual installed Actions archive           | `564b2919cb971859cf44e95684c6f020487db5f2645c8b8b72bb4a04dbf40eeb` |
| Combined installed acceptance seal         | `36a5c166cf50b14aa7dd538334212eee8015f76f70802e1f1de814d21a5fd593` |
| Whole Musea browser receipt                | `975ff8fb87689e015563f6f82a2301b3713a221be82a2c133541270e87fbe0eb` |
| Independent installed receipt review       | `a09990c9af1edbd500b08892fb12b0050d73bb9d6b921fb6fef340cda833d59a` |
| Second independent whole installed review  | `b9600d89ccba27135a16d30b2d13546d7e04e41bb329b65b9cebe59d0e3a150e` |
| Separate Open VSX body review              | `28d290e6ae281777d363b7b4a9146234e0ae8eabd1cbdb40f31f9b2fa31fc963` |

## Following finite minor

Verified mandatory publication releases the catalog and manifest hold for
separately reviewed source work. Changes merged after C belong to the next
finite minor, even if they occur in M's ancestry. In particular, the nested-body
repair #8457 and n8n slot-policy commits #8422/#8444 are excluded from H and gain
no 0.441 installed acceptance credit. Their next genuinely including public
release must receive its own installed receipt and original-case replays.
Do not reuse an older consumer's authority or wait for every umbrella to finish
before releasing a coherent, actually merged snapshot.
