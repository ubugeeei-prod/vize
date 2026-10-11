# Nuxt route metadata authority

This record follows [#4103](https://github.com/ubugeeei-prod/vize/issues/4103),
the authored Nuxt head and route oracle. It corrects where the oracle observes
static route properties; the original SSR, hydration, navigation, head, params,
and source-preservation cases remain required.

## Authenticated failure

[App E2E run 38111801880](https://github.com/ubugeeei-prod/vize/actions/runs/38111801880)
tested source `605429c660b328bb03bec5bbfef15961e59fcaf0`. Its `dev:npmx` row
passed 14 tests and failed the authored-head test on all three attempts. The
canonical docs route had the correct path, name, params, and `scrollMargin`.
The sole reported difference was the expectation that `name`, `path`, and
`alias` would also remain inside `route.meta`.

The source pins npmx.dev at `c9e8be91e61eeeae956e2068722caa09e293cc68`.
Read-only Git acquisition verified the page, package manifest, and lockfile
against their exact Git blobs. The original
[docs page](https://github.com/npmx-dev/npmx.dev/blob/c9e8be91e61eeeae956e2068722caa09e293cc68/app/pages/package-docs/%5B...path%5D.vue)
authors `name: 'docs'`, `/package-docs/:path+`, two aliases, and
`scrollMargin: 180`. The runner log confirms Nuxt 4.4.8, Vue 3.5.39, and
Vue Router 5.2.0, matching the pinned manifest.

The original 524,002-byte, 13-entry artifact has SHA-256
`e23f80af7920f72cb6a38e3f1a92e1fa5bee981ac550289a8afd153de732a1ce`.
Its logs, screenshots, and report are failure evidence. It contains no
independent native binary or source-binding JSON; checkout and build custody
come from the original job log.

## Decision

Nuxt 4.4.8 [extracts static route properties](https://github.com/nuxt/nuxt/blob/v4.4.8/packages/nuxt/src/pages/utils.ts)
into the route record. Its
[page-meta transform](https://github.com/nuxt/nuxt/blob/v4.4.8/packages/nuxt/src/pages/plugins/page-meta.ts)
removes those serializable properties from the runtime meta module. Vue Router
[merges each matched record's meta](https://github.com/vuejs/router/blob/v5.2.0/packages/router/src/matcher/index.ts),
without copying the record's name or path into that object.

An independent control ran the exact Nuxt 4.4.8 extraction, transform, and
normalizer on the authenticated authored pages, then resolved them with
Vue Router 5.2.0 and Vue 3.5.39. The installed Nuxt module matched the published
tarball byte for byte. A private copy exposed its internal functions by adding
exports; the original module bytes and function bodies stayed unchanged.
All three docs routes produced `meta: { scrollMargin: 180 }`, matching the
original failure. The accessibility route produced empty meta. This is an
obsolete oracle expectation, not evidence of dropped Vize metadata.

The oracle therefore checks these authorities separately:

| Observation                     | Required value                                                                           |
| ------------------------------- | ---------------------------------------------------------------------------------------- |
| Current name                    | `docs`                                                                                   |
| Current path and full path      | The exact requested URL                                                                  |
| Current params                  | `path: ['nuxt', 'v', '4.0.0']`                                                           |
| Current and matched-record meta | Exactly `{ scrollMargin: 180 }`                                                          |
| Canonical record                | `/package-docs/:path+`, with no `aliasOf`                                                |
| Alias records                   | Exactly `/package/docs/:path+` and `/docs/:path+`, both pointing to the canonical record |

Each original navigation also checks the matched record and the complete
same-name record inventory. Missing, extra, or misdirected aliases fail.
The pinned source contract now includes the authored canonical path. Each
docs/accessibility packet is attached to the Playwright result before assertion.
The fixture, retries, deadlines, hydration checks, and exact head checks retain
their original behavior. Compiler output and fixture setup stay unchanged.

## Completion evidence

The independent reference classifies the expectation; it does not replace the
real App E2E lane. Completion requires the unchanged complete npmx dev suite
on this change's exact source, followed by its protected merge. Old source
checks and the retired release observation do not transfer to a new source.
