# Roadmap: graduate the experimental tsrs checker backend

Vize now has an explicit experimental path for running `tsrs` as its native
TypeScript checker. This discussion would track the evidence needed before it
can become a supported alternative to the bundled TypeScript/Corsa runtime.

## What works today

The compatibility probe used `tsrs@0.18.0-ts7.1.0-dev.20260929` on macOS arm64.
Its native CLI accepted the flags Vize's batch executor relies on:
`--checkers`, `--incremental`, `--tsBuildInfoFile`, `--pretty false`, and
`--project`. A published `vize@0.504.0` check of a minimal Vue SFC reported an
authored script error through tsrs with the expected TS2322 code and non-zero
exit status.

tsrs also documents the same `--lsp -stdio` protocol as tsgo and an unstable
TypeScript 7 API protocol. That makes it a plausible fit for Vize's batch,
type-aware lint, and editor paths without a second virtual-code implementation.

## Gaps observed in the current release

- tsrs targets a TypeScript 7.1 development snapshot while Vize's bundled
  runtime is currently TypeScript 7.0. Projects need a pinned backend version
  and a diagnostic comparison before changing CI.
- Upstream reports 4,074 of 4,546 language-service fourslash groups passing.
  Content-mapper callbacks are not implemented in the tsrs language server.
- The Node API is explicitly unstable. Vize should keep the executable protocol
  boundary and avoid taking a package API dependency until it stabilizes.
- `--init` and `--showConfig` are unsupported. They are not needed by Vize's
  runtime adapter, but they prevent a blanket claim of full `tsc` CLI parity.
- Version 0.18.0 publishes macOS arm64 and Linux native packages, with no Windows
  or Intel macOS package.

## Proposed graduation gates

1. Run Vize's authored Vue diagnostic fixtures with pinned tsrs and the default
   runtime, recording intended diagnostic differences by TypeScript version.
2. Exercise editor diagnostics, hover, completion, navigation, rename, and code
   actions against the same virtual Vue project on both runtimes.
3. Cover type-aware lint sessions, incremental edits, project references,
   declaration emit, and multi-server sharding.
4. Add packaged consumer smoke tests on each platform tsrs publishes and report
   an actionable unsupported-platform message everywhere else.
5. Define the supported tsrs version range and update policy before removing the
   experimental label.

The first integration remains opt-in through `typeChecker.tsrsPath`,
`--tsrs-path`, or `TSRS_PATH`. The existing backend stays the default while
these gates are open.

## Sources

- [tsrs 0.18.0 release notes](https://github.com/oxc-project/tsrs/blob/main/docs/releases/0.18.0.md)
- [CLI usage](https://github.com/oxc-project/tsrs/blob/main/docs/USAGE.md)
- [language server status](https://github.com/oxc-project/tsrs/blob/main/docs/LSP.md)
- [unstable Node API](https://github.com/oxc-project/tsrs/blob/main/docs/NODE_API.md)
- [compatibility matrix](https://github.com/oxc-project/tsrs/blob/main/docs/COMPATIBILITY.md)
