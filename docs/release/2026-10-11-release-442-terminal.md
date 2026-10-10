# v0.442.0 publication and installed acceptance

The immutable source is published. The original native and gallery checks pass
with public packages; the new configured-base VRT check fails before its first
Art is selected. Keep [#8501](https://github.com/ubugeeei-prod/vize/issues/8501)
open until a later release containing the packed-gallery repair passes the
unchanged installed VRT campaign.

## Immutable source and actual delivery

| Identity             | Value                                                                            |
| -------------------- | -------------------------------------------------------------------------------- |
| Cut C                | `85d5068b82d870f20e300a7ac26fdf7a09653b8c`                                       |
| Shipping H           | `7ba77aef30cb24ee29985e5164fe3a747838cefd`                                       |
| Source PR            | [#8529](https://github.com/ubugeeei-prod/vize/pull/8529), closed without merging |
| Pin                  | `eaa861e3c0eda5d219738e3100746e23fd53a957`                                       |
| Metadata PR          | [#8530](https://github.com/ubugeeei-prod/vize/pull/8530), actually merged        |
| Signed metadata M    | `58cf824af2fb089e3f3485d99c83ca7e01011e71`                                       |
| Actual M parent      | `d55ae011d26272cfdc806922e3d105330731fd24`                                       |
| Annotated tag object | `c01097253f6112773b2ce645f8a0da6f43c2e5f6`, targeting H                          |
| Public version       | [v0.442.0](https://github.com/ubugeeei-prod/vize/releases/tag/v0.442.0)          |

The official [Release run 38070927471](https://github.com/ubugeeei-prod/vize/actions/runs/38070927471)
completed successfully at 18:52:25 UTC on October 10. Its
[operator 38070663385](https://github.com/ubugeeei-prod/vize/actions/runs/38070663385)
completed successfully at 18:53:33 UTC. The operator authenticated publication
after the signed metadata merge; it created the tag at H. No source PR merge or
manual tag promotion is claimed.

All 37 generated-version changes from C to H equal the changes from the actual
metadata parent to M. The complete official 498,824-byte publish catalog is
byte-equal at H and M, SHA256
`16346b5e555b1a9d4c63ecedf3646e3ea684202d7ef90e78ef52d8d65fa9249c`.
All 24 raw publication authorities also agree. Metadata was refreshed once onto
actual main after the stale event-snapshot failures, retaining the source, pin,
Release run and original generated changes. Earlier failed event evidence remains
historical; a later event alone is not a source qualification.

The required exact-H full gates are successful:

- [Full Check](https://github.com/ubugeeei-prod/vize/actions/runs/38071067843)
- [Miri](https://github.com/ubugeeei-prod/vize/actions/runs/38071069329)
- [Fuzz](https://github.com/ubugeeei-prod/vize/actions/runs/38071072096)
- [Real Project Matrix](https://github.com/ubugeeei-prod/vize/actions/runs/38071073579)
- [Full Docs](https://github.com/ubugeeei-prod/vize/actions/runs/38071070880)

The actual protected metadata Check and all required integration workflows also
succeeded. The shipping full Docs run used two workers, completed its four
preflight render cases, 90 pages and 1,420 frame observations. This qualifies the
production build; it does not complete the separate Docs performance comparison
whose runner lost communication.

## Public publication

The targets were derived from H. Every planned public npm archive was downloaded
and matched its SHA512 SRI: 26 packages. Every planned crates.io archive was
downloaded and matched its SHA256: 26 non-yanked versions. The actual Marketplace
version is `0.442.0`; the Zed archive matches its published checksum.

GitHub Release `409139779` is stable and contains all 78 expected uploaded,
nonempty assets with SHA256 metadata. This records the whole expected asset set,
not independent body hashing of all 78 assets. The npm provenance bindings are
retained; cryptographic provenance signature verification is not claimed.

The canonical joined publication receipt has SHA256
`b20163fe38b4bd79d73f856e0a292a477f2cb25f1661e0151262c4e5b5881634`.
It binds the 16 complete source, catalog, merge, tag, workflow and external
publication records. Its public archive verification receipt has SHA256
`de73dc7488178a18fef0766b5ab50d283e91be09b4fac739c9ff3cc945d6d133`.

Open VSX is separate. Its
[optional run 38077879265](https://github.com/ubugeeei-prod/vize/actions/runs/38077879265)
completed successfully. The actual public `ubugeeei.vize@0.442.0` VSIX is
116,752 bytes, SHA256
`0eb9ad86b8ffcd9f1ffbb207d699c0226d293694463c4e33d0fa3893cd4c3aca`,
and equals the frozen GitHub Release VSIX body. The separate receipt SHA256 is
`66a869a5707fa332bea732f4d107c3581806c524724a47612598936a45bb5fd6`.

## Installed Linux result: original checks pass, VRT fails

The single official
[public-consumer run 38077840063](https://github.com/ubugeeei-prod/vize/actions/runs/38077840063)
executed at M using the exact published H/version and one fresh public consumer.
All 19 consumer source authorities agree with H. The 13 original example inputs
and five VRT fixtures also match H exactly. Both native and gallery probes
observed the same actual public Linux addon and package identities.

All eight original DOM/SSR checks pass. The original gallery passes its 65 full
frame observations, six persistent Document tokens, setup-call count, toolbar
updates, sibling/foreign rejection, new-tab initial globals, copied URL,
same-URL reload negative control and one-argument setup hook. These are bounded
original checks, separate from the VRT result.

The configured VRT gallery starts at `/gallery/vrt/`, discovers both original
Left and Right Art files and compiles them without a native error. Its retained
HTML nevertheless requests `/__musea__/assets/index-qkpGvqJF.js`; that request
returns 404. The retained `#app` is empty. The original 15-second Left selector
deadline fails, before any VRT result phase. The final combined acceptance seal
is correctly skipped; no successful `acceptance.json` exists.

The source attribution is a packed-gallery custom-base product defect:
server middleware injects the configured gallery globals without applying the
existing gallery-base HTML rewrite, while assets are served only at the
configured path. The same whole middleware, static-base and gallery build
sources are present at H, M and the later compact merge #8522. That compact
change does not repair this failure. Add an authentic dist-import regression
and deliver the focused repair through source checks, protected merge and a
later included release. Preserve the original fixtures, selectors and deadlines;
do not retry the known-failing `0.442.0` campaign as acceptance.

Artifact `11678972376` retains 113 files. Its 893,619-byte ZIP matches SHA256
`de3dce8daba53e7c7c4c7c70a28539afd01dd5e09cc882b800c11420b192c4fc`.
Every extracted file was independently hashed. The original-pass/VRT-failure
review receipt has SHA256
`028e39945958dd335dbe470d0d93d017905386ecabaedfa872911aaabf66841e`.
The failed installed run remains part of the release record.

## Remaining acceptance and next release

The single Darwin public-install collector and independent review are sealed.
All 203 installed payload files across five packages have complete archive-byte
and SHA512 SRI custody, including the actual loaded native addon and bundled
Corsa 7.0.2. The authority receipt SHA256 is
`3b3f094bcb6fae3991c8863e704ba91458320ceaebac59952e860b1b059da9e8`;
the finite custody seal is
`57c44988b9120824828a6e28336ca2184889183c2c76d6586c974c1ee29af74c`.
The original eleven empty runtime overrides are authenticated; npm user/global
configuration files and cryptographic provenance signatures were not verified.

This same immutable consumer may support the unchanged installed HTML and n8n
slot campaigns and the bounded default-policy replay. Their success is not
inferred from installation, publication, source tests or the Linux original
gallery. No additional consumer installation or old-version replay is required.

The next finite minor release should include genuinely merged urgent #8507
repair #8524 and the packed-gallery custom-base repair when qualified. Ordinary
source work continues independently. Post-cut source changes require their own
included release evidence; #3956, #8090, VRT/props/service and the v1-alpha
release umbrella are not completed by this publication.
