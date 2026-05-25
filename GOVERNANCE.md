# Governance

This document describes how Vize is maintained. It is intentionally lightweight
for the v1 alpha phase and will be revisited before v1 GA.

## Roles

### Maintainer

A maintainer can merge pull requests, cut releases, and triage issues across
the workspace. The current maintainer is listed in `package.json` and
`Cargo.toml`. Today the project is led by [@ubugeeei](https://github.com/ubugeeei).

### Surface owner

Some areas of the codebase have a designated owner who signs off on changes
that affect that surface during a release. The owners are listed in
[`docs/release/v1-alpha-go-no-go.md`](./docs/release/v1-alpha-go-no-go.md).

### Contributor

Anyone who opens an issue, discussion, or pull request. Contributions of any
size are welcome; see [`CONTRIBUTING.md`](./CONTRIBUTING.md) for how to set up
the workspace.

## Decision making

The project uses a lazy-consensus model:

1. Routine changes — bug fixes, dependency bumps, documentation edits — land
   once a maintainer approves the pull request and CI is green.
2. Behavioral changes to public surfaces (CLI flags, `vize.config.*` schema,
   Patina rule semantics, type-checker diagnostics, npm package exports,
   public Rust crate APIs) require:
   - an issue or RFC-style discussion before the PR,
   - sign-off from the surface owner listed in the release checklist,
   - a release note that mentions the change under the appropriate SemVer
     level (see [`docs/content/stability.md`](./docs/content/stability.md)).
3. Releases follow [`docs/release/v1-alpha-go-no-go.md`](./docs/release/v1-alpha-go-no-go.md).
   The release captain has final go/no-go authority.
4. Security issues follow [`SECURITY.md`](./SECURITY.md) and are handled
   privately until a fix is available.

When a decision is contested, the maintainer makes the final call. A future
governance revision may introduce a steering committee once the contributor
base grows.

## Becoming a maintainer

There is no fixed timeline. Sustained, high-quality contributions over
multiple releases, paired with willingness to take on release or surface
ownership, are the practical signals. Open an issue or reach the maintainer
through the channel listed in [`SUPPORT.md`](./SUPPORT.md) if you would like
to discuss it.

## Code of conduct

Vize adopts the [Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).
Enforcement contact is the same as the security reporting channel described in
[`SECURITY.md`](./SECURITY.md).

## Changes to this document

Changes to governance require a pull request and a maintainer's approval.
Substantive changes should be announced in a release note.
