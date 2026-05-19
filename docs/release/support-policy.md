# Support Policy

This page is the canonical answer to "for how long, and under what conditions, will Vize keep
something working?" It complements the broader [stability tiers](../content/stability.md) and the
[production-readiness checklist](./production-readiness.md). When this page disagrees with marketing
copy or a blog post, this page wins.

## Scope of this policy

This policy applies to all surfaces in the `alpha-supported` and `preview` tiers as listed on the
stability page. `experimental` and `incubating` surfaces are explicitly out of scope: they may
change or disappear in any release.

In-scope surfaces:

- **CLI**: `vize` binary, its subcommands, and their documented flags.
- **Config**: `vize.config.*` keys and their value shapes.
- **Public Rust crates**: items reachable through a published crate's `pub` API.
- **Public npm packages**: items in each package's exported entrypoints.
- **Patina lint rules**: rule names, default severities, and message ids.
- **Type-checker diagnostics**: error codes listed in the docs.

## SemVer mapping

While Vize is in `0.x` alpha:

| Change                                                         | Treated as     |
| -------------------------------------------------------------- | -------------- |
| Remove or rename a CLI subcommand, flag, or env var            | Breaking       |
| Remove or rename a config key                                  | Breaking       |
| Tighten a config value's accepted shape                        | Breaking       |
| Remove a `pub` item from a published Rust crate                | Breaking       |
| Remove or rename an export from a published npm package        | Breaking       |
| Promote a Patina rule from `warn` to `error` by default        | Breaking       |
| Demote a Patina rule from `error` to `warn` by default         | Non-breaking   |
| Add a new diagnostic code                                      | Non-breaking   |
| Change a diagnostic message string (code unchanged)            | Non-breaking   |
| Change template parser output for previously rejected programs | Non-breaking   |
| Change template parser output for previously accepted programs | Breaking       |
| Change formatter output for the same input                     | Documented[^1] |
| Change generated `virtualTs` shape for type-checking           | Documented[^1] |

[^1]:
    Documented in release notes; not a breaking change for SemVer purposes but adopters should
    treat formatter and `virtualTs` drift as a review-time signal.

Once Vize reaches v1 stable, the same table applies with `0.x` replaced by SemVer-major rules.

## Deprecation windows

Vize keeps deprecated surfaces working for a minimum window before removal:

| Surface                | Minimum deprecation window before removal |
| ---------------------- | ----------------------------------------- |
| Public Rust crate item | One minor release with a `#[deprecated]`  |
| npm package entrypoint | One minor release with a console warning  |
| CLI flag or subcommand | One minor release with a stderr warning   |
| Config key             | One minor release with a stderr warning   |
| Patina rule (name)     | One minor release with a stderr warning   |
| Diagnostic code (id)   | One minor release with a release note     |

A "minor release" means one published version with `minor` or higher SemVer bump; weekly patches
do not count. When two deprecations are linked, both removal points must satisfy this window.

Every removal must appear in the release notes for the version that performs it, with a one-line
"removed since X.Y" entry that includes a link to the migration guidance.

## Emergency breaking changes

A breaking change may bypass the deprecation window only when **all** of:

- It is required to address a security vulnerability that has no non-breaking remediation, **or**
  required to fix data loss in a published surface.
- The release notes for the emergency release call out the breakage explicitly.
- A migration note is published in the same release.

Emergency breakage is rare. Most issues should be addressed by a forward-compatible patch and a
proper deprecation window for the long-term fix.

## Release line support

Vize publishes a single supported release line at a time:

- The **current minor** receives bug fixes, security backports, and accepted feature work.
- The **previous minor** receives security backports for the duration of one calendar month after
  the next minor ships, then is end-of-life.
- Older minors are end-of-life and do not receive backports of any kind.

Once Vize reaches v1 stable, this policy will be revisited to declare whether Vize will run an LTS
program. Until then, **Vize does not commit to an LTS release line**.

## Security backports

Security advisories are handled through `SECURITY.md`. Once a fix lands on the current minor, the
release captain decides whether to backport to the previous minor based on severity and effort. The
decision is recorded in the security advisory.

## How this policy changes

Changes to this policy ship as a regular pull request and follow the same review process as code
changes. A change that tightens guarantees may apply retroactively; a change that loosens
guarantees must be announced in release notes for the next minor before taking effect.
