# Vize v1 Alpha Go/No-Go Checklist

Use this checklist for every v1 alpha release candidate before creating the release tag. The
release captain owns the final go/no-go decision; each surface owner signs off only on the section
they operate.

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

- [ ] All required PR checks are green on the release commit:
  - [Check](../../.github/workflows/check.yml)
  - [Benchmark](../../.github/workflows/benchmark.yml)
  - [App E2E](../../.github/workflows/e2e.yml) for `dev`, `preview`, and `build`
  - [Deploy Docs](../../.github/workflows/deploy-docs.yml) on the target commit or a matching dry run
- [ ] [Fuzz](../../.github/workflows/fuzz.yml) status, seeded corpus health, and uploaded
      reproducers are reviewed when parser/compiler surfaces changed.
- [ ] No release-blocking draft PR, open P0/P1 fix request, or failing required workflow remains.
- [ ] Version is agreed and matches the intended channel, for example `1.0.0-alpha.N`.
- [ ] Changelog or release post draft exists under `docs/content/blog/releases/`.
- [ ] Local smoke commands pass from a clean checkout:

```bash
vp install --frozen-lockfile
vp run --workspace-root check:ci
vp run --workspace-root test:scripts
cargo test --workspace
cargo audit --deny warnings
vp run --workspace-root build:packages
```

- [ ] Package-specific smoke checks pass when relevant:

```bash
vp run --filter './npm/vite-plugin-musea' test
vp run --filter './npm/vite-plugin-musea' build
vp run --filter './npm/vize-native' build:debug
```

## Tag Gate

- [ ] Release captain confirms the worktree is clean and on `main`.
- [ ] Release captain runs the release preparation script:

```bash
moon run --target native - -- alpha -y < tools/moon/scripts/release.mbtx
```

- [ ] The release commit is pushed to `main`.
- [ ] The `vX.Y.Z-alpha.N` tag exists on GitHub and points at the release commit.
- [ ] The [Release](../../.github/workflows/release.yml) workflow starts from the tag.

## Publish Gate

- [ ] Release workflow jobs pass for:
  - CLI archives and GitHub release creation
  - native npm packages
  - root npm packages
  - WASM npm package
  - crates.io publishing
  - VS Code marketplace publishing
- [ ] npm owner verifies every package is visible with the expected prerelease dist-tag:

```bash
npm view vize dist-tags --json
npm view @vizejs/vite-plugin dist-tags --json
npm view @vizejs/vite-plugin-musea dist-tags --json
npm view @vizejs/wasm dist-tags --json
```

- [ ] Rust owner verifies crates.io propagation without assuming `cargo install` support:

```bash
cargo search vize --limit 5
curl -sf https://crates.io/api/v1/crates/vize | jq '.crate.max_version'
```

- [ ] Editor owner verifies the VS Code marketplace page shows the new pre-release.
- [ ] Release captain verifies GitHub release notes, artifacts, and prerelease status.

## Post-Publish Gate

- [ ] Fresh install smoke passes on a clean machine or throwaway directory:

```bash
tmp="$(mktemp -d)"
cd "$tmp"
vp dlx vize@alpha --version
vp install -D @vizejs/vite-plugin@alpha @vizejs/vite-plugin-musea@alpha
```

- [ ] Docs owner verifies the docs site, search index, and release post after deploy.
- [ ] npm owner verifies native optional dependency resolution on macOS, Linux, and Windows runners.
- [ ] Release captain posts release communication with:
  - version and channel
  - installation commands
  - known limitations
  - rollback status and support window
- [ ] Production-readiness status is updated against [Production Readiness](./production-readiness.md).

## Rollback Plan

Prefer a fixed alpha over destructive rollback. Use destructive actions only when a token leak,
malware risk, or severe install break requires immediate containment.

- [ ] Stop promotion by moving npm dist-tags back to the previous known-good alpha:

```bash
npm dist-tag add vize@<previous-version> alpha
npm dist-tag add @vizejs/vite-plugin@<previous-version> alpha
npm dist-tag add @vizejs/vite-plugin-musea@<previous-version> alpha
```

- [ ] Deprecate bad npm versions with an actionable message:

```bash
npm deprecate vize@<bad-version> "Do not use this alpha; upgrade to <fixed-version>."
```

- [ ] Yank bad crates.io versions when Rust consumers must not resolve them:

```bash
cargo yank --vers <bad-version> vize
```

- [ ] If GitHub artifacts are broken, mark the release as draft or delete only the affected assets,
      then rerun the release workflow from a fixed tag.
- [ ] If docs are wrong, revert the docs commit or redeploy the previous known-good Pages artifact.
- [ ] If the VS Code extension is broken, publish a fixed pre-release and update the marketplace
      description. Do not unpublish without editor owner and release captain approval.

## Communication

- [ ] Release captain opens a tracking comment or discussion with current status: go, no-go, or
      rollback.
- [ ] Owners add verification evidence and links to the release workflow run.
- [ ] If rollback is triggered, publish the user-facing impact, affected versions, fixed version, and
      recommended action before closing the incident.
