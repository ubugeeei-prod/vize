# Authored HMR observers use Vite-visible SFC identities

For [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), keep the
Misskey and Nuxt UI authored HMR checks bound to the module identity Vize
actually exposes to Vite. The compiler's internal TypeScript suffix is not
the public HTTP module identity.

[PR #6904](https://github.com/ubugeeei-prod/vize/pull/6904) changed that
identity to `.vue?vue&vize` so Nuxt can associate client and server styles
with the same SFC. `toPluginVisibleVirtualId` in
`npm/builder/vite/src/virtual.ts` owns the conversion. Its complete source
has SHA-256
`9587a51bb615792a7475b00605adc64bd0a18cc8373333c49c7a25ac6b4c768b`,
identical at release candidate `605429c660b328bb03bec5bbfef15961e59fcaf0`
and this change's base `cbfc422fab6996eac1b37565dd932f72abb4d0b4`.

The [original App E2E run](https://github.com/ubugeeei-prod/vize/actions/runs/38111801880)
failed both authored HMR checks while waiting for an initial `.vue.ts`
response that this producer cannot emit. Misskey retained its 120-second
initial window; Nuxt UI retained its 180-second initial window and reached
the original row's deadline after three failed attempts. Rendered-page
checks alone do not establish HMR correctness.

Use the authored `.vue` suffix, the current browser origin, and both `vue`
and `vize` query markers. Reject server-only requests, stale `.vue.ts`
suffixes, raw SFC requests, foreign origins, and other authored files.
Hot requests additionally require their original timestamp marker. The
focused tooling law calls the actual public producer and the same predicate
both browser observers consume.

Every authored edit and repair, marker assertion, response-body check,
request count, update log, no-reload sentinel, source restore, and existing
deadline remains required. Successful initial observations record their
actual complete URL and HTTP status in the original job log. Product and
upstream source remain unchanged.

The original failed jobs and their whole artifact archives are retained.
Fresh Actions must pass the complete original Misskey and Nuxt UI suites;
the focused predicate laws alone are not browser or release acceptance.
