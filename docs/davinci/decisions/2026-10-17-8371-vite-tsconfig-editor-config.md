# Project configuration through Vite and TypeScript

Issue: [#8371](https://github.com/ubugeeei-prod/vize/issues/8371).
Delivery: [#9281](https://github.com/ubugeeei-prod/vize/pull/9281), followed by
[#9284](https://github.com/ubugeeei-prod/vize/pull/9284).

This decision supersedes the dedicated-file and legacy-namespace behavior in
the October 12 canonical configuration decision. The maintainer requested
existing project configuration, the `vue:` namespace and the fluent
`defineConfig(vpConfig).withVue(vueConfig)` form. A separate Vize configuration
file or a Vize-specific configuration inheritance system must not be required.

## Public configuration

`vite.config.*` owns the shared native `vue` settings. The fluent helper accepts
an existing Vite+ object, Promise, factory or async factory and preserves its
environment, plugins, hooks and tasks. Evaluation and plugin injection occur
once. Ordinary JavaScript imports provide reusable settings.

`tsconfig.json` owns TypeScript `compilerOptions` and supported
`vueCompilerOptions`. Standard TypeScript JSONC, `extends`, package inheritance
and `references` remain supported. An explicitly authored Vite Vue version
takes precedence over the TypeScript Vue target. The reader guide lists the
supported options and remaining limitations; accepting a field must not imply
an unimplemented compiler behavior.

Dedicated `vize.config.*` discovery, the `vize:` namespace and custom Vize
`extends` are retired with explicit migration diagnostics. A stale dedicated
file cannot override a valid selected Vite configuration. Explicit internal
native serialization controls remain separate from project discovery.

## Editor ownership and reload

The native language server owns project configuration for VS Code, Zed,
Neovim, Vim, Helix and Emacs. Adapter defaults must not silently replace the
selected project's settings. Explicit editor feature overrides remain explicit.

Each document belongs to its Vite root and effective TypeScript project.
Reference projects sharing one directory retain independent Vue targets and
applied settings. Immutable snapshots retain both the authored policy and its
applied result; a failed reload preserves the last valid snapshot. Initially
invalid configuration refuses the affected operation rather than pretending
that default settings were authored.

Watch dependencies include selected Vite sources, imported helpers and
inherited TypeScript configuration. The nearest present Vite installation is
authoritative; an invalid nearer installation must not fall through to an
unrelated outer installation. Watch events under the private
`node_modules/.vize` output directory are excluded before routing or native
request generations change, including events through symlink aliases.

Scoped entries currently admit only `basePath`, `files`, `ignores` and
`linter`. Additional scoped native sections and per-file editor scope effects
remain unfinished. This migration does not claim those features.

## Qualification

Historical fixtures retain their original inputs, full output expectations
and authority hashes. Active canonical carriers use explicit finite source
transitions. Rehashing an altered oracle or filtering actual diagnostics is
not evidence of compatibility.

[Editor conformance 38988521161](https://github.com/ubugeeei-prod/vize/actions/runs/38988521161)
passed at `1ac59318070c89ce283f3bfc6002c3474db097df`: four TS-45 adapter cases
and genuine Vim/Emacs configuration cases. The Emacs packet retains all seven
cases, 143 raw entries, shutdown acknowledgment, exit notification and observed
native PID termination. Its unobserved exit status is recorded as `null`.
The Zed case is protocol/adapter conformance, not a GUI session. These receipts
do not qualify a different source revision or an installed public release.

Fresh combined native/tooling checks, actual protected merges and installed
public acceptance remain required before closing the issue. The broader VS
Code authoring oracle also remains required independently of TS-45.
