# Corsa first publication

The six new SDK names need one authenticated initial publication before ordinary
crates.io Trusted Publishing can take over. An existing name, an HTTP 200, or a
same-version `cargo info` result does not prove ownership, source identity, or trust.

This source prepares a protected carrier. Adding the workflow does **not** authorize
creating its secret, dispatching it, publishing a namespace, or changing publisher
configuration. Review the actual future H, package/source provenance, direct owner,
credential scope and finite publication plan before those irreversible operations.
Historical releases must never supply a replacement first-publication source.

## Before a new Release run

Both pinned and legacy dispatch paths return an already-authenticated R before the
new gate. Existing R resumes preserve their original identity and evidence.
An H without a Corsa catalogue keeps its existing path. An H declaring Corsa must
contain the gate, exact six-name catalogue and six workspace-version packages.

Without authenticated completion evidence the gate earns full H Check, appends
one H/C/source-PR/tag-bound pause marker to its own source receipt, and fails before
NEW-R or queue admission. It preserves the original body. Even six names appearing
after an earlier 404 cannot bypass this pause. A changed H/C cannot inherit it.
The child inherits the original operator deadline; the pause grants no extra budget.

## One protected carrier

Use `Release Corsa first publication` on the exact frozen `release/vTAG` ref, with
the paused source PR, H, C, future tag, actual full Check run, intended crates.io user
ID, and literal `first-publication:H:vTAG` confirmation. The existing `crates-io`
environment protects its one credential, `CORSA_BOOTSTRAP_TOKEN`.
The approved short-lived token must be limited to `publish-new` and
`trusted-publishing` for exactly these names, in this order:

1. `vize_corsa_core`
2. `vize_corsa_jsonrpc`
3. `vize_corsa_client`
4. `vize_corsa_lsp`
5. `vize_corsa_orchestrator`
6. `vize_corsa`

Caller-provided user ID is insufficient: live canonical `vize` direct ownership
must match the original/current triggering GitHub maintainer and source author.
The authenticated by-user configuration endpoint then binds the token to that
actual user ID. Every partial/new SDK namespace must retain that direct owner.
Team membership alone is insufficient to create publisher configuration.

Before any write, all existing partial names must have this exact version, staged
H archive checksum, and authenticated direct ownership. Each selected publication
uses H's existing publisher, then crosses exact-version API, complete downloaded
archive/hash/source, Cargo registry resolution and actual server publisher barriers
before the next dependent name. Unknown inventories and foreign namespaces fail.
The server must return owner/repository/workflow/environment exactly
`ubugeeei-prod/vize`, `release.yml`, `crates-io`, including the real owner ID.

H's existing registry consumer runs in `--sdk-only` mode with its original six SDK
default/distributed fixture, graph and whole-archive laws. It explicitly does not
qualify Canon. Default seven-crate acceptance remains mandatory after normal public
release. Partial archives and original failures remain separate custody artifacts.
The single-member handoff ZIP has an independently checked GitHub digest and binds
run/attempt, H/C, source PR, qualification, actual owner and six server records.

The carrier has fifteen minutes including setup/queue time. It never extends a
failed attempt. A new explicitly authorized same-H carrier may inspect and resume
partial publication; original failed runs and archives remain historical evidence.
Revoke the short-lived credential after successful handoff, and remove its secret.
This is an explicit website/credential-owner operation. At the pinned server
revision, `GET /me/tokens/{id}` and `DELETE /tokens/current` use the default auth
policy, which rejects scoped tokens. Do not broaden the credential to call them,
invent token self-introspection, or report revocation from a YAML declaration.

## Resume and later ordinary releases

Resume the same paused source. It reauthenticates the successful same-H carrier and
whole ZIP before creating ordinary R. Normal OIDC publication and full public
acceptance still run. No manual-array waiver or release-success flag is introduced.

After actual official R success and signed main integration M, retain the original
carrier receipt in an ordinary reviewed PR. From a checkout retaining that policy:

```sh
node tools/support/release/corsa_bootstrap/main.ts --prepare-checkpoint \
  /absolute/path/to/actual-carrier-receipt.json ACTUAL_M ACTUAL_R \
  tools/support/release/corsa_bootstrap/completed.json
```

This command authenticates the real carrier and official R, signed M, entire H/M
tree equality, owner and closed policy hashes before writing a fresh file. No
placeholder checkpoint ships with this preparation. Later NEW-R checks require M
on C's history, preserved source policy and live canonical/six-name direct ownership;
they need no persistent bootstrap credential. Changed policy or missing historical
authority fails closed for explicit review. Actions artifact expiry cannot erase
the committed original receipt. Keep original R/public/failed-run receipts intact.

## Primary API contract

The [official Rust announcement](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/)
requires manual initial publication before configuring Trusted Publishing.
Implementation follows immutable crates.io source
[`33e1ca9`](https://github.com/rust-lang/crates.io/tree/33e1ca9abb94e17779d580bd37516c6db1bcd1ab):
[`krate/owners.rs`](https://github.com/rust-lang/crates.io/blob/33e1ca9abb94e17779d580bd37516c6db1bcd1ab/src/controllers/krate/owners.rs),
[`github_configs/list.rs`](https://github.com/rust-lang/crates.io/blob/33e1ca9abb94e17779d580bd37516c6db1bcd1ab/src/controllers/trustpub/github_configs/list.rs),
[`create.rs`](https://github.com/rust-lang/crates.io/blob/33e1ca9abb94e17779d580bd37516c6db1bcd1ab/src/controllers/trustpub/github_configs/create.rs), and
[`trustpub.rs`](https://github.com/rust-lang/crates.io/blob/33e1ca9abb94e17779d580bd37516c6db1bcd1ab/crates/crates_io_api_types/src/trustpub.rs).
Creation is `POST` with a nested `github_config`; list uses either `crate` or
`user_id` with seek pagination. The existing `/me` endpoint is cookie-only and
cannot authenticate this API token. These are server contracts, not guessed paths.
