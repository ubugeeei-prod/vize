---
title: Analysis Diagnostics
---

# Analysis Diagnostics

This page explains how Vize diagnostics are organized. The detailed rule reference now lives in the
Rules section so each rule can keep its behavior, default severity, preset coverage, and Bad/Good
examples together.

## Rule Reference

- [Rules overview](../rules/index.md)
- [Vue rules](../rules/vue.md)
- [Accessibility rules](../rules/accessibility.md)
- [Type and script rules](../rules/type-and-script.md)
- [HTML rules](../rules/html.md)
- [SSR rules](../rules/ssr.md)
- [Vapor rules](../rules/vapor.md)
- [Cross-file analyzer rules](../rules/cross-file.md)
- [Musea and CSS rules](../rules/musea-and-css.md)

## Diagnostic Families

Patina rules are single-file lint rules. They use names such as `vue/require-v-for-key` and can be
configured from `vize.config.*`, the CLI, the JavaScript API, and the Oxlint bridge.

Cross-file analyzer diagnostics use `vize:croquis/cf/*` codes. They are emitted by
`vize lint --cross-file` after Vize builds a project graph, so they can compare providers with
injectors, track duplicated IDs, and spot reactivity hazards across component boundaries.

Type-aware diagnostics use the TypeScript checker. They need the same project configuration that
TypeScript sees through `tsconfig.json`, including `compilerOptions.types`, `paths`, and project
references. Vize does not require a separate `globals` list for those names.

Musea and CSS diagnostics are library-backed rules. They run when Musea art blocks or style content
are parsed and are documented separately because they are not part of the standard Vue template rule
surface.
