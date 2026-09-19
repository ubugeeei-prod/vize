# Trusted Publishing Recovery

Repair the publisher identity before rerunning a failed release. Keep the
original tag and artifacts: publication is idempotent, and a successful job
must still verify the exact version in the public registry. First publication
of an npm package follows the separate [bootstrap procedure](./supply-chain.md#first-publish-bootstrap).

## npm

Use an npm CLI with `npm trust` support (11.15.0 or newer), authenticated as a
package owner with settings 2FA. Read the existing configuration first:

```bash
npm trust list @vizejs/native-linux-arm64-musl --json
```

If the release publisher is missing, configure the exact workflow identity:

```bash
npm trust github @vizejs/native-linux-arm64-musl \
  --file release.yml --repo ubugeeei-prod/vize --env npm --allow-publish --yes
npm trust list @vizejs/native-linux-arm64-musl --json
```

Repeat for the other affected packages, including every platform package and
its parent package. Preserve unrelated publishers. An `EOTP` response requires
interactive 2FA; it does not mean the CLI cannot manage Trusted Publishing.
If a CLI authentication URL expires, start a fresh authentication attempt.
Never add a long-lived npm token to the release workflow as a recovery shortcut.

The [npm trust reference](https://docs.npmjs.com/cli/v11/commands/npm-trust/)
documents the current authentication and publisher-management options.

## crates.io

Audit every crate in `published_crates` in
`tools/moon/cmd/publish_crates/main.mbt`, including dependencies that the failed
job never reached. All publishable workspace crates belong in that automatic
plan, in dependency order. The plan tests compare it against Cargo metadata.

An existing crates.io owner token with Trusted Publishing management permission
can use the management API from a CLI. Send it in the `Authorization` header,
with a descriptive `User-Agent`, without printing or committing the token:

- Read: `GET https://crates.io/api/v1/trusted_publishing/github_configs?crate=CRATE_NAME`
- Add a missing publisher: `POST https://crates.io/api/v1/trusted_publishing/github_configs`

The POST JSON body is:

```json
{
  "github_config": {
    "crate": "CRATE_NAME",
    "repository_owner": "ubugeeei-prod",
    "repository_name": "vize",
    "workflow_filename": "release.yml",
    "environment": "crates-io"
  }
}
```

Read the configuration back and compare every identity field. Leave existing
unrelated configurations intact. Management credentials stay on the operator's
machine; the release job continues to request its short-lived OIDC token.

## Rerun and verify

```bash
gh run view RELEASE_RUN_ID --repo ubugeeei-prod/vize
gh run rerun RELEASE_RUN_ID --job FAILED_JOB_ID --repo ubugeeei-prod/vize
```

Wait for terminal job status, then query each expected package and exact version
from its public registry. For example:

```bash
npm view @vizejs/native@VERSION version --json
curl --fail-with-body https://crates.io/api/v1/crates/vize_canon/VERSION
gh release view TAG --repo ubugeeei-prod/vize
```

Record the verified version list and workflow links in the release issue.
Successful publisher registration, a successful crates job, and a completed
GitHub Release are separate checks. Keep the issue open while any required
channel is incomplete. The legacy JSX/Patina handoff dispatch remains available
for older tags; new tags publish these crates through the automatic plan.
