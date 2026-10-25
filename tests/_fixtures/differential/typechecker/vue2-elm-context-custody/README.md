# Vue 2 diagnostic context successor (#10529)

The historical `vue2-elm-check.snap` remains byte-identical. The live Vue 2
application test now compares its entire result with
`vue2-elm-context-check.snap`, captured from source revision
`c8c4e052e6002c05ae3a4ef645458b25033da1eb`, full Check run `40166711835`,
Vue parity job `121089153833`, artifact `vue-parity-snapshot-differences`.

Both packets contain all 61 files and 408 errors, with zero warnings. Every
historical canonical message, severity, position, code and ordering remains
present. The successor appends authored source context to exactly 92 entries,
matching the intentional JSON reporting contract introduced by #10335.

`custody.json` records the two complete packet digests and 92 independently
reviewed line witnesses. The 19 source blobs were fetched from the pinned
upstream commit `b85ff0220366be863868269177a4abf74c7a85ca`; their SHA-256 and
git blob IDs are retained alongside the full authored lines and binding labels.
The upstream source is GPL-2.0-only, as recorded by the existing fixture registry.

The custody test builds the full expected successor from the preserved old
packet and those source witnesses. It never strips context from current output
or changes diagnostic comparisons or budgets. It also rejects altered canonical
messages, coordinates, contexts, counts and file inventory. Updating the live
snapshot alone cannot replace either frozen packet unnoticed.

Reproduce the packet acquisition with:

```sh
gh run download 40166711835 --repo ubugeeei-prod/vize \
  --name vue-parity-snapshot-differences --dir /tmp/vue2-context-capture
node --test tests/tooling/vue2-diagnostic-context-custody.test.mjs
```

The full Vue parity lane continues to execute the original untouched app,
byte-stable repeated checks, authored block-range checks, and the intentional
JavaScript-error repair test. Pinia message-protocol comparison is separate.
