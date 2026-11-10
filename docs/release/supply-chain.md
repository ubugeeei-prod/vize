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

For an existing package with a broken or missing publisher, follow
[Trusted Publishing Recovery](./trusted-publishing-recovery.md). Both npm and
crates.io settings can be repaired from a CLI using authenticated owner access.

### First-publish bootstrap

npm only allows Trusted Publishing to be configured after a package exists. Do
not use an older failed release for recovery. Before cutting the release, add the
new package's exact repository path and npm name to the reviewed allowlist in
`tools/support/release/npm_bootstrap/policy.ts` and to the matching workflow
choice. The bootstrap workflow and allowlist must already be on `main` when the
normal Release workflow starts.

Let the Release workflow build and smoke the candidate, create its immutable
tag, and reach a terminal failure in `Publish npm packages` because the new
package has no Trusted Publisher yet. Record the exact failed Release run ID.
Run the handoff workflow from `main`; do not rerun it from a branch or create a
replacement tag:

```bash
VERSION=X.Y.Z
FRESH_TAG=v$VERSION
RELEASE_RUN_ID=123456789
PACKAGE_PATH=npm/path/already-added-to-the-bootstrap-allowlist
gh workflow run release-npm-bootstrap.yml --ref main \
  -f version="$VERSION" \
  -f tag_name="$FRESH_TAG" \
  -f release_run_id="$RELEASE_RUN_ID" \
  -f package_path="$PACKAGE_PATH"
```

The policy requires a strict version and matching `vX.Y.Z` tag. That tag must
resolve to a commit with exactly one parent, and that parent must be the exact
`main` head of the supplied completed, failed `.github/workflows/release.yml`
run. Candidate preparation, the single `npm-packages` build, tarball smoke, tag
creation, and post-build publication admission must have succeeded; the npm
publisher must have failed. The workflow selects the one live `npm-packages`
artifact by its numeric ID, records its GitHub SHA-256 digest, and downloads it
into an otherwise empty staging tree.

The selected package path, manifest name, tagged workspace/package version,
and public access must match the allowlist. npm must return exactly HTTP 404 for
the package; an existing package or an ambiguous registry failure refuses the
handoff. The existing release manifest normalizer resolves only the candidate
artifact's workspace and catalog protocols. The workflow then runs `npm pack`
twice with lifecycle scripts disabled, requires byte-for-byte equality and npm's
SHA-512 integrity, and uploads this credential-free handoff artifact:

```text
npm-cli-first-publish-<scope>-<package>-X.Y.Z/
├── SHA512SUMS
├── npm-publish-handoff.json
└── <scope>-<package>-X.Y.Z.tgz
```

The workflow has only `actions: read` and `contents: read`. It has no deployment
environment, OIDC permission, npm secret, publication command, provenance flag,
tag writer, or dispatch writer. Download the exact artifact named in its job
summary, inspect `npm-publish-handoff.json`, verify `SHA512SUMS`, authenticate
locally as an npm owner with settings 2FA, and make the one-time first publish
from that tarball:

```bash
shasum -a 512 --check SHA512SUMS
npm whoami
npm publish ./<scope>-<package>-X.Y.Z.tgz --access public
```

This local CLI first publish does not have GitHub Actions OIDC provenance. That
is expected only for the package-creation bootstrap; do not pass
`--provenance`, and do not describe this run as Trusted Publishing.

Immediately after the first publish, configure the normal release workflow as
the package's trusted publisher. Use npm CLI 11.17.0 or newer and authenticate
as an npm owner with settings 2FA:

```bash
PACKAGE_NAME=@vizejs/new-package
npm trust github "$PACKAGE_NAME" --file release.yml --repo ubugeeei-prod/vize --env npm --allow-publish --yes
```

Confirm the package's npm settings name `release.yml` and the `npm` environment,
then rerun only the failed jobs in the same Release run. The standard OIDC-only
publish job will see the exact version and finish its registry verification,
allowing the remaining release jobs to complete. Use
`.github/workflows/release.yml` for every later version. Do not use the bootstrap
workflow after the package exists; its registry guard will reject the request.

After every package is configured and one release has verified OIDC publishing,
set the npm package publishing access to require two-factor authentication and
disallow tokens. Do not create an npm publishing secret for the Release workflow;
remove any obsolete npm publishing secret that predates the local handoff.

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
disclosure process in [`SECURITY.md`](../../.github/SECURITY.md). Do not open a public
tracker entry with the failure details until the maintainers have confirmed whether
the artifact set should be revoked.
