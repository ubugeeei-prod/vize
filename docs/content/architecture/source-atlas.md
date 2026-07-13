---
title: Atlas artifact graph
---

# Atlas artifact graph

Atlas is Vize's typed, demand-driven execution substrate. It is not a registry
of compiler nouns and it is not a Relief-to-Croquis pipeline.

A compilation owns stable source identities, revisions, open products and
providers, dependency planning, memoized artifacts, selective invalidation,
provider-attributed observations, execution outcomes, counters, and traces.
Compilers, linters, typecheckers, language-server features, formatters,
inspectors, and bundler adapters are recipes that request root products from
the same compilation. The canary production paths now follow that contract:
`build`, `lint`, `check`, Maestro, Glyph, Inspector, NAPI/WASM, and the
Vite/Nuxt/bundler hosts consume typed products rather than rebuilding a private
frontend pipeline.

“Zero cost” in this design describes compiler operation:

- a provider outside the requested dependency closure is not planned or run;
- an unrequested product creates no persistent cache entry;
- two roots share each common dependency execution;
- a source or typed input change evicts only affected products and transitive
  consumers.

It does not claim that generated JavaScript has zero runtime cost.

This page is the implementation contract for
[#1634](https://github.com/ubugeeei-prod/vize/issues/1634).

## One graph, several representations

Atlas is not a universal IR. The representations are peer products with
different jobs:

| Product                        | Owner                   | One job                                                                                                           |
| ------------------------------ | ----------------------- | ----------------------------------------------------------------------------------------------------------------- |
| SFC descriptor/template source | `vize_atelier_sfc`      | Decompose a container and retain parent ranges.                                                                   |
| Raw Vue template frontend      | `vize_atelier_template` | Parse standalone template sources without fabricating an SFC container.                                           |
| JS/TS module snapshot          | `vize_module`           | Own parser-lifetime-free imports, exports, declarations, references, diagnostics, and OXC CFG facts.              |
| JSX/TSX syntax snapshot        | `vize_atelier_jsx`      | Own OXC-derived JSX syntax without constructing Relief.                                                           |
| Relief snapshot                | `vize_relief`           | Preserve Vue-template syntax and exact source locations.                                                          |
| Croquis semantic snapshot      | `vize_croquis`          | Preserve derived identity, scope, binding, usage, and reactivity facts.                                           |
| Flow graph                     | `vize_flow`             | Represent single-file control, data, and effect flow.                                                             |
| Croquis project snapshot       | `vize_croquis_cf`       | Lightweight, serializable component and provide/inject index across requested documents.                          |
| Cross-file analysis artifact   | `vize_croquis_cf`       | Run the full dependency/rule analyzer and retain diagnostics, complexity, layouts, and tree output.               |
| Rendu HIR                      | `vize_rendu`            | Represent frontend-neutral render intent for backends.                                                            |
| DOM/SSR/Vapor output           | target Atelier crates   | Emit or plan one target from Rendu without parsing source.                                                        |
| Patina diagnostics             | `vize_patina`           | Request the syntax, Module, and semantic products selected by the document shape and production recipe.           |
| Canon Virtual TS               | `vize_canon`            | Consume the SFC descriptor and Croquis, plus Relief for a template and Module for a script; never fabricate Flow. |

No product above is “the foundation”. Atlas supplies identity and execution;
the owning crate supplies the representation and independently applicable
providers.

```mermaid
flowchart LR
    C["Compilation / Atlas"]
    SFC["SFC source"] --> SD["SFC descriptor"]
    SD --> VT["template source"]
    SD -. "when script exists" .-> MODULE["module facts"]
    VT -. "when requested" .-> RELIEF["Relief syntax"]
    RAWTPL["raw template"] -. "when requested" .-> RELIEF
    RAWTPL -. "when requested" .-> SEM["Croquis semantics"]
    JSX["JSX / TSX source"] --> OXC["owned JSX syntax"]
    OXC -. "when requested" .-> MODULE
    RAWMOD["raw JS / TS"] --> MODULE

    RELIEF -. "when requested" .-> SEM
    MODULE -. "when requested" .-> SEM
    OXC -. "when requested" .-> SEM
    RELIEF -. "when requested" .-> FLOW["Flow graph"]
    OXC -. "when requested" .-> FLOW
    MODULE -. "when requested" .-> FLOW
    RELIEF -. "when requested" .-> RENDU["Rendu HIR"]
    OXC -. "when requested" .-> RENDU

    RENDU -. "selected target" .-> DOM["DOM module"]
    RENDU -. "selected target" .-> SSR["SSR module"]
    RENDU -. "selected target" .-> VAPOR["Vapor plan"]
    RELIEF -. "template rules" .-> LINT["Patina diagnostics"]
    MODULE -. "module rules" .-> LINT
    SEM -. "semantic rules" .-> LINT
    SD --> VTS["Canon Virtual TS"]
    SEM --> VTS
    RELIEF -. "when template exists" .-> VTS
    MODULE -. "when script exists" .-> VTS
    MODULE --> INSPECT["Inspector module facts"]
    SEM --> PROJECT["Croquis project snapshot"]
    SEM --> CFA["full cross-file analysis"]
    RAWMOD --> CFA

    C -. "plans, caches, invalidates" .-> SD
    C -.-> OXC
    C -.-> MODULE
    C -.-> RELIEF
    C -.-> SEM
    C -.-> FLOW
    C -.-> RENDU
```

The arrows above are available provider edges, not one mandatory pipeline.
Planning inspects the source shape and the requested root before selecting a
closure. A template-only source does not acquire Module facts, and a
script-only SFC does not acquire Relief. Raw templates can request Relief and
Croquis directly without fabricating an SFC descriptor.

## Module, Relief, Croquis, Flow, and Rendu

These names are deliberately not interchangeable.

`vize_module` is the source-faithful, owned JavaScript/TypeScript layer. It
retains module declarations and references and projects OXC's real CFG without
depending on Relief, Croquis, or an Atelier crate. SFC script blocks, the single
JSX/TSX parse, and raw JS/TS files all provide that same product. Croquis remains
the Vue/component semantic model; it is not the module parser or the CFG owner.

| Question                                   | Relief                                                   | Croquis                                                  | Flow                                                             | Rendu                                                                       |
| ------------------------------------------ | -------------------------------------------------------- | -------------------------------------------------------- | ---------------------------------------------------------------- | --------------------------------------------------------------------------- |
| What was written?                          | Yes: tags, directives, expressions, comments, locations. | No.                                                      | No.                                                              | No.                                                                         |
| What does a name mean?                     | No.                                                      | Yes: identity, bindings, scopes, components, reactivity. | References already identified symbols/values.                    | Only opaque render expressions/bindings.                                    |
| How can execution branch or effects order? | Retains `v-if`/`v-for` syntax.                           | May supply semantic facts.                               | Yes: blocks, control/data/effect edges, reachability, dominance. | Retains structured render branches/iteration, not an analysis CFG.          |
| How should a target render it?             | No.                                                      | No.                                                      | No.                                                              | Yes: elements, components, slots, props, directives, text, branches, lists. |
| Frontend-specific?                         | Vue template.                                            | Value contract is frontend-neutral.                      | Frontend-neutral.                                                | Frontend-neutral.                                                           |

Relief is therefore not “before Croquis” in every recipe. A syntax rule may
request Relief and stop. JSX can produce Croquis or Rendu directly from its own
owned syntax and never create Relief. A compiler may request Rendu without
requesting Flow; a flow-aware tool may request Flow without Rendu.

`vize_croquis_cf` is also separate from `vize_flow`: Flow is one compilation
unit's CFG/data/effect representation, while Croquis CF owns opt-in cross-file
module/component analysis. Within Croquis CF, `CroquisProjectProduct` is only a
lightweight semantic index. `CrossFileAnalysisProduct` is the distinct full
analyzer result; neither is presented as a substitute for the other.

## Open provider selection

Products and providers are ordinary Rust types. Atlas contains no enum naming
Relief, Croquis, Rendu, tools, or targets.

`Product::Value` is the owned `'static` storage kept in the cache, not a rule
that every consumer must use an owned tree. A product can separately implement
`ProductView` to project that storage into a borrow, an arena/intern-table
facade, or an iterator-like stream tied to the query outcome's lifetime.
The storage may itself be a cloned `Shared` handle to compilation-owned source
or arena bytes, so a provider can expose a lazy view without copying the source
or materializing an intermediate collection. Products that only need their
stored value do not implement the view contract.
`CachePolicy::Transient` keeps an ephemeral or streamed value inside the
current execution only, so sibling roots can share it without creating a
persistent artifact entry. If every consumer is a cache hit, Atlas prunes the
transient dependency instead of recreating it speculatively.

Multiple crates may register providers for one product. For example, the SFC,
raw-template, and JSX crates provide source-specific `RenduProduct` and
`CroquisDocumentProduct` providers.
`Provider::supports` selects exactly one provider for a source during planning.
No central `match input_kind` must be edited when another frontend is added.

Planning fails explicitly when no provider applies or more than one provider
claims the same product. The immutable plan records the selected provider
identity, its declared dependencies, and relevant typed-input revisions.

Providers can attach structured diagnostics, fallback records, or notes to the
exact `(SourceId, ProductId)` request they are serving. Each observation also
stores the concrete `ProviderId`, target source/range, code, and message. These
side outcomes are cached with the product, so a cache hit preserves provenance
instead of reconstructing telemetry after execution. JSX parser diagnostics
exercise this path in the canary integration tests.

## Frontend registration and host composition

A frontend registrar registers only provider implementations owned by that
frontend crate. It may implement a peer product identity such as Module,
Croquis, Flow, or Rendu for its own source shape, but it never calls a peer
crate's registrar and never installs DOM, SSR, or Vapor backends implicitly.

The application host is the composition root. A build host registers the SFC
or JSX frontend plus the selected peer render backends. A lint host registers
the applicable frontends, raw Module support, and Patina, but no render
backend. Cross-file and compact semantic projections are registered only by
hosts that expose those roots. Registration alone does not execute a product;
the requested root still determines the plan.

`tools/check-graph-backend-boundaries.sh` enforces this registrar boundary for
the SFC, JSX/TSX, and raw-template frontends. Source-shaped plan tests then
prove that raw JS/TS remains Module-only, template-only SFCs avoid Module, and
unrequested peer products have zero executions and no cache entry.

## Sources and provenance

`Compilation` assigns stable `SourceId` values and monotonic revisions. An
embedded source records its parent ID, the exact parent revision, and a
half-open byte range. SFC template products retain that information, so a
template, generated projection, diagnostic, and mapping can stay connected
without scanning generated text to rediscover structure.

Updating a parent revises its embedded descendants and evicts only their cache
entries. Unrelated source trees remain reusable. A plan created for an older
source, provider registry, or relevant typed-input revision is rejected rather
than executed against mismatched state.

For an SFC with authored script, `SfcScriptSyntaxProduct` is the parse owner.
Each `<script>` and `<script setup>` block is parsed once per source revision
while its OXC `Program` is live. Before that allocator is dropped, the provider
projects the owned `ModuleDocument`, Croquis script analysis, and compiler
preanalysis used by normal-script and script-setup compilation. Module, Canon,
Croquis, and the SFC compiler consume those projections instead of reparsing the
authored block.

## Cross-source requests and immutable snapshots

A request is identified by both `SourceId` and `ProductId`. Providers declare
complete cross-source dependency requests during planning and can read only
those declared products during execution. A plan captures every participating
source revision, so no project result can silently mix old and new documents.

Croquis CF exposes two executable cross-source contracts. The lightweight
`CroquisProjectProvider` declares one `CroquisSemanticProduct` request for
every supported SFC or JSX/TSX source. The full
`CrossFileAnalysisProvider` instead declares complete
`CroquisDocumentProduct` requests for SFC/JSX/TSX sources, records raw source
revision dependencies for JS/TS modules, constructs the real
`CrossFileAnalyzer` inside the provider, and returns diagnostics, dependency
facts, complexity, source layouts, and provide/inject tree output. No CLI or
WASM host constructs that analyzer itself.

The cache records the source-revision dependencies of each product. Updating a
TSX component evicts that component's semantic document and dependent project
products, while an unchanged SFC document remains reusable. Updating a raw
`.ts` module also evicts the full cross-file result without manufacturing a
semantic product for that module. An immutable compilation snapshot can be
forked for editor or project work without sharing later mutations back into
the original compilation.

## Versions and targets are context

Vue v1, v2, v2.7, and v3 are values of the open `VueDialectInput`; DOM, SSR,
Vapor, non-Vapor, and custom-renderer availability are values of
`RenderCapabilitiesInput`. They shape provider applicability or output but are
not top-level pipelines and are not Atlas product variants.

Providers declare the inputs that can affect them. Atlas propagates that
relevance through dependencies. Changing render capabilities can evict a DOM
output while keeping Relief and Rendu cached; changing the Vue dialect evicts
Relief and every dependent product while preserving the SFC descriptor and
template-source product.

## Recipes

A recipe is only a root-product set:

| Recipe                    | Root products                    | Products intentionally absent                |
| ------------------------- | -------------------------------- | -------------------------------------------- |
| DOM compiler              | DOM module                       | SSR, Vapor, lint, Virtual TS, Flow           |
| Multi-backend compiler    | DOM + SSR + Vapor                | lint and Virtual TS unless also requested    |
| Document lint             | Patina report                    | Rendu and backend output                     |
| Typecheck                 | Canon Virtual TS                 | Rendu and backend output                     |
| Combined editor analysis  | Patina report + Canon Virtual TS | Rendu unless a preview also requests it      |
| Lightweight project index | Croquis project snapshot         | full cross-file rules, Rendu, backend output |
| Cross-file rule analysis  | Cross-file analysis artifact     | Rendu and backend output                     |

Atlas derives the source-shaped transitive closure and executes it in
topological order.
The canary tests prove that multi-backend SFC and TSX requests execute Rendu
once, SFC and JSX combine authored module control flow with template/render
control flow, and combined lint/typecheck requests execute Croquis semantics
once. Raw JS/TS Flow plans contain only Module plus Flow—no Relief or Croquis.
Nested-function and unreachable OXC edges remain explicit non-traversable Flow
edges rather than being misreported as entry-reachable work. Canon does not
request Flow merely to count it; typechecking stops at the products it actually
uses. The tests also prove that TSX compiler/lint/typecheck/flow closures contain
no Relief product. Separate multi-source tests prove both Croquis CF roots run
only when requested, the full root executes the actual analyzer, and one changed
source selectively recomputes only its document plus dependent project roots.

The hidden `vize graph <sources...>` command remains the inspectable diagnostic
consumer. It registers every source in one immutable compilation snapshot,
then requests compiler, Patina, Canon, and optional project roots. Its JSON
report records the selected provider and executed/cache status for every
request. Production consumers use the same product/provider contracts directly;
there is no shadow execution or second parse used only for telemetry.

## Production host roots

| Host                          | Compilation lifetime                                                                                                           | Requested roots                                                                                                                                        |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `vize build`                  | one multi-source snapshot per invocation                                                                                       | source-aware SFC compiled modules and maps                                                                                                             |
| `vize lint`                   | one multi-source compilation, including autofix revalidation                                                                   | Patina document reports plus optional full Croquis CF analysis                                                                                         |
| `vize check`                  | one project snapshot for Vue, TS, declarations, and JSX/TSX                                                                    | Canon typed-document products; SFCs use descriptor and Croquis, conditional Relief/Module, and no fabricated Flow                                      |
| Maestro                       | one URI-keyed mutable compilation; open and discovered file-backed Vue dependency URIs retain source identity across revisions | SFC descriptor/Module/Relief/Croquis, raw-template Relief/Croquis, JSX syntax, Patina, Canon, `GlyphFormatProduct`, and virtual documents as requested |
| Standalone Glyph / `vize fmt` | one document compilation per SFC formatting request                                                                            | `GlyphFormatProduct` over the SFC descriptor                                                                                                           |
| Inspector                     | one report-scoped multi-source compilation                                                                                     | `InspectorAgentReport` over per-source analyses; SFC uses descriptor/Relief/Croquis plus conditional Module, JSX/TSX uses owned JSX syntax/Module/Croquis without Relief, and raw JS/TS uses Module |
| NAPI/WASM bindings            | one compilation per stateless request; one compilation shared by each batch API                                                | SFC/JSX compile, raw `TemplateCompile`, Patina, Canon, and cross-file analysis roots exposed by that binding surface                                   |
| Bundler hosts                 | one native compile request per transform, with native batch compilation where the host batches inputs                          | SFC or JSX compiled-module products and source maps through the binding API; bundlers do not own graph products                                        |

For normal `.vue` editor requests, Maestro queries `CanonVueDocumentProduct`
for the host and every discovered non-Art Vue dependency in that same compilation.
Open-document contents take precedence over disk contents, and either source
keeps its URI-keyed `SourceId` while revisions change. Maestro then passes the
prebuilt host and dependency products to Corsa as overlays; this synchronization
does not create a private `Compilation` or reparse the SFCs. Art/Musea virtual
documents use specialized generation paths and are outside this guarantee.

The SFC compiled-module root requests Rendu only when a template must be
rendered and invokes the graph-native DOM, SSR, or Vapor emitter. It does not
call the legacy frontend-coupled backend entry points. A script-only SFC
requests no Relief, Croquis, Flow, or Rendu product; a template without script
requests syntax and Rendu but not Module or semantic analysis.

## Physical dependency rules

- `vize_atlas` depends only on lower-level source/utility primitives and names
  no domain product.
- `vize_rendu` has no Relief, Croquis, SFC, JSX, or backend dependency.
- `vize_flow` has no syntax, semantic, frontend, Atelier, or cross-file
  dependency.
- `vize_module` depends only on Atlas, Carton, Flow, and OXC; it has no Relief,
  Croquis, Atelier, Patina, or Canon dependency.
- graph-native DOM, SSR, and Vapor providers consume Rendu and do not parse
  source. Legacy frontend-coupled entry points remain only as deprecated/public
  compatibility surfaces; production recipes do not invoke them.
- `vize_atelier_core` owns the IR used by its legacy-compatible Vue-template
  transform/emission lane. It does not own the Atlas product/graph kernel or act
  as a shared workspace foundation. Relief, Armature, and Carton aliases are
  crate-private implementation details; no public root facade remains.
  Production consumers import those contracts from their owning crates directly.
- frontend-specific producers live in frontend crates; stable owned products
  cross the graph boundary.

Executable dependency-boundary tests reject production imports that route
owned syntax, parser, or allocator APIs through Atelier Core. They also enforce
the Atlas, Module, Rendu, Flow, and Croquis contracts.
Integration tests enforce the
Rendu-only dependency closure of the graph-native backend providers; they do
not claim that the legacy backend surfaces have already been removed.

## Measurements

The initial compiler/lint/typecheck/combined allocation, peak-live-byte,
wall-time, query, execution, and cache-entry measurements are recorded in
[Artifact graph cost baseline](./artifact-graph-cost-baseline.md). They are a
canary baseline, not a cross-machine speed claim.
