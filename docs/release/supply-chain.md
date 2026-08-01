# Supply Chain

This page documents how Vize release artifacts are signed and how adopters can
verify them. It complements `SECURITY.md` (which covers vulnerability reporting)
and `docs/release/support-policy.md` (which covers compatibility promises).

## Signed artifacts

Every published GitHub Release includes, for each shipped artifact:

| File                       | Contents                                                                                |
| -------------------------- | --------------------------------------------------------------------------------------- |
| `<artifact>`               | The release asset itself (tarball, zip, or vsix).                                       |
| `<artifact>.sig`           | Detached Sigstore signature (base64).                                                   |
| `<artifact>.pem`           | X.509 certificate issued by Fulcio that attests the signing GitHub Actions workflow.    |
| `<artifact>.cosign.bundle` | Combined Sigstore bundle (Rekor inclusion proof + cert + sig) for offline verification. |

Signing uses [Sigstore cosign](https://docs.sigstore.dev/) in keyless mode. The
signing identity is the OIDC token of the GitHub Actions workflow that built
the release. No long-lived private key exists; the certificate and Rekor entry
are sufficient evidence.

## SBOMs

Two SBOMs are attached to every release:

- `vize-<tag>-cyclonedx.sbom.json` — CycloneDX 1.5 JSON, the default input for
  most SCA scanners (Snyk, Trivy, Grype) and GitHub's Dependency Graph.
- `vize-<tag>-spdx.sbom.json` — SPDX JSON for OSS-license auditing.

Both SBOMs cover the same source tree at the release commit and are signed by
cosign with the same workflow identity as the binaries.

## Verifying a release artifact

You need [`cosign`](https://docs.sigstore.dev/system_config/installation/) on
`$PATH`. The repository identity is fixed: signing only happens from the
`create-github-release` job in `.github/workflows/release.yml` on the
`ubugeeei-prod/vize` repository.

```bash
# Verify a CLI tarball
cosign verify-blob \
  --bundle vize-x86_64-unknown-linux-gnu.tar.gz.cosign.bundle \
  --certificate-identity-regexp 'https://github.com/ubugeeei-prod/vize/.+' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  vize-x86_64-unknown-linux-gnu.tar.gz
```

Same call works for the SBOM:

```bash
cosign verify-blob \
  --bundle vize-<tag>-cyclonedx.sbom.json.cosign.bundle \
  --certificate-identity-regexp 'https://github.com/ubugeeei-prod/vize/.+' \
  --certificate-oidc-issuer https://token.actions.githubusercontent.com \
  vize-<tag>-cyclonedx.sbom.json
```

A successful verification prints `Verified OK` and exits zero. Treat any
failure as evidence that the artifact has been tampered with or replaced —
do not install it.

## npm Trusted Publishing

All npm publishing in `.github/workflows/release.yml` is expected to use npm
Trusted Publishing through GitHub Actions OIDC. The release jobs run on
GitHub-hosted Ubuntu runners, request `id-token: write`, and use the `npm`
deployment environment. They intentionally do not configure
`secrets.NPM_TOKEN` or write an npm auth token fallback.

Configure each npm package's Trusted Publisher with:

- Organization or user: `ubugeeei-prod`
- Repository: `vize`
- Workflow filename: `release.yml`
- Environment name: `npm`

### First-publish bootstrap

npm only allows Trusted Publishing to be configured after a package exists. Do
not use an older failed release for recovery. Merge the bootstrap workflow,
create a fresh release tag at the current default-branch tip, and let the normal
Release workflow reach a terminal failure at the new package's OIDC publish
job. Record that Release run ID, then send the fixed `npm-bootstrap` repository
dispatch:

Freeze `main` from creation of the fresh tag until GitHub has accepted the
repository dispatch and the bootstrap run has started. Disable PR auto-merge
and allow neither direct pushes nor other merges during that window. If the
bootstrap reports that the tag and repository-dispatch SHA differ, stop and
investigate; do not create replacement tags merely to chase a moving `main`.

```bash
FRESH_TAG=vX.Y.Z
RELEASE_RUN_ID=123456789
gh api repos/ubugeeei-prod/vize/dispatches \
  -f event_type=npm-bootstrap \
  -F "client_payload[tag_name]=$FRESH_TAG" \
  -F "client_payload[release_run_id]=$RELEASE_RUN_ID" \
  -F 'client_payload[package_path]=npm/framework/nuxt-lint-config'
```

GitHub executes the workflow definition from the default branch. The bootstrap
requires the tag commit to equal that repository-dispatch SHA and to be on
`origin/main`'s first-parent history. It also binds the tag, workspace, and
package versions, and queries the supplied Release run before any credential is
available. The run must be the completed failed `.github/workflows/release.yml`
tag run for the same SHA. Its package build, release preflight, and tarball smoke
jobs must be successful, while only the target package publish job is required
to have failed. The exact package artifact from that run is downloaded,
identity-checked, smoke-installed again, and published with provenance. The
workflow refuses to run unless the package remains absent from npm.

npm Granular Access Tokens expire after at most 90 days. Before dispatch, rotate
an expired or long-lived `NPM_TOKEN` repository secret to a short-lived Granular
Access Token scoped to write packages in `@vizejs` and permitted to bypass 2FA.
The secret is exposed only to the final publish step. Never place the token in a
workflow-level or job-level environment.

Immediately after the first publish of `@vizejs/nuxt-lint-config`, configure the
normal release workflow as its trusted publisher. Use npm CLI 11.10.0 or newer
and authenticate as an npm owner with settings 2FA:

```bash
npm trust github @vizejs/nuxt-lint-config --file release.yml --repo ubugeeei-prod/vize --env npm --allow-publish --yes
```

Confirm the package's npm settings name `release.yml` and the `npm` environment.
Revoke the short-lived Granular Access Token immediately and remove or rotate
the `NPM_TOKEN` secret, then rerun only the failed jobs in the same Release run.
The standard OIDC-only publish job will see the exact version and finish its
registry verification, allowing the remaining release jobs to complete. Use
`.github/workflows/release.yml` for every later version. Do not use the bootstrap
workflow after the package exists; its registry guard will reject the request.

After every package is configured and one release has verified OIDC publishing,
set the npm package publishing access to require two-factor authentication and
disallow tokens, then revoke the old automation token.

## What is not signed

- Per-platform NAPI native packages (`@vizejs/native-*`): published to npm via
  Trusted Publishing (OIDC) so the npm registry's own provenance attestation
  applies. See the `npm` page for `@vizejs/native` for the provenance badge.
- Rust crates on crates.io: rely on crates.io's repository hosting integrity.
  Vize publishes from the workflow's OIDC identity; the crates.io UI shows the
  publish source.

Cosign signing of npm tarballs and crate tarballs is a future hardening step
once a maintained verification surface exists for those registries.

## Reporting a verification failure

If `cosign verify-blob` fails for an artifact you downloaded from
[GitHub Releases](https://github.com/ubugeeei-prod/vize/releases), follow the
disclosure process in [`SECURITY.md`](../../SECURITY.md). Do not open a public
tracker entry with the failure details until the maintainers have confirmed whether
the artifact set should be revoked.
