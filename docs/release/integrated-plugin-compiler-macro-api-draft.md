# Integrated compiler-macro API draft

> [!WARNING]
> This is a next-wave design draft for plugin API v2. It does not change built-in
> compiler macros, generated output, diagnostics, or the v1 release scope.

Custom compiler macros should provide Vue-Macros-like product value without
requiring separate compiler, type-checker, lint, bundler, and editor plugins.
One declaration owns recognition, expansion, runtime artifacts, type projection,
source maps, diagnostics, HMR, and language-tool behavior.

The parent [integrated plugin API draft](./integrated-plugin-api-draft.md)
defines backend selection, versioning, security, and shared caches. This
document defines the `compiler.macro` capability for JavaScript and Rust authors.

## Compatibility baseline

Vize already extracts built-in macro artifacts such as `definePage`,
`definePageMeta`, and route rules. Vite exposes selected artifacts as virtual
modules, while Canon and Maestro separately model compiler-macro types and
editor behavior. Those paths are compatibility oracles, not public extension
points to replace in one step.

The migration first represents existing artifacts in the shared contract and
compares exact compiler output, artifacts, diagnostics, mappings, virtual types,
and HMR. A plugin macro is admitted only after the no-plugin and built-in paths
remain output-identical.

## Registration

```ts
api.compiler.macro("defineFeature", {
  syntax: {
    documents: ["script-setup"],
    binding: {
      kind: "import",
      from: "@acme/vize-macros",
      export: "defineFeature",
    },
    calls: {
      arguments: { min: 1, max: 1 },
      typeArguments: { min: 0, max: 1 },
    },
  },
  phase: "script:pre-lower",
  targets: ["vdom", "vapor", "ssr"],
  analyze: "feature-definition/v1",
  expand: "feature-expansion/v1",
  emits: ["runtime-import", "setup-binding", "macro-artifact", "virtual-type-document"],
  languageServer: {
    completion: true,
    hover: true,
    definition: true,
    references: true,
    rename: true,
    signatureHelp: true,
  },
});
```

The plugin owns the resolved macro binding, not every identifier with the same
text. Registration declares supported SFC regions, import source and export,
call shape, public phase, output targets, artifact schemas, and editor features.

Global macros are an explicit alternative registration mode. They fail on name
collisions and cannot shadow local bindings. Imported macros are preferred
because definition, rename, versioning, and package ownership remain exact.

## Binding-aware recognition

The host parser and resolver identify candidates before a plugin runs. Matching
uses import and lexical-binding identity:

- renamed imports are recognized;
- namespace calls are recognized only when declared;
- locally shadowed names are not macros;
- same-name imports from another package are not macros;
- strings, comments, property names, optional calls, and invalid arity do not
  become candidates accidentally; and
- parse recovery produces bounded candidates with explicit uncertainty rather
  than guessing from source text.

The manifest says whether a macro may appear once, multiple times, at top level,
inside control flow, or in a specific SFC region. Violations produce authored
diagnostics before expansion.

## Public candidate input

Private parser and compiler arenas never cross the plugin boundary. The host
builds a versioned immutable candidate batch:

```ts
interface MacroCandidate {
  id: MacroCandidateId;
  source: SourceIdentity;
  document: "script" | "script-setup" | "template";
  binding: ResolvedBindingIdentity;
  callRange: ByteRange;
  calleeRange: ByteRange;
  typeArguments: readonly PublicTypeSyntax[];
  arguments: readonly PublicExpression[];
  scope: PublicScopeId;
  facts: Readonly<Record<string, FactHandle>>;
}
```

`PublicExpression` is a versioned, bounded syntax IR with exact authored ranges.
Unknown syntax stays an opaque exact source slice that a plugin may preserve or
reject. Plugins cannot mutate nodes or keep host handles after the batch.

Static fact demands declare any binding, constant-evaluation, import, component,
or bounded type information needed by expansion. Unknown facts refuse the macro
at plan construction instead of returning empty data.

## Structured expansion

```ts
interface MacroExpansion {
  candidate: MacroCandidateId;
  edits?: readonly ExactSourceEdit[];
  imports?: readonly RuntimeImport[];
  bindings?: readonly SetupBinding[];
  componentOptions?: readonly ComponentOptionContribution[];
  artifacts?: readonly VersionedMacroArtifact[];
  virtualDocuments?: readonly MappedVirtualDocument[];
  dependencies?: readonly Dependency[];
  diagnostics?: readonly VizeDiagnostic[];
  hmr?: MacroHmrDisposition;
}
```

Common operations are structured: erase a call, replace it with an expression,
insert or reuse an import, add a setup binding, contribute component options,
and emit a virtual module or artifact. An exact edit is the escape hatch inside
the claimed candidate or negotiated phase range; arbitrary whole-file rewrite
is not.

Every edit and generated region carries an exact map to authored source or an
explicit generated-only marker. The host rejects overlaps, out-of-range edits,
missing maps, stale candidate IDs, malformed artifacts, and undeclared imports
or dependencies.

## Phases, ordering, and ownership

The host exposes a small stable phase set rather than internal pass names:

- `script:analyze` for read-only facts and diagnostics;
- `script:pre-lower` for setup/runtime expansion;
- `template:pre-lower` for claimed template syntax;
- `component:assemble` for structured component-option contributions; and
- `artifact:emit` for mapped virtual modules and build metadata.

A macro selects the earliest phase that provides its required records. Plugins
order with validated `before` and `after` edges. Discovery order and numeric
priority are not semantics. Cycles, overlapping owners, incompatible phase
claims, and competing edits fail before code generation.

One plugin may expose several macros and share analysis facts. A macro cannot
observe another macro's private artifact; cross-plugin data uses a declared
versioned public fact or artifact schema.

## Runtime and target behavior

A macro declares one of these runtime dispositions per target:

- compile-time erased;
- replace with a pure expression;
- emit a direct runtime import and call;
- hoist a stable value or setup binding;
- contribute component options;
- emit a virtual module or asset; or
- refuse the target.

VDOM, Vapor, SSR, hydration, and client-only output either implement the same
declared semantics or fail during plan construction. There is no silent fallback
from an unsupported Vapor or SSR expansion to VDOM behavior.

Runtime imports are structured references and deduplicated by the host. Generated
code does not concatenate arbitrary strings. Side effects, evaluation count,
hoisting, request scope, serialization, and tree-shaking are explicit.

## Types and Content Mapper

Macro type behavior is declared with structured imported types and mapped
virtual documents. A plugin may contribute call signatures, generic constraints,
return types, setup bindings, component public types, and declaration artifacts.
It never receives a mutable TypeScript program or post-filters unrelated errors.

Type-aware expansion uses bounded semantic queries over the same snapshot as
Canon and Content Mapper. Required queries fail closed when unavailable. The
compiler, ordinary type checking, Content Mapper, declaration output, and editor
checking consume the same macro artifact and source-map chain.

Generic and imported type references keep package, export, type arguments, and
resolution identity so hover and definition navigate to the original symbol.
Runtime validation remains a separate schema contract when a macro consumes
serializable configuration.

## Lint, LSP, and editor behavior

The macro declaration supplies signature help, completion, hover, definition,
references, rename, semantic tokens, inlay hints, diagnostics, and code actions.
These providers read the cached candidate and expansion artifact; they do not
re-run a separate macro implementation inside the language server.

Lint rules can consume public macro facts and artifacts without decoding a
plugin-private payload. Fixes and editor edits require exact authored mappings
and enter the common conflict planner.

Rename follows the resolved imported binding and declared generated references.
Generated-only names are not exposed as editable authored symbols. Requests are
versioned and cancellable; late results are dropped.

## HMR and dependencies

Dependencies from macro analysis, runtime imports, external schemas, and emitted
artifacts join the project graph. HMR distinguishes analysis-only, type-only,
runtime-module, component-shape, and full-application changes.

```ts
type MacroHmrDisposition =
  "none" | "invalidate-types" | "reload-runtime-module" | "reload-component" | "full-reload";
```

The host may raise an unsafe weak result. A docs or hover change cannot invalidate
runtime output, while a component-shape change cannot claim type-only HMR.

## JavaScript and single-file authoring

A macro may be authored directly in `vite.config.ts`, a project-local module, or
a published package. One authored plugin file may contain config-time analysis
and emitted runtime helpers. The SDK compiler splits runtime exports into stable
virtual modules without writing generated source into the project.

Ordinary module imports are preserved. Config-process values cross the split
only as declared canonical options, import references, or explicit virtual-module
exports. An implicit closure capture fails with a source diagnostic.

JavaScript macros run under Node or Bun. Candidates for one plugin and document
cross as one batch, never one callback per call, node, type query, or editor
request. Declarative match and expansion tables lower directly to native host
operations without keeping a JS guest alive.

## Rust authoring

Rust plugins declare the same macro and receive typed borrowed candidate batches:

```rust
#[vize_plugin(api = 2, backend = "native")]
impl Plugin for FeatureMacros {
    fn manifest() -> Manifest {
        Manifest::new("com.acme.feature-macros", env!("CARGO_PKG_VERSION"))
            .compiler_macro(
                CompilerMacro::imported("defineFeature", "@acme/vize-macros")
                    .phase(MacroPhase::ScriptPreLower)
                    .targets([Target::Vdom, Target::Vapor, Target::Ssr]),
            )
    }

    fn expand(
        &self,
        batch: MacroBatch<'_>,
        output: &mut MacroOutputSink<'_>,
    ) -> PluginResult<()> {
        for candidate in batch.candidates() {
            output.erase(candidate)?;
            output.artifact(candidate, FeatureArtifact::from(candidate)?)?;
        }
        Ok(())
    }
}
```

The Rust SDK generates stable C ABI entrypoints; Rust layouts never cross the
boundary. The host invokes once per admitted batch with size-prefixed borrowed
tables and host-owned output sinks. Per-platform artifact packages use the same
Parcel-inspired distribution and fail-closed backend selection as custom lint.

## Cache, determinism, and performance

When no plugin macro is registered, existing compiler entrypoints, scans, and
output remain unchanged. A registered but absent macro uses the parser's existing
binding index and performs no guest call.

Candidate, analysis, expansion, type, editor, and runtime hashes are separate so
type-only changes do not invalidate runtime output. Keys include exact source,
Vize build, target, API and artifact schemas, plugin integrity, canonical
options, imported binding identity, facts, and declared dependencies.

Identical input must produce byte-identical edits, artifacts, maps, diagnostics,
and dependency sets. Cache hits execute no plugin callback. The host bounds
candidates, batch bytes, output bytes, edits, artifacts, dependencies, time,
memory, and diagnostics, and attributes all cost to the owning plugin.

Every implementation PR records exact base/head SHAs and measures output and
diagnostic identity, cold/warm wall, CPU, peak RSS, allocations, files, bytes,
callbacks, candidate count, and cache hits under native, Node, and Bun. Disabled
and absent-macro p50/p95 target at most 1% overhead.

## Migration and acceptance

1. Represent existing macro artifacts in the shared schema without changing a
   consumer.
2. Differentially migrate `definePage` across compiler, Vite, Canon, Content
   Mapper, Maestro, HMR, and source maps.
3. Add one JS plugin macro behind the v2 opt-in and compare Node/Bun output.
4. Add the Rust implementation and require the same conformance corpus.
5. Publish an independent package only after VDOM/Vapor/SSR, Nuxt/Vite/Vite+,
   CLI, LSP, packaged-editor, and real-project qualification passes.

Acceptance includes imported aliases and shadows, global conflicts, invalid
arity, generics, parse recovery, Unicode and CRLF maps, multi-macro ordering,
cache invalidation, HMR, SSR isolation, all Vue versions in scope, and exact
built-in behavior when the plugin is disabled.
