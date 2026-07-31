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
  - [Docs Build](../../.github/workflows/build-docs.yml) evidence on the exact target commit
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
vp run --filter './npm/builder/vite-musea' test
vp run --filter './npm/builder/vite-musea' build
vp run --filter './npm/native' build:debug
```

## Tag Gate

- [ ] Release captain confirms the worktree is clean and on `main`.
- [ ] Release captain runs the release preparation script:

```bash
moon run --target native tools/moon/cmd/release -- alpha -y
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
  - required VS Code Marketplace publishing and exact-version visibility
- [ ] Open VSX is an optional channel. Publish it only when the editor owner explicitly dispatches
      [`release-open-vsx.yml`](../../.github/workflows/release-open-vsx.yml) for an existing,
      published GitHub Release tag; it is not part of the official release completion signal.
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

- [ ] Docs owner verifies the docs site, search index, and release post after
      [Deploy Docs](../../.github/workflows/deploy-docs.yml) publishes current `main`.
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

### Partial editor publication recovery

- If VS Code Marketplace publication fails, the required job prevents GitHub Release creation.
  Restore the `VSCE_PAT` secret in the protected `vscode-marketplace` environment, then rerun the
  failed jobs in the same Release workflow. Do not recreate or force-push the tag. The publisher
  skips the exact version when it is already visible, so rerunning after a partial publish is safe.
- Open VSX does not run automatically. For an existing published GitHub Release, restore `OVSX_PAT`
  in the protected `open-vsx-registry` environment and manually dispatch the optional workflow with
  that release tag. It checks out the same tag and requires exactly one attached VSIX. Re-dispatching
  is safe when the exact version is already visible.
- A publisher exit code is never sufficient evidence by itself. Both registry jobs stay red until
  the exact extension version is visible in the destination registry.

## Communication

- [ ] Release captain opens a tracking comment or discussion with current status: go, no-go, or
      rollback.
- [ ] Owners add verification evidence and links to the release workflow run.
- [ ] If rollback is triggered, publish the user-facing impact, affected versions, fixed version, and
      recommended action before closing the incident.
