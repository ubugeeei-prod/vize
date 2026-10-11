# Native screenshot identity across frequent Docs deployments

Issue: [#8374](https://github.com/ubugeeei-prod/vize/issues/8374).

## Observed failure

The official `50c7780a5456ad9af9e58a984e51c47c8195c884` publication succeeded in
publisher `38157820045`, deployment `6998799921`, from full Docs `38153837007`.
Its public manifest stayed byte-identical during the narrow HTTP observation.
The new browser receipt expected `use-cycle-list-720-interacted.png` body
`8593f14610a6d3f2be890d61cf9a474c14871d11a9f05418b73ef94676d28514`,
but the ordinary image URL returned the previous `782594c` body
`0491d94fbf17e7238ca2c7803c103fb61a371d9673aeafc600dca08ec6ad11b2`.
The response had `Age: 494`, the old publication's `Last-Modified`, and
`Cache-Control: max-age=14400`. A separate source-query diagnostic returned the
new body. That query was not accepted as proof for the ordinary URL.

## Decision

Give every captured UI, composable and unavailable-storage PNG a filename
containing its complete body SHA256. Generated documentation links that immutable
filename. Preserve the original PNG bytes and stable aliases for existing links;
do not add rendering passes, change viewports, alter native interactions or remove
any screenshot case. The capture receipts retain each actual filename and body hash.

Source-only catalogue rendering remains available before generated captures exist.
Every actual Docs build captures first; the real site browser gate requires full
body hashes in all native PNG URLs and checks the complete HTTP response bytes.
Missing immutable assets or old stable URLs fail that gate. A real old/new PNG
pair reproduces the stale-cache failure and verifies both complete bodies remain
unchanged while their canonical URLs differ.

## Acceptance boundary

Keep the existing 32 SFCs, strict SSR identity, clipboard/scroll sequencing,
scoped-input laws, native preferences, unavailable storage, overflow and link gates.
Require fresh exact-head source and full Docs evidence, protected actual merge,
official publication custody and fresh public native URL/body verification.
Historical public `782594c` image proof and the successful `50c7780` 38-route reader
remain separate evidence. The current stable image URL failure is unfinished until
the immutable screenshot fix is actually published and accepted. Full catalogue
coverage in #8374 and #6101 remains open.
