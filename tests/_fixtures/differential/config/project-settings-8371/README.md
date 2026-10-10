# Project settings without a dedicated config (#8371)

The original inputs configure compilation, formatting, linting, and editor
features through `vite.config.mjs`, with the checker project and aliases in
`tsconfig.json`. No Vize config or imported helper is required.

`crates/vize/tests/project_config_cli.rs` runs the public native CLI and LSP
JSON-RPC entry points. It checks preserved template whitespace, custom elements,
single quotes, a configured accessibility error, project aliases, and both valid
and invalid component props and TypeScript assignments. Additional temporary
packages verify nearest-project discovery, dedicated-config compatibility,
explicit overrides, and disabled editor formatting.

The checker uses the same installed Vue types and required TypeScript native
runtime as the other CLI regressions. `VIZE_TEST_REQUIRE_TSGO=1` makes a missing
runtime fail rather than silently skip in Actions.
