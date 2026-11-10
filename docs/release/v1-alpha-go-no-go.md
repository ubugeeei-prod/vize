# Vize v1 Alpha Go/No-Go Checklist

Use this checklist for every v1 alpha release candidate. The release captain freezes an exact
`main` source commit and dispatches the supported [Release workflow](../../.github/workflows/release.yml)
with an explicit `1.0.0-alpha.N` version. The workflow stamps a child candidate commit, pushes it to
`refs/release-candidates/<source commit>`, and creates the tag only after the candidate artifacts and
credential-free smoke checks pass. It never rewrites `main`. Each surface owner signs off only on
the section they operate, and the release captain owns the final go/no-go decision.

## Owners

| Surface                  | Owner           | Required sign-off                                                        |
| ------------------------ | --------------- | ------------------------------------------------------------------------ |
| Release coordination     | Release captain | Version, tag, release notes, and final go/no-go                          |
| npm packages             | npm owner       | Package artifacts, native optional deps, WASM, dist-tags, smoke installs |
| crates.io                | Rust owner      | Crate publish order, trusted publishing, propagation, yanks if needed    |
| Editor marketplace       | Editor owner    | VS Code package, pre-release channel, extension smoke                    |
| Docs and GitHub Pages    | Docs owner      | Docs build, deploy, release post, search index                           |
| GitHub release artifacts | Release captain | CLI archives, checksums, generated notes, prerelease flag                |

## Pre-Tag Gate

- [ ] All alpha qualification checks are green on the exact source commit selected for release: ([PT-1 evidence](./davinci-v1-evidence.md#pre-tag-gate))
  - [Check](../../.github/workflows/check.yml)
  - [Benchmark](../../.github/workflows/benchmark.yml)
  - [App E2E](../../.github/workflows/e2e.yml) for `dev`, `preview`, and `build`
  - [Docs Build](../../.github/workflows/build-docs.yml) evidence on the exact target commit
- [ ] [Fuzz](../../.github/workflows/fuzz.yml) status, seeded corpus health, and uploaded
      reproducers are reviewed when parser/compiler surfaces changed. ([PT-2 evidence](./davinci-v1-evidence.md#pre-tag-gate))
- [ ] No release-blocking draft PR, open P0/P1 fix request, or failing required workflow remains. ([PT-3 evidence](./davinci-v1-evidence.md#pre-tag-gate))
- [ ] Version is agreed and matches the intended channel, for example `1.0.0-alpha.N`. ([PT-4 evidence](./davinci-v1-evidence.md#pre-tag-gate))
- [ ] Changelog or release post draft exists under `docs/content/blog/releases/`. ([PT-5 evidence](./davinci-v1-evidence.md#pre-tag-gate))
- [ ] Local smoke commands pass from a clean checkout: ([PT-6 evidence](./davinci-v1-evidence.md#pre-tag-gate))

```bash
vp install --frozen-lockfile
vp run --workspace-root check:ci
vp run --workspace-root test:scripts
cargo test --workspace
cargo audit --deny warnings
vp run --workspace-root build:packages
```

- [ ] Package-specific smoke checks pass when relevant: ([PT-7 evidence](./davinci-v1-evidence.md#pre-tag-gate))

```bash
vp run --filter './npm/builder/vite-musea' test
vp run --filter './npm/builder/vite-musea' build
vp run --filter './npm/native' build:debug
```

## Release Candidate and Tag Gate

- [ ] Release captain records the latest green `main` source commit, confirms that it still contains
      every approved alpha change, and dispatches the workflow as an authenticated maintainer: ([RP-1 evidence](./davinci-v1-evidence.md#release-pr-and-promotion-gate))

```bash
git fetch origin main --tags --prune
SOURCE_SHA="$(git rev-parse origin/main)"
gh workflow run release.yml --repo ubugeeei-prod/vize --ref main \
  -f source="$SOURCE_SHA" -f version="1.0.0-alpha.N"
```

- [ ] The run records the requested source SHA and creates one version-stamped candidate whose sole
      parent is that source commit. The source workspace version is not expected to change. ([RP-2 evidence](./davinci-v1-evidence.md#release-pr-and-promotion-gate))
- [ ] CLI, native, WASM, JavaScript packages, universal editor artifacts, package tarball smoke, and
      editor-host smoke pass for the exact candidate before tag creation. ([RP-3 evidence](./davinci-v1-evidence.md#release-pr-and-promotion-gate))
- [ ] If the selected source commit is no longer the approved alpha source, stop before publication
      and dispatch a new candidate from the replacement SHA; never move an existing tag. ([RP-4 evidence](./davinci-v1-evidence.md#release-pr-and-promotion-gate))
- [ ] The workflow creates `v1.0.0-alpha.N` at the validated candidate and removes only its temporary
      `refs/release-candidates/<source commit>` ref. Do not manually create or force-move the tag. ([RP-5 evidence](./davinci-v1-evidence.md#release-pr-and-promotion-gate))
- [ ] Record the candidate SHA, its source parent, the immutable tag, and the original Release run ID
      before monitoring publication and the separately dispatched post-publish verification. ([RP-6 evidence](./davinci-v1-evidence.md#release-pr-and-promotion-gate))

## Publish Gate

- [ ] Release workflow jobs pass for: ([PU-1 evidence](./davinci-v1-evidence.md#publish-gate))
  - CLI archives and GitHub release creation
  - native npm packages
  - root npm packages
  - WASM npm package
  - crates.io publishing
  - required VS Code Marketplace publishing and exact-version visibility
  - Open VSX universal and platform publishing and exact-version visibility
- [ ] The same Release run publishes Open VSX automatically from the immutable tag. Verify the
      universal VSIX and all expected platform targets separately from Marketplace; a successful
      publisher exit without public visibility is not completion. ([PU-2 evidence](./davinci-v1-evidence.md#publish-gate))
- [ ] npm owner verifies every package is visible with the expected prerelease dist-tag: ([PU-3 evidence](./davinci-v1-evidence.md#publish-gate))

```bash
npm view vize dist-tags --json
npm view @vizejs/vite-plugin dist-tags --json
npm view @vizejs/vite-plugin-musea dist-tags --json
npm view @vizejs/wasm dist-tags --json
```

- [ ] Rust owner verifies the exact version of every crate in `publishedCrates` in
      `tools/support/release/publish_crates/main.ts`, including Canon and Patina. For example: ([PU-4 evidence](./davinci-v1-evidence.md#publish-gate))

```bash
curl --fail-with-body https://crates.io/api/v1/crates/vize_canon/VERSION
curl --fail-with-body https://crates.io/api/v1/crates/vize_patina/VERSION
```

- [ ] Editor owner verifies the VS Code marketplace page shows the new pre-release. ([PU-5 evidence](./davinci-v1-evidence.md#publish-gate))
- [ ] Release captain verifies GitHub release notes and artifacts, `prerelease: true`, and
      `make_latest: false`. ([PU-6 evidence](./davinci-v1-evidence.md#publish-gate))
- [ ] The post-publish [Release verify](../../.github/workflows/release-verify.yml) run verifies every
      npm package at the `alpha` dist-tag, normal fresh installs, config-free projects, Windows JSON
      configuration, and Musea VRT, then dispatches [JSR publication](../../.github/workflows/release-jsr.yml).
      The JSR run publishes every configured package and passes Node, Deno, and Bun fresh consumers.

## Post-Publish Gate

- [ ] Fresh install smoke passes on a clean machine or throwaway directory: ([PP-1 evidence](./davinci-v1-evidence.md#post-publish-gate))

```bash
tmp="$(mktemp -d)"
cd "$tmp"
vp dlx vize@alpha --version
vp install -D @vizejs/vite-plugin@alpha @vizejs/vite-plugin-musea@alpha
```

- [ ] Docs owner verifies the docs site, search index, and release post after
      [Deploy Docs](../../.github/workflows/deploy-docs.yml) publishes current `main`. ([PP-2 evidence](./davinci-v1-evidence.md#post-publish-gate))
- [ ] npm owner verifies native optional dependency resolution on macOS, Linux, and Windows runners. ([PP-3 evidence](./davinci-v1-evidence.md#post-publish-gate))
- [ ] Release captain posts release communication with: ([PP-4 evidence](./davinci-v1-evidence.md#post-publish-gate))
  - version and channel
  - installation commands
  - known limitations
  - rollback status and support window
- [ ] Production-readiness status is updated against [Production Readiness](./production-readiness.md). ([PP-5 evidence](./davinci-v1-evidence.md#post-publish-gate))

## Rollback Plan

Prefer a fixed alpha over destructive rollback. Use destructive actions only when a token leak,
malware risk, or severe install break requires immediate containment.

- [ ] Stop promotion by moving npm dist-tags back to the previous known-good alpha: ([RB-1 evidence](./davinci-v1-evidence.md#rollback-plan))

```bash
npm dist-tag add vize@<previous-version> alpha
npm dist-tag add @vizejs/vite-plugin@<previous-version> alpha
npm dist-tag add @vizejs/vite-plugin-musea@<previous-version> alpha
```

- [ ] Deprecate bad npm versions with an actionable message: ([RB-2 evidence](./davinci-v1-evidence.md#rollback-plan))

```bash
npm deprecate vize@<bad-version> "Do not use this alpha; upgrade to <fixed-version>."
```

- [ ] Yank bad crates.io versions when Rust consumers must not resolve them: ([RB-3 evidence](./davinci-v1-evidence.md#rollback-plan))

```bash
cargo yank --vers <bad-version> vize
```

- [ ] If GitHub artifacts are broken, retain the immutable tag and evidence, publish a fixed alpha,
      and point users at the fixed version. Do not replace assets to make an old version appear
      repaired. ([RB-4 evidence](./davinci-v1-evidence.md#rollback-plan))
- [ ] If docs are wrong, revert the docs commit or redeploy the previous known-good Pages artifact. ([RB-5 evidence](./davinci-v1-evidence.md#rollback-plan))
- [ ] If the VS Code extension is broken, publish a fixed pre-release and update the marketplace
      description. Do not unpublish without editor owner and release captain approval. ([RB-6 evidence](./davinci-v1-evidence.md#rollback-plan))

### Partial package publication recovery

Repair and read back the exact npm or crates.io publisher identity using the
[CLI recovery procedure](./trusted-publishing-recovery.md), then rerun the failed
job in the original Release workflow. Verify every expected version and GitHub
Release completion before closing the release issue or creating another tag.

### Partial editor publication recovery

- If VS Code Marketplace publication fails, repair `VSCE_PAT` in the protected
  `vscode-marketplace` environment, then rerun the failed job in the same Release workflow. Do not
  recreate or force-push the tag. The publisher skips an exact target that is already visible.
- Open VSX runs automatically in the same Release workflow. If it fails, repair `OVSX_PAT` in the
  protected `open-vsx-registry` environment and rerun only the failed job for the same immutable
  tag. Its universal and platform publishers also skip exact targets that are already visible.
- A publisher exit code is never sufficient evidence by itself. Both registry jobs stay red until
  the exact extension version is visible in the destination registry.

## Communication

- [ ] Release captain opens a tracking comment or discussion with current status: go, no-go, or
      rollback. ([CO-1 evidence](./davinci-v1-evidence.md#communication))
- [ ] Owners add verification evidence and links to the release workflow run. ([CO-2 evidence](./davinci-v1-evidence.md#communication))
- [ ] If rollback is triggered, publish the user-facing impact, affected versions, fixed version, and
      recommended action before closing the incident. ([CO-3 evidence](./davinci-v1-evidence.md#communication))
