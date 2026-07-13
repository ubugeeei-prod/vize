---
title: Production Atlas hosts
---

# Production Atlas hosts

Every production host composes peer providers and requests roots from Atlas.
This table makes the compilation lifetime and requested contract explicit.

| Host                          | Compilation lifetime                                                                                                           | Requested roots                                                                                                                                                                                     |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `vize build`                  | one multi-source snapshot per invocation                                                                                       | source-aware SFC compiled modules and maps                                                                                                                                                          |
| `vize lint`                   | one multi-source compilation, including autofix revalidation                                                                   | Patina document reports plus optional full Croquis CF analysis                                                                                                                                      |
| `vize check`                  | one project snapshot for Vue, TS, declarations, and JSX/TSX                                                                    | Canon typed-document products; SFCs use descriptor and Croquis, conditional Relief/Module, and no fabricated Flow                                                                                   |
| Maestro                       | one URI-keyed mutable compilation; open and discovered file-backed Vue dependency URIs retain source identity across revisions | SFC descriptor/Module/Relief/Croquis, raw-template Relief/Croquis, JSX syntax, Patina, Canon, `GlyphFormatProduct`, and virtual documents as requested                                              |
| Standalone Glyph / `vize fmt` | one document compilation per SFC formatting request                                                                            | `GlyphFormatProduct` over the SFC descriptor                                                                                                                                                        |
| Inspector                     | one report-scoped multi-source compilation                                                                                     | `InspectorAgentReport` over per-source analyses; SFC uses descriptor/Relief/Croquis plus conditional Module, JSX/TSX uses owned JSX syntax/Module/Croquis without Relief, and raw JS/TS uses Module |
| NAPI/WASM bindings            | one compilation per stateless request; one compilation shared by each batch API                                                | SFC/JSX compile, raw `TemplateCompile`, Patina, Canon, and cross-file analysis roots exposed by that binding surface                                                                                |
| Bundler hosts                 | one native compile request per transform, with native batch compilation where the host batches inputs                          | SFC or JSX compiled-module products and source maps through the binding API; bundlers do not own graph products                                                                                     |

For normal `.vue` editor requests, Maestro queries `CanonVueDocumentProduct`
for the host and every discovered non-Art Vue dependency in that same
compilation. Open-document contents take precedence over disk contents, and
either source keeps its URI-keyed `SourceId` while revisions change. Maestro
then passes the prebuilt host and dependency products to Corsa as overlays;
this synchronization does not create a private `Compilation` or reparse the
SFCs. Art/Musea virtual documents use specialized generation paths and are
outside this guarantee.

The SFC compiled-module root requests Rendu only when a template must be
rendered and invokes the graph-native DOM, SSR, or Vapor emitter. It does not
call the legacy frontend-coupled backend entry points or request
`CroquisDocumentProduct`. A template plus script requests only the narrow
template-binding projection needed by transforms and Rendu. A script-only SFC
requests no Relief, Croquis, Flow, or Rendu product; a template without script
requests syntax and Rendu but not Module or semantic analysis.
