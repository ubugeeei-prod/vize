# Keep configured-cache Vue resolution within its source budget

Tracking: [#8618](https://github.com/ubugeeei-prod/vize/pull/8618),
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

Keep Hebilicious’s configured `cacheDir` fix and its original regression intact.
Move the existing `isOptimizedVueDependency` predicate, including its explanation,
into `resolve-optimized-vue.ts`. Keep its public export through `resolve.ts` and
both original call sites. The predicate body and matching behavior are unchanged.
This is a module extraction; it does not change compiler output or replace a
legacy product implementation.

The contributor’s signed commit remains the direct ancestor of the move-only
commit. Its author, message, four original source blobs and parent are retained.
The configuration hook, state field and original test file stay byte-exact.
Reversing only the move and its import/export reconstructs the authored resolver
byte-for-byte. The verified public contributor is credited on the move commit.

## Validation

The approved original Check run `38135113909`, tooling shard 3 job
`114465331168`, failed the existing source-length test: `resolve.ts` grew from
1,271 to 1,290 lines. Its 1,214 tests recorded 1,204 passes, one failure and nine
existing skips. The extraction returns the resolver to 1,271 lines and adds a
24-line module. The unchanged source-length command passes against the same
`31323aaa027b48540b4cf5f80b68155b704a30fc` base, with no limit change.

Formatting, lint with warnings denied, and whitespace checks pass for the moved
source. The authored resolver regression remains mandatory in the existing
package test command. Byte preservation proves this extraction keeps its source
logic; it does not stand in for fresh Actions execution or consumer acceptance.

## Remaining delivery

Keep the contributor’s Draft state and branch intact until the concrete change
is reviewed and its exact remote head is checked. Qualify the published corrected
head through existing Actions, protected integration and a release. Any paired
issue update stays concise. The original source failure and pending remote
outcomes remain separate from the local size-gate pass.

## Frozen reporter oracle (2026-10-12)

The contribution merged as `3bf3a80874dc6caf0dea6404c52f2ffae890728e`.
[#8623](https://github.com/ubugeeei-prod/vize/issues/8623) remains pending
installed acceptance of a successful official successor release. Source packages
and a source native replacement do not establish publication.

Preserve reporter commit `6cb621553b40c6af751c2d4bf2dc32ae09bc5790`, its
original counter, configuration and 154 non-Vize package and snapshot packets.
The frozen runtime is Vite `8.3.2`, Playwright `1.63.0`, Vue and Vapor
`3.6.0-rc.10`, and Vitest `4.1.11`. The original public `0.443.0` plane
fails to update the counter from 1 to 6 in both the default cache and the separate
custom-`cacheDir` probe. Fresh caret resolution to Vite `8.3.4` and Playwright
`1.64.0` passes; it does not replace this frozen regression plane.

Normal `vp pack` and `pnpm pack` on source
`881e229393b611cf9c4c1770ac2b6cef3e00dbe6` produce tarball SHA-256
`3337a9776658ac56497755a2309067fe50bb924300664eb2f0cc3922af486e7f`.
Installing that JS with unchanged public native `0.443.0` passes the original
counter in both cache roots. The native counter module is byte-identical to
source `881e2293` (SHA-256
`52f04083e1c2059ca29ad6f66b9170f0d8c09be8b2d1d5a1d5b6a3bb01c8fe44`),
so this JS integration result does not depend on the newer Vapor provider.

A source `881e2293` native replacement also passes both counter probes after
independently starting with empty optimizer and Vize precompile caches. Each
cold run emits the same 14 complete original SFC modules. The actual test-process
native SHA-256 is
`e7fef4e8e002872d3b922e060cfab0052624881c8a5d0630e8e067c1f3c06ff9`.
The unchanged complete reporter browser suite passes all 11 tests on this cold
source plane. The original 154 non-Vize packets, importers and source hashes
remain unchanged. The public-only successor observer must use a fresh consumer
and public packages, without a source bundle or native replacement.

Retain whole packets rather than only test counts:

- Source JS/public native observation:
  `70d9d16954dc25f580a18b441c1ffa4b13d98a7f2a778df05aa9ea2713b473f3`.
- Cold source JS/native observation:
  `467a40976de5b2de6c06047b80a13729cdc5d989e78df8d163a5fed98059de73`.
- Complete cold default SFC packet:
  `a7906acd804f3f25ce3aba2ed416c08f777bdf7f571a9605ec0febeeb0233d53`.
- Complete cold custom-cache SFC packet:
  `e3bf637e691bd46c3dbe2e7abd77b347340c17f4b33a4734722d223fccf8e959`.
- Original 11-test cold browser report:
  `19dd79911974f9b4b0d331affbf2aeb77a35c1fac7f89db852e48f7773768826`.

## Bounded native identity follow-up

TODO for the Vite cache owner: resolve the actual loaded pnpm platform binary
when keying a same-version native rebuild, and pin a warm rollover control. The
current identity scans only `@vizejs/native`'s wrapper directory for `.node`
files; its pnpm platform package is a sibling, so that directory contributes an
empty binary vector at version `0.443.0`. Changing only the platform binary can
therefore reuse earlier compiled SFCs. A release version change creates a new
cache namespace; this bounded same-version follow-up does not broaden #8623 or
change its original counter/configuration.

The preserved public-native and cold source manifests share key
`06e1ffd7b5dae0929f943b44c5257201`. Their SHA-256 values are respectively
`ba77c824abe7eece7b30197241a23945e56c89c5f50eceeeb6b2d476b40af2bf`
and `bf017a94028ea62548146858f7a14a4de0b92e11baa609b1c6aa7a4c705d00f4`.
All 14 sources match, but six complete module bodies differ and explain the five
failures in the earlier warm source-swap diagnostic. Preserve those warm logs as
setup diagnostics. Independent full comparison SHA-256:
`30307a3337b6562a20a743e7fd9f836cb0f719589b016421064a2f1f5e053690`.
The cold reruns are the source emission authority.

## Same-version native cache follow-up

The [loaded-body identity decision](./2026-10-12-loaded-native-cache-identity.md)
preserves the original fourteen-module/154-entry source controls and adds an
independent warm rollover and loaded-memory negative control. Capture the
successful native body's identity once rather than scanning the wrapper
directory or hashing a later replacement file. Source qualification, protected
delivery and installed official-release acceptance remain separate.
