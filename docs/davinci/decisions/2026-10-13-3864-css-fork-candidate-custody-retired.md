# CSS fork candidate custody is retired

Issue: [#3864](https://github.com/ubugeeei-prod/vize/issues/3864).

## Decision

The qualification apparatus built for the CSS engine fork candidate is
removed now that the forks are part of `main`
([#8709](https://github.com/ubugeeei-prod/vize/pull/8709)). This supersedes
the custody and evidence requirements of the
[candidate record](./2026-10-12-css-engine-vendor-exit.md); its description of
the two engine repairs and of the source-map migration stays accurate.

Removed:

- the tooling test that required the live root `Cargo.toml` and `Cargo.lock`
  to equal a frozen 0.443.0 baseline apart from the release version rewrite;
- the two tests that pinned the release driver sources by SHA-256;
- the custody, observation and downstream-identity jobs of
  `css-engine-vendor-candidate.yml`, their helper scripts under
  `tools/support/dependencies/`, and the fixtures only they read.

Kept:

- the fork test suites. The workflow still runs
  `cargo test -p vize_lightningcss --all-features` and
  `cargo test -p vize_parcel_sourcemap --features json` in the `dev` and
  `release` profiles when the forks or `Cargo.lock` change, and the workspace
  test archive builds both forks on every pull request;
- the panic-boundary regression tests in Atelier, Glyph and Patina;
- the release rewrite tests, which run against fixture copies.

## Reason

The manifest test failed for any edit to the root manifests other than a
version sweep. Raising `rust-version` by one line failed it, and so would
every dependency update. The driver pins fail whenever the release tooling is
edited. Neither test observes product behavior: they assert that files which
ordinary pull requests must edit have not changed.

Tests in this repository verify product behavior or immutable historical
facts. A check that fixes the bytes of files under active development is
merge friction, and the candidate it qualified has already merged.

## Tradeoff

A pull request can now change the fork manifests without reproducing the
candidate's before/after observations. The fork suites and the boundary tests
are the regression gate for such a change.
