# @vizejs/typescript-vue-plugin

Vize's opt-in plugin for the **JavaScript TypeScript language server** (tsserver,
TypeScript 5.7–6.x). It uses the same native Vue projection and feature masks as
Vize's TypeScript 7 ContentMapper. It does not start a second language server.

```sh
npm install --save-dev @vizejs/typescript-vue-plugin typescript
```

```json
{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "plugins": [{ "name": "@vizejs/typescript-vue-plugin" }]
  },
  "include": ["src"]
}
```

In VS Code, select **TypeScript: Select TypeScript Version → Use Workspace
Version**. The Vize extension stages this same package for its existing
`typescriptServerPlugins` contribution; installing the package separately is
useful for other tsserver clients. Set `"enabled": false` in the plugin entry
to disable it in that project, then restart tsserver after configuration changes.

The plugin provides authored positions for hover, completion, definition, type
definition, implementation, references and rename in native-supported `.vue`
and `.art` projections. Explicit extension imports use TypeScript's module
resolver, including configured `paths` aliases. An extensionless import does
not implicitly select a component. Native feature masks control which generated
copies participate in each operation. Edits are returned only for exact,
length-preserving verbatim spans; synthesized helpers cannot be renamed or
written into the source. The existing component-import hover remains available.
Framework-specific navigation requiring Canon-only semantic links is outside
this adapter's contract. Formatting/refactoring and other editor features are
provided by Vize's language server, not promised by this package.

Native projection failures are diagnostics, never an `any` component stub.
Missing imports remain errors. The native engine and its UTF-8/UTF-16 conversion
helper are resolved from the same installed package. The default is the plugin's own matching native dependency. A project can explicitly
select another compatible `@vizejs/native` package directory with `"nativePath"`
in its plugin entry. Overrides must supply the projection API and its matching
helper; a failed override remains an error and is never silently substituted.
Unsaved snapshots invalidate generated code and mappings together.

**Ordinary `tsc` does not load language service plugins.** This package does not
add `.vue` support to `tsc`, alter emitted JavaScript, or replace a runtime loader.
For TypeScript 7's native `tsc`/LSP use Vize's existing `vize` ContentMapper:

```json
{
  "contentMappers": [{ "package": "vize", "extensions": [".vue", ".art"] }]
}
```

That is a separate native protocol integration. Both integrations use Canon's
projection producer; their configuration and host capabilities differ.

See the official [TypeScript language service plugin guide](https://github.com/microsoft/TypeScript/wiki/Writing-a-Language-Service-Plugin)
and [ContentMapper protocol description](https://github.com/microsoft/typescript-go/pull/4712)
for the host contracts. The real tsserver packet test packs this package, installs
it into a separate consumer and retains requests, responses, diagnostics and edits.

The platform VSIX carries its own exact-release Vize native projection provider.
Its default is bound to the installed extension's package, producer token and
file hashes; workspace `nativePath` values and global native loader variables
do not replace that implementation dependency. This does not change the LSP's
existing `vize.serverPath` selection. The small universal VSIX is a separate
metadata-only artifact; full native-service acceptance uses a matching platform
VSIX. Standalone npm/tsserver consumers retain the explicit override above.
