# Getting Support

Vize is still pre-1.0; the surfaces that are ready for adopters and the ones
that are not are listed in
[`docs/release/production-readiness.md`](./docs/release/production-readiness.md)
and [`docs/release/stability.md`](./docs/release/stability.md). Please check
those before opening a support thread.

## Where to ask

| You want to                                                | Use                                                                    |
| ---------------------------------------------------------- | ---------------------------------------------------------------------- |
| Read documentation                                         | https://vizejs.dev                                                     |
| Try Vize without installing                                | https://vizejs.dev/play/                                               |
| Report a reproducible bug, regression, or incorrect output | [GitHub Issues](https://github.com/ubugeeei/vize/issues/new/choose)    |
| Propose a feature, integration, or workflow change         | [GitHub Issues](https://github.com/ubugeeei/vize/issues/new/choose)    |
| Discuss design, ask "how do I", share usage feedback       | [GitHub Discussions](https://github.com/ubugeeei/vize/discussions)     |
| Report a security vulnerability                            | Follow [`SECURITY.md`](./SECURITY.md) — do **not** open a public issue |
| Sponsor / fund development                                 | https://github.com/sponsors/ubugeeei                                   |

If GitHub Discussions are not enabled yet for this repository, open an issue
with the `question` label and we will move the conversation if it grows.

## What is in scope

The following are in scope for the issue tracker:

- bugs in the supported tiers listed in
  [`docs/release/production-readiness.md`](./docs/release/production-readiness.md)
- crashes, panics, or memory issues in the CLI, native bindings, or WASM
  builds
- incorrect or missing diagnostics from `vize lint`, `vize check`, or LSP
- install failures of any published npm or crates.io package
- regressions in the compiler, formatter, type checker, or Vue/Vapor runtime
  output
- documentation gaps in the official docs site or repository docs

## What is not in scope

These are not in scope for the issue tracker (the right place is in the
table above):

- general Vue, Vite, Nuxt, Rust, or Node.js questions
- design debates without a concrete proposal
- support for surfaces marked **experimental** or **incubating** in the
  stability guide (we will read these, but they may close without action)

## Response expectations

Vize is maintained on a best-effort basis by [@ubugeeei](https://github.com/ubugeeei).
We do not promise a fixed SLA outside of security reports (see
[`SECURITY.md`](./SECURITY.md)). For everything else:

- Triage usually happens within a few days.
- Bug reports with a minimal reproduction get attention soonest.
- Releases follow the schedule described in the release docs; bug fixes are
  not always backported to older prereleases.

## Commercial / priority support

There is no commercial support tier. Sponsorship through
[GitHub Sponsors](https://github.com/sponsors/ubugeeei) helps keep the project
funded but does not grant priority issue handling.
