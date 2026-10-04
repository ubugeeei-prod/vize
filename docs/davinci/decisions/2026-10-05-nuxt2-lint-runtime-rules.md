# Nuxt major-specific generated runtime lint rules

Paired issue: [#7828](https://github.com/ubugeeei-prod/vize/issues/7828).

## Decision

`buildNuxtLintPlan(features, dirs, nuxtVersion)` accepts an optional `2 | 3 | 4`
major. Its default stays `3`, preserving the complete original modern oracle
and existing two-argument callers. The Nuxt module passes the detected live
major. Nuxt 2 omits `nuxt/prefer-import-meta` and the `nuxt/pages` block for
`nuxt/no-page-meta-runtime-values`; all other ordered blocks stay intact.
Project rules and addons retain their existing precedence and explicit choices.

Nuxt 2 supports `process.client` and `process.server`, documented by the
[official process helpers](https://v2.nuxt.com/docs/concepts/context-helpers/#process-helpers).
The actual pinned [Nuxt 2.17.3 public constructor version](https://github.com/nuxt/nuxt/blob/v2.17.3/packages/core/src/nuxt.js)
is `Nuxt.version`, with a leading `v`. Version detection retains the precedence
of `_version`, instance `version`, and `options._nuxtVersion`, then recognizes
that public constructor authority. Missing or unrecognized versions still fall
back to the existing modern plan at the generation boundary.

## Authored regression and source custody

The new `test/nuxt-version-compat` corpus retains the issue's seven-line
`process.client` / `process.server` function. Its exact 115-byte source has
SHA-256 `2dcfa5bf523c99a7a669a453ddeca8085f5673fb80f01ac398cdf1db191c3ed6`. Nuxt 2 expects no diagnostics;
Nuxt 3 and 4 each retain both migration diagnostics, in original source order,
including full messages/help, severity, filename and exact byte/line labels.
The plan tests independently fix every remaining block, glob and severity.
The prior `@nuxt/eslint@1.16.0` whole-plan and byte-artifact recordings remain
unchanged. Existing Nuxt 2 build-directory regeneration also checks omission
of both modern-only rules after the real hook reruns.

The Nuxt 2 workflow builds both candidate JavaScript packages and preserves
the locked `nuxt@2.17.3` webpack/SSR fixture and all original route assertions.
After those checks it separately loads genuine Nuxt 2 with lint enabled,
using the official `loadNuxt` config override seam. Original fixture/config
bytes are retained. Candidate installed dist inventories must equal the
freshly packed module and shared preset before the probe runs. Generated
configuration and whole oxlint JSON reports are uploaded with hashes.
The default module artifact remains captured. Actual CLI execution uses the
public `lint.configFile` option at the fixture root: the repository's oxlint
1.78 rejects the existing emitter's parent-relative ignore patterns before
lint runs. This separate compatibility limitation remains unresolved; the
probe does not rewrite generated configuration or suppress its failure.
The lint-rule oracle uses the fixture's pinned published plugin/native
`0.429.1`; this receipt qualifies JavaScript configuration, not a current
native compiler build. Nuxt 3/4 controls exercise authored plans on the same
original source; they do not claim a second real Nuxt runtime build.

## Validation and remaining gates

Before the fix, the authored Nuxt 2 plan fails while the three modern/default
controls pass. After the fix, all nine focused plan/builder laws pass locally.
Hosted exact-source plan/oracle, genuine Nuxt 2 lint/webpack/SSR checks, protected
queue checks and actual signed merge remain required. The issue stays open
until actual tested delivery. No Davinci default, dialect completion or type
checker performance claim follows from this ordinary integration fix.
