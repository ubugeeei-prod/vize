---
title: Source Atlas Glossary
---

# Source Atlas Glossary

This page is the stable vocabulary and request contract for the Source Atlas
architecture. It complements [Source Atlas](/architecture/source-atlas) (the
narrative) with a fixed glossary and a plate-request matrix, so every
implementation track names the same plates and agrees on who may request each
one. The implementation track is
[#1766](https://github.com/ubugeeei-prod/vize/issues/1766).

## Glossary

| Term            | One job                                                                                      |
| --------------- | -------------------------------------------------------------------------------------------- |
| `Source Atlas`  | Neutral request ledger: requested sources, products, targets, coordinates, and fallbacks.    |
| `Armature`      | The Vue template tokenizer and parser that builds Relief nodes.                              |
| `Relief`        | Source syntax: what node was written, its shape, and its location.                           |
| `Croquis`       | Derived meaning: identity, scopes, bindings, usage, dependencies, and analysis graphs.       |
| `Virtual TS`    | A typecheck/editor projection built from Armature/Relief/Croquis — not the canonical IR.     |
| `Rendu`         | Borrowed render projection for DOM/SSR/Vapor emitters; not syntax or general semantics.      |
| `AtelierOutput` | The structured output plate (imports/hoists/functions/exports/sections/maps) before flatten. |
| `Vitrine`       | The public display case: stable payloads only, never unstable internal plates.               |

Supporting vocabulary:

- `PlateFamily` — Source, Syntax, Semantic, Projection, Render, Target, Finish.
- `SourceAtlasCoordinate` — the resolved-once Vue-era fact (`v0`..`v3`, `vapor`).
- `SourceAtlasRegistry` — a lane's recorded request (route + fallbacks).
- `AtelierFallback` — a recorded reason a lane took a legacy or reduced path.
- `MarkupDocument` — Patina's zero-copy source/rule view; **kept separate from
  `Rendu`** so lint rules never pay for render semantics.

## Relief and Croquis are different layers

The boundary is whether a fact is present in the source tree or derived by
analysis.

| Question            | Relief (`vize_relief`)                | Croquis (`vize_croquis`)                                     |
| ------------------- | ------------------------------------- | ------------------------------------------------------------ |
| What does it model? | Syntax nodes and source locations     | Meaning and relationships derived from syntax/OXC            |
| Identity            | A node's textual name and span        | Which declaration or component that name resolves to         |
| Scope               | Nested syntax shape                   | Lexical scopes, bindings, captures, and undefined references |
| Reactivity          | Expressions as written                | Reactive sources, effects, losses, and dependency edges      |
| Control flow        | `v-if` / `v-for` syntax nodes         | Control-flow facts and graphs requested by an analysis       |
| Lifetime            | Source-faithful arena nodes           | Demand-shaped summaries, tables, overlays, and graphs        |
| Typical consumers   | parser, formatter, syntax-aware rules | linter semantics, typechecker, LSP, compiler optimizations   |

Transforming or normalizing a Relief node does not make it Croquis. Croquis
begins when Vize assigns identity or derives a relationship that was not
explicitly represented by the source syntax.

## Physical crate boundaries

| Crate               | Owns                                                             | Must not become                       |
| ------------------- | ---------------------------------------------------------------- | ------------------------------------- |
| `vize_atlas`        | Neutral request/capability/fallback ledger                       | AST, semantic engine, or emitter      |
| `vize_relief`       | Source syntax, locations, errors, options                        | Symbol table or dependency graph      |
| `vize_croquis`      | Semantic facts, scopes, symbols, call/effect/control-flow graphs | Render IR or code generator           |
| `vize_croquis_cf`   | Opt-in cross-file aggregation and project graph facts            | Source AST or mandatory compiler pass |
| `vize_rendu`        | Borrowed output-facing render operations and sections            | General-purpose toolchain IR          |
| `vize_atelier_core` | Shared transforms and JavaScript emission                        | Owner of all compiler infrastructure  |

## Plate-request matrix

Which lane may request which plate, and whether the request is cheap on the
normal path or built only on demand. "—" means the lane does not request that
family.

| Lane            | Source | Syntax | Semantic | Projection | Render      | Target | Finish  |
| --------------- | ------ | ------ | -------- | ---------- | ----------- | ------ | ------- |
| Compiler        | cheap  | cheap  | reuse    | —          | on demand   | yes    | yes     |
| Linter (Patina) | cheap  | cheap  | reuse    | rule-gated | —           | —      | —       |
| Typecheck       | cheap  | cheap  | reuse    | on demand  | —           | —      | maps    |
| Language server | cheap  | cheap  | reuse    | on demand  | —           | —      | maps    |
| Formatter       | cheap  | cheap  | feature  | —          | —           | —      | —       |
| Inspector       | cheap  | reuse  | reuse    | on demand  | on demand   | view   | view    |
| Playground      | cheap  | reuse  | reuse    | on demand  | on demand   | yes    | yes     |
| Source map      | cheap  | —      | —        | —          | range marks | —      | yes     |
| Bundler         | cheap  | —      | reuse    | —          | on demand   | yes    | yes     |
| Vitrine         | —      | —      | —        | —          | —           | —      | payload |

The negative entries are the point: lint, format, and editor lanes stay
render-free, and only render lanes build `Rendu`. The cheap checks behind this
matrix are `SourceAtlasRegistry::requests_render()`,
`requests_projection()`, and `is_render_free()`.

## Cost rules

- Source and Syntax plates are always cheap enough to identify and span-map.
- Semantic facts (`Croquis`) are demandable per fact and reused across lanes.
- Projection (`Virtual TS`) is built only for the requesting typecheck/editor
  surface.
- Render (`Rendu`) is built only for render lanes; owned cloning on the hot
  path needs benchmark evidence.
- Finish (`AtelierOutput`, maps) is structured before flattening — no string
  rescans when the data already exists.

## References

- [Source Atlas](/architecture/source-atlas)
- [#1634](https://github.com/ubugeeei-prod/vize/issues/1634) — design gate.
- [#1692](https://github.com/ubugeeei-prod/vize/issues/1692) — plate registry.
