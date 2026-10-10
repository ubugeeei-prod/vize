# v0.440.0 public delivery

Related acceptance: [#8337](https://github.com/ubugeeei-prod/vize/issues/8337).
Original reports: [#8328](https://github.com/ubugeeei-prod/vize/issues/8328)
and [#8329](https://github.com/ubugeeei-prod/vize/issues/8329).

## Immutable source and promotion

| Receipt                                | Exact identity                                                                                            |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Cut C                                  | `e5702be6d0119a716499827fa7b918ce8c9dcbae`                                                                |
| Source H                               | `bf2cd911c268605588b2ceedbc011e36c79d113c`                                                                |
| Source PR                              | [#8401](https://github.com/ubugeeei-prod/vize/pull/8401), closed after verified publication, never merged |
| Pin object                             | `8a529462dcbe9b5489b1882e195ae9e82ada6c31`                                                                |
| Metadata head                          | `510d72772ebfc0348e0392e66e425481a790eccf`                                                                |
| Metadata PR                            | [#8402](https://github.com/ubugeeei-prod/vize/pull/8402), merged 2026-10-10 06:45:27 UTC                  |
| Protected candidate and actual merge M | `2f723955e866d8cdb273fbb4e56966a002866d74`                                                                |
| Actual M parent                        | `c3a5d2e94609dbdc434cf2ccf694161190a8a423`                                                                |
| Annotated tag object T                 | `e6a4a955e5374e5748157dc261208e49747d10b2`                                                                |
| Tag target                             | `v0.440.0` points to exact H                                                                              |
| Pinned Release R                       | [38026960473](https://github.com/ubugeeei-prod/vize/actions/runs/38026960473)                             |
| Official operator                      | [38026768251](https://github.com/ubugeeei-prod/vize/actions/runs/38026768251)                             |

H has the sole parent C. The metadata PR head's complete tree equals H.
The final protected candidate equals the actual metadata merge. Its current-main
source additions do not alter the frozen publication catalog and authority bytes. The official operator promoted the
annotated tag at 06:46:13 UTC after the protected checks passed. Its annotation
binds C/H/R, both PRs, the metadata head, M and its parent. The tag is unsigned;
this record makes no cryptographic tag-signature claim.

The source includes the public native tag-classification repair from
[#8360](https://github.com/ubugeeei-prod/vize/pull/8360), actual commit
`63dc7edee87548548f37ebbd3419c636efc49fd1`, and the open-buffer alias repair from
[#8326](https://github.com/ubugeeei-prod/vize/pull/8326), actual commit C.

## Terminal publication

R completed successfully at 07:25:27 UTC: 47 successful jobs and the intentional
crates.io handoff skip, with no failed or pending jobs. The official operator
also completed successfully. The stable, non-draft GitHub Release was published
at 07:25:24 UTC with 78 assets.

An independent invocation of the official public publication verifier passed
every planned archive and channel at 07:26:35 UTC. Its registry-only observation
SHA256 is `7a8e06a4c7b1664a184b045637a17a9ae6940f57475952ef9566e961b62a0658`.
The Zed archive SHA256 is
`0a5b72c6734714c68e1f2c7bdd76d7efbf351279a441217292f6f8cbe82ea87f`.
Publication and manifest holds were released only after that full verification.
The closed source receipt retains its exact branch and commit.

This verifies publication; it does not claim that both installed products passed.
The optional official Open VSX run
[38034426716](https://github.com/ubugeeei-prod/vize/actions/runs/38034426716)
also completed successfully. Its anonymous exact namespace/name/version and own
VSIX download identity were independently verified; its separate metadata
observation SHA256 is
`dc9c93aa10202340909e2da650b8f90091085fd9928fc4808979fdacb1329f67`.

## Exact-source gates

All five complete H gates passed before metadata queue admission:

- [Check 38027074977](https://github.com/ubugeeei-prod/vize/actions/runs/38027074977)
- [Miri 38027075884](https://github.com/ubugeeei-prod/vize/actions/runs/38027075884)
- [Docs 38027076782](https://github.com/ubugeeei-prod/vize/actions/runs/38027076782)
- [Fuzz 38027077771](https://github.com/ubugeeei-prod/vize/actions/runs/38027077771)
- [Real Project Matrix 38027078664](https://github.com/ubugeeei-prod/vize/actions/runs/38027078664)

R's platform builds, packed installation smoke checks and complete safety
preflight also passed before admission. The final protected candidate passed
[Check 38030676988](https://github.com/ubugeeei-prod/vize/actions/runs/38030676988)
and its Musea, n8n, scoped Nuxt and Nuxt 3 integration workflows. Earlier queue
candidates were replaced as their preceding queue prefix changed; their green
states were not used as the final candidate's proof.

## Retained runtime failure

Metadata Check 38027451698 attempt 1 had one fatal Node child exit with
SIGSEGV 11 in Rust shard 2/4, while replaying the unchanged mounted
`object-positional-toggle-open` case. Complete VDOM output was printed, but the
fatal exit correctly rejected the test before oracle comparison. The worker's
synthetic checkout `1d0f18e51a73f627922690bc4592689c927e080e` had the exact H tree;
its actual spawned Node engine identity was not captured.

The failed shard alone was rerun after the run became terminal. Attempt 2
passed all 4,183 tests with the same source and strict exit assertion. Independent
image/core and 512-child diagnostics also passed. They do not identify the
original crash's cause or its engine. [#7951](https://github.com/ubugeeei-prod/vize/issues/7951)
remains responsible for that unresolved runtime failure; successful replay is
not a cause or closure claim.

## Public acceptance boundary

The mandatory H-derived publication plan contains 26 npm packages, 26 crates,
Marketplace `ubugeeei.vize@0.440.0`, and the own GitHub Zed extension archive.
Every npm tarball must match SHA512 SRI and every non-yanked crate archive its
registry SHA256. Open VSX is independently observed and published through its
optional official workflow. Package availability alone is not installed-product
acceptance.

The independent original 0.439 replay
[38032119383](https://github.com/ubugeeei-prod/vize/actions/runs/38032119383)
retained the expected #8328 DOM failure and also exposed a real Musea gallery
failure: the default variant frame did not render the expected component. The
strict browser check ran independently after failed native output and still
rejected the combined acceptance seal. Local public-package diagnosis found
correct globals, one setup call and document UUIDs in all six frames, but literal
`<museacomponent>` nodes instead of the expected rendered button. No JS or HTTP
failure was observed. The exact public 0.440 comparison must establish whether
its native classification repair resolves this path; no old-release proof is
transferred to the new release.

The exact 0.440 Linux public installation replay
[38034417442](https://github.com/ubugeeei-prod/vize/actions/runs/38034417442)
passed all eight native whole-output DOM/SSR cases and ordinary-is controls.
The actual loaded public Linux native binary SHA256 was
`131bb07c04960f8457e37544b378ebe455dddc630856cf415be417e6245a11b6`.
Musea then failed the unchanged default-frame 30-second predicate at
`gallery.ts:9` / `browser.ts:95`, so the overall workflow failed and produced
no combined acceptance seal. Its retained
[artifact 11663024620](https://github.com/ubugeeei-prod/vize/actions/runs/38034417442/artifacts/11663024620)
has archive SHA256
`e460c5938d7f434155f95a64413f8631cacfd750d210afc964ba6de201acabd1`.

Independent public 0.440 macOS diagnosis observed the same unresolved inline-Art
host in all six frames with correct globals and one setup call. Therefore
[#8462](https://github.com/ubugeeei-prod/vize/pull/8462)'s focused component-binding
repair is required in the following source cut. The source-native Chromium
result is qualification for that repair, not public completion. #8329 and #8337
remain open until the new included release passes their original installed laws.

The single official Darwin ARM64 collector also succeeded and its independent
review rehashed all 202 installed archive files, the complete payload manifest,
lock and native loader journal. Its immutable receipt SHA256 is
`ad20350f3e7e9d644e311abba179c6f7f61fa0cef8e9a3a9655cfb017fb35ce3`;
the reviewed eight-file collector snapshot SHA256 is
`a26411442a3fde2590d969160fd83b97685e9019d2de4decb0c103a376f10786`.
It authenticates the public consumer used for bounded supplemental native and
n8n observations. It does not prove those campaigns completed or assert a
cryptographic signature verification.

A separate reviewed native consumer then preserved the reporter's exact
multiline template and trailing LF, passed 18 complete DOM/SSR template packets,
executed nine complete SFC SSR renders, and verified 18 whole browser DOM
observations across initial rendering and prop updates. Its browser receipt
SHA256 is `b4feb893176b7a1fd1b2ad6d6fb592bb3d9c3edfd798aca3190a144d5a735515`;
the linked native/SSR receipt SHA256 is
`eb708649ad8f29b38668bcc1ab461cf66f941890a005f41259ef3525dd6a92a6`.
The official consumer's full payload and lock remained unchanged after the
browser ran. Root's independent review SHA256 is
`f62d20375c2159ce41be63cca73cba803f0ac7486d9810fd00804103b0023917`.
The retained scripts, source fixtures, raw receipts and browser capture archive
SHA256 is `8d9e2050a9903657724b37af00c4363f7d63e6c5a8b066b8e18e9f80490cd368`.
This completes the bounded original #8328 path; it grants no Musea credit.

## Following finite minor

After terminal 0.440 publication and complete planned-channel verification,
publication-authority and manifest holds may be released for separately reviewed,
exact-head-green source work. Those changes are excluded from H. The next minor
uses a fresh finite snapshot of actually merged main, including the third-party
CSS repair #8358 and other coherent fixes merged after C. The reproduced public 0.440
Musea failure requires its focused resolver repair to actually merge before the
next source pin. Do not wait for every umbrella issue, rewrite the 0.440 tag,
or count a source-only receipt as released acceptance.
