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
