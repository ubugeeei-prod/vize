# Misskey HMR completions identify the accepted SFC

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The Misskey row of [App E2E run 38866347055](https://github.com/ubugeeei-prod/vize/actions/runs/38866347055/job/116860084982)
failed all three attempts after its first authored edit. The fresh transformed
response and visible DOM marker passed. The next assertion expected one
completed SFC update but observed two, then reached its unchanged 30-second
poll deadline. All 583 initial SFC compilations succeeded.

The server emitted one update containing both
`/src/ui/visitor.vue?vue&vize` and its `.vue.__vize_style_0.css` module.
Vite 8.2 logs each accepted module separately, using the accepted module first
and an optional `via` importer afterward. The observer's substring match for
`/src/ui/visitor.vue` therefore also counted the style module.
[Vite's completion logger](https://github.com/vitejs/vite/blob/v8.2.0/packages/vite/src/shared/hmr.ts#L284).

Parse that completion format and pass only its accepted module to the existing
canonical client-URL predicate. Require the authored SFC pathname and both
`vue` and `vize` query markers. Reject style paths, obsolete TypeScript suffixes,
SSR identities, foreign origins, and a target mentioned only as the importer.
The regression test covers a component/style pair for both edit and repair,
and proves duplicate real SFC completions are still counted.

This fixes an observer mismatch. Preserve every authored edit and repair,
response-body marker assertion, exact request/response/completion count,
server update log, no-reload sentinel, navigation check, source restore, and
existing deadline from the [earlier identity decision](./2026-10-11-6830-authored-hmr-client-identity.md).
Product and upstream source remain unchanged. The retained failed logs and
artifact explain this correction; a fresh complete Misskey Actions row is
still required before claiming browser acceptance.
