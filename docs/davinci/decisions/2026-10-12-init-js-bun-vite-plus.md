# Fresh init JavaScript coverage through Bun and Vite+

Issue: [#3956](https://github.com/ubugeeei-prod/vize/issues/3956).

## Decision

Append the existing `vite-vue-js-checkjs` project after the original eight
fresh-install matrix cells, once with Bun and once through the Vite+ package
manager runner. Preserve those eight complete objects and their order. The
Vite+ runner operates the same ordinary Vite JavaScript project; this does
not introduce a Vite+ JavaScript scaffold or a new project shape.

The earlier matrix exercised this JavaScript project through npm, pnpm and
Yarn, while Bun and Vite+ exercised TypeScript projects only. Every declared
package-manager runner now executes the same authored JavaScript/checkJs
clean, broken and repaired diagnostic oracle. This is an installed-artifact
coverage repair, not evidence of a previously observed Bun or Vite+ defect.

## Preserved contracts

The manager definitions, dependency commands, authenticated tarball redirects,
all project source/configuration bytes, current config-free projection and
fresh-install driver stay unchanged. The separate complete stdout expectation
helper selects the source-authored detected manager for ordinary Vite shapes
and retains `vp` for the existing Vite+ TypeScript shape. This follows the
unchanged installer contract in `npm/cli/src/init/plan.ts::resolveInstaller`: a
Vite+ project selects `vp`, otherwise the lockfile selects its package manager.
A Vite project bootstrapped through `vp` owns a pnpm lockfile and correctly
prints the pnpm installation command; running its manager through `vp` is
still checked independently. Both new cells use the existing dry-run,
apply and idempotency stdout comparisons, complete generated file/dependency
plan, project-local version/Corsa discovery and poisoned-PATH generated check.
The original broken `src/main.js` reports exactly one Error/TS2322 at 5:7,
exits normally with code one, and repairs to the original complete source.
No source imports from the checkout, process-exit requirements, diagnostic
assertions, performance ceilings or supported-host policies are relaxed.

The two existing complete matrix assertions retain their original eight
expectations and append the two declared new cells. All other authored
fixture and test expectations remain unchanged.

## Qualification and limits

Require exact-head Actions and the existing Native Smoke workflow: six host
native workers plus twelve fresh-install workers across six hosts and Node
22/24. Each fresh worker must reach all ten matrix completion markers and
pass the unchanged full runtime assertions; failures, skips and unsupported
platforms are reported separately. Source tests do not substitute for those
executions. Protected merge and a later published-artifact replay are separate
requirements.

Initial source checks pass all 35 existing focused fresh-init, toolchain,
config-free, Yarn-archive and typechecker controls without skips. The preserved
JavaScript shape SHA256 is
`b1f80058b0b54048db35217f44df29e2a046c77dc8ce8853361d9169458d4048`.
The manager, config-free projection, packed driver and both whole Yarn
fixtures remain byte-exact; the foundation retains all 350 rows and every
incoming clause. This local evidence does not execute either new install cell.

## Preserved initial failures

Native Smoke [38154287351](https://github.com/ubugeeei-prod/vize/actions/runs/38154287351)
on original `538069c8953149eddd65759ebf83ee71fe8c79f2` first passed all eight
earlier cells and the new Bun JavaScript cell on macOS arm64 Node22/24. The
final ordinary Vite JavaScript cell bootstrapped through `vp` was correctly
rejected by the entire dry stdout comparison: its legitimate pnpm installation
plan differed from the helper's unconditional vp runner expectation. Full
original job logs and complete actual/expected packets are retained in Actions;
these partial observations do not qualify the whole workflow or final cell.

The helper repair changes only that declared installer choice. All 24 original
stdout packets across eight cells and three modes stay byte-exact, alongside
both original whole Yarn fixtures. New complete authored ordinary-Vite
Bun/pnpm stdout laws reproduce the original vp failure before this change.
All 37 local source controls pass after the repair, without skips. Fresh
exact-head source/native/full-host qualification is still required.

The original Windows x64 Node22/24 workers fail earlier, in the first
spaced/Unicode npm TypeScript cell: the legitimate clean project returns exit
two because Rolldown cannot resolve the config wrapper's extended Windows
`\\?\C:` import path. This is a separate production config-loader defect,
not a missing-matrix expectation. Preserve those full original failures and
fix the JavaScript import boundary without moving the fixture, dropping the
path control or accepting configuration failure. A complete all-host successor
remains required; the partial Unix Bun observations do not establish delivery.

This narrow matrix repair does not complete #3956. Nuxt/monorepo shapes, musl,
packaged editor and authored LSP lifecycle acceptance, workspace-trust and
remaining failure/timing/size evidence remain open. Only the release owner
starts or publishes official cuts and dispatches their public consumers.
