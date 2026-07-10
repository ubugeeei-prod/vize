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
| `Armature`      | The source ledger: files, blocks, spans, parser events, source-map registration marks.       |
| `Relief`        | The source-faithful syntax surface for Vue templates, JSX, and TSX. Not a codegen dump.      |
| `Croquis`       | The semantic study tools ask for: bindings, scopes, components, directives, CSS vars, edges. |
| `Virtual TS`    | A typecheck/editor projection built from Armature/Relief/Croquis — not the canonical IR.     |
| `Rendu`         | The render-semantic plate for DOM/SSR/Vapor. Borrows Relief/Croquis; not the toolchain IR.   |
| `AtelierOutput` | The structured output plate (imports/hoists/functions/exports/sections/maps) before flatten. |
| `Vitrine`       | The public display case: stable payloads only, never unstable internal plates.               |

Supporting vocabulary:

- `PlateFamily` — Source, Syntax, Semantic, Projection, Render, Target, Finish.
- `SourceAtlasCoordinate` — the resolved-once Vue-era fact (`v0`..`v3`, `vapor`).
- `SourceAtlasRegistry` — a lane's recorded request (route + fallbacks).
- `AtelierFallback` — a recorded reason a lane took a legacy or reduced path.
- `MarkupDocument` — Patina's zero-copy source/rule view; **kept separate from
  `Rendu`** so lint rules never pay for render semantics.

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
