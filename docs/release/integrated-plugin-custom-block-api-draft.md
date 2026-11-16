# Integrated custom-block API draft

> [!WARNING]
> This is a companion design draft for the experimental integrated plugin API.
> It does not change the v1 custom-block behavior or release scope.

Custom blocks must behave like first-class source documents across parsing,
runtime generation, type checking, linting, formatting, HMR, LSP, and editors.
They are independently negotiated as `sfc.custom-block`; they are not opaque
bundler payloads or merely another preprocessor.

The parent [integrated plugin API draft](./integrated-plugin-api-draft.md)
defines configuration, backend selection, versioning, security, and global
performance rules. This document defines the custom-block contract shared by
native, JavaScript, and optional Wasm implementations.

## Compatibility baseline

Vize already preserves basic custom-block ranges and has useful product
integrations for `route`, `i18n`, and `art`, but the implementations do not yet
share one semantic artifact:

- `SfcCustomBlock` carries type, content, a block range, and an attribute map,
  but not authored attribute order and ranges, normalized `src` and `lang`,
  dependencies, or a block source map;
- the Vite path exposes raw `?vue&type=<type>&index=N` modules, returns no map,
  and conservatively full-reloads custom-block changes;
- Musea, Canon, Patina, Glyph, and Maestro independently parse overlapping
  subsets of `route`, `i18n`, and `art`; and
- formatting, type checking, linting, and editor support admit different block
  subsets.

The migration adds a shared artifact beside these paths before replacing any
of them. Existing output, diagnostics, ranges, edits, and HMR are compatibility
oracles. `art` is the first differential canary because it crosses the most
products. `route` and `i18n` can share analysis before runtime ownership moves.
An unclaimed block retains the current raw-module and reload behavior.

## Registration and ownership

```ts
api.sfc.block("route", {
  languages: ["json", "json5", "ts"],
  externalSource: true,
  parse: "route-document/v1",
  runtime: { module: "virtual:acme/routes" },
  typecheck: { virtualDocument: "typescript" },
  lint: { ruleset: "route" },
  format: { language: "route" },
  languageServer: {
    completion: true,
    hover: true,
    definition: true,
    references: true,
    rename: true,
    semanticTokens: true,
  },
  hmr: true,
});
```

Ownership is exclusive for `(block type, optional language)`. Registration
fails before source processing when two plugins claim the same key. Authored
config order never resolves an ownership conflict.

Multiple observers may consume public facts, dependencies, and diagnostics.
Observers are read-only: they cannot replace the parsed artifact, emit runtime
modules, or produce edits. The owning provider controls only the claimed block,
not SFC boundaries, attributes owned by core, or another embedded document.

Unknown blocks follow an explicit project policy: preserve, warn, or error.
Under the v2 opt-in, a registration may remove only the matching unknown-block
warning. The v1 rule and default behavior remain unchanged.

## Source input

The core SFC parser continues to own tags and boundaries. It additively exposes
the following provider input:

```ts
interface CustomBlockInput {
  id: CustomBlockId;
  owner: SourceIdentity;
  type: AuthoredToken;
  lang?: AuthoredToken;
  src?: AuthoredToken;
  attributes: readonly AuthoredAttribute[];
  openingTagRange: ByteRange;
  contentRange: ByteRange;
  closingTagRange?: ByteRange;
  blockRange: ByteRange;
  selfClosing: boolean;
  source: InlineSource | ExternalSource;
  contentHash: ContentHash;
}
```

`id` is stable for owner file, block type, and occurrence. The content hash is
separate so an edit does not change document identity. Authored attributes keep
order, name and value ranges, quoting, and duplicate-attribute diagnostics.
Normalized `src` and `lang` values never replace authored tokens.

Internal offsets are UTF-8 bytes. Conversion to UTF-16 happens only at the LSP
boundary. Line and column values are derived views, never cache identities.

## External sources

Ownership is resolved before an external `src` is read. An unclaimed block
must not cause Vize to access another file. Relative paths resolve from the SFC;
aliases and packages use the host resolver so Vite, Vite+, Nuxt, CLI, and LSP
agree.

The resolved identity, content hash, permissions, and reverse dependency enter
the execution plan and cache. Diagnostics use the external URI as their primary
location and the SFC `src` value as related information. Rename, definition,
document links, and dependency inspection expose both ends.

Formatting the SFC never implicitly writes an external source. The external
file may be formatted only through its own document request or an explicit
workspace edit that names every affected URI.

## Parse result and shared artifact

Plugin-specific ASTs remain opaque to the host. The provider returns a bounded,
versioned artifact and public projections:

```ts
interface CustomBlockParseResult {
  artifact?: {
    schemaVersion: number;
    mediaType: string;
    bytes: Uint8Array;
  };
  facts?: readonly FactPacket[];
  embeddedDocuments?: readonly EmbeddedDocument[];
  dependencies?: readonly Dependency[];
  diagnostics?: readonly VizeDiagnostic[];
  map?: ProjectionMap;
}
```

Only later hooks from the same plugin decode `artifact`. Vize owns its size
limit, fingerprint, transport, cache, and dependency invalidation. Public
projections are validated independently so a malformed map or diagnostic
cannot corrupt another product.

The artifact is immutable for a content identity. Compiler, type checker,
linter, formatter, LSP, editor, runtime generation, and HMR consume the same
instance or a serialized equivalent, rather than re-parsing the block.

## Mapping and edits

The mapping graph distinguishes three domains:

1. authored inline block content;
2. an authored external source; and
3. generated runtime, type-check, or embedded documents.

Diagnostics may use approximate display mappings, clearly marked as such.
Fixes, rename, formatting, and code actions require exact bidirectional ranges.
Generated-only regions cannot receive authored edits. Overlapping edits enter
the common deterministic conflict planner.

The core formatter owns the opening tag and attributes. A block formatter owns
only the inline content range. It must be idempotent and preserve the exact
semantic parse result. External content follows the separate-file rule above.

## Type checking and linting

A provider may emit mapped TypeScript documents, declarations, globals, and
diagnostics. It does not receive a TypeScript checker object or mutate another
virtual file. Content Mapper, ordinary checking, incremental checking,
declaration output, and editor checking consume the same projected documents.

Lint rules request declared public facts on demand. A read-only observer can
lint a block without decoding the owner artifact. Diagnostics and fixes retain
their complete identity and exact authored ranges. The v1 custom-block warning
is preserved unless the matching v2 owner is active.

## Runtime and compiler output

Runtime output is structured rather than concatenated text:

```ts
interface CustomBlockRuntimeResult {
  module: VirtualModule;
  apply: "none" | "component-augment" | "app-install" | "side-effect" | "asset";
  dependencies?: readonly Dependency[];
  map?: ProjectionMap;
}
```

Bundler adapters lower this result to their native module graph. VDOM, Vapor,
SSR, and hydration must either consume equivalent output or reject the plugin
for that target before compilation. A provider cannot rely on adapter-specific
query-string behavior as part of the portable API.

## HMR

Each changed artifact returns one bounded disposition:

- `none`;
- `reload-custom-module`;
- `reload-component`; or
- `full-reload`.

The host may raise an unsafe weak result and combines affected blocks using the
strongest disposition. Analysis-only changes do not invalidate runtime output.
External changes invalidate only their reverse dependency set. A provider must
declare whether state can survive its accepted update.

## LSP and editor behavior

The owner may implement the common bounded provider set: diagnostics,
completion and resolve, hover, signature help, definition, type definition,
implementation, references, rename, symbols, semantic tokens, inlay hints,
code actions, code lenses, links, colors, folding, and selection ranges.

Requests carry document version and cancellation. Late responses are dropped.
Core results remain unless ownership explicitly grants replacement for that
document. Packaged editors discover capabilities from the server and do not
need block-specific extension code.

The `src` attribute always supports document links and definition. Rename is
allowed only when the host resolver can prove an exact workspace edit for both
the reference and target.

## Performance contract

If an SFC has no custom blocks, the provider registry, hashing, serialization,
cache, and guest bridge are not entered. An unclaimed block uses the existing
raw path without building a shared artifact.

An admitted block is parsed at most once per content identity. Native and JS
backends pass the same output corpus; JS callbacks cross the process boundary
once per admitted batch, never once per token or AST node. PR evidence includes
callback counts, IPC bytes, cache hit and miss counts, cold/warm latency, CPU,
peak RSS, allocations, and filesystem writes on Node and Bun.

## Migration gates

1. Add the immutable artifact beside current paths and compare full output,
   diagnostics, ranges, mappings, edits, and HMR without changing consumers.
2. Move `art` through a differential canary across every current consumer.
3. Share `route` and `i18n` analysis while leaving existing runtime ownership.
4. Admit provider-owned mappings, dependencies, and HMR only after exact parity.
5. Publish the general API after Node/Bun, Vite/Vite+/Nuxt, VDOM/Vapor/SSR,
   packaged-editor, native/JS, and real-project qualification passes.

Every gate fails closed on an unsupported schema, owner conflict, malformed
map, undeclared dependency, nondeterministic output, or behavior difference.
