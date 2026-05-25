# Contributing

Thanks for helping make Vize sharper. This project is still moving toward v1 alpha, so small, focused changes with clear verification are the easiest to review.

## Setup

Use the Node.js version from `.node-version` and the Rust version from `rust-toolchain.toml`. The workspace declares a minimum supported Rust version (MSRV) of `1.95.0` in `Cargo.toml` (`[workspace.package].rust-version`); contributions must compile under that version.

Install dependencies from the workspace root:

```sh
vp install --frozen-lockfile --prefer-offline
```

If `vp` is not available yet, install the package manager from `package.json` and use the workspace scripts through the local toolchain.

## Common Checks

Run the narrowest check that covers your change, then broaden when you touch shared behavior.

```sh
vp check <changed-files>
node --test tests/tooling/<test-file>.test.ts
cargo fmt --all -- --check
cargo test -p <crate>
```

Before opening a PR that changes shared tooling, release automation, native bindings, or compiler behavior, run the relevant workspace task from CI locally when practical.

For GitHub Actions changes, use `actrun` to lint or preview the workflow graph before pushing:

```sh
actrun lint .github/workflows/check.yml
actrun workflow run .github/workflows/check.yml --dry-run
```

When a focused job is practical on your machine, run it locally before opening the PR:

```sh
actrun workflow run .github/workflows/check.yml --job check-js
```

## Language Processor Change Discipline

Vize follows compiler-project practice from rustc, TypeScript, TypeScript-Go, and Flow: classify the
change, add the smallest meaningful fixture, review generated output as a contract, then broaden to
parity, performance, or release gates when the touched surface needs it. See
[Language Engineering Practices](./docs/content/architecture/language-engineering-practices.md) for
the full matrix.

Use one of these change classes in PRs when applicable:

- Parser or AST
- Compiler and codegen
- Semantic analysis, lint, and cross-file analysis
- Virtual TypeScript and type checking
- Formatter and LSP
- Runtime packaging, release, or docs

For language-facing changes, include the fixture or snapshot diff that proves the behavior. For
snapshot refreshes, explain why the new output is correct and avoid broad baseline churn unless the
PR is specifically about that output family.

## Pull Requests

- Use Conventional Commits for commit messages and PR titles, such as `fix(vite-plugin): surface SFC compile errors`.
- Keep PRs focused on one behavioral change or one documentation/governance change.
- Include verification commands in the PR body.
- Do not refresh large snapshot baselines unless the PR is specifically about those outputs.
- Do not include secrets, registry tokens, private vulnerability details, or machine-local paths in issues, commits, or PRs.

## Issues

Use the bug report template for regressions, crashes, incorrect diagnostics, package installation problems, and release failures. Use the feature request template for new integrations, API changes, or workflow improvements.

Security issues should follow `SECURITY.md` instead of the public issue templates.

## Code of Conduct and Governance

By participating, you agree to abide by the [Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/).
The governance model and decision-making process are documented in [`GOVERNANCE.md`](./GOVERNANCE.md).
For help finding the right channel, see [`SUPPORT.md`](./SUPPORT.md).
