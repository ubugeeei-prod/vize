# Freeze the Markdown-to-template boundary

Status: accepted design for [#8909](https://github.com/ubugeeei-prod/vize/issues/8909).
This record fixes the producer/consumer contract only; it does not claim the
compiler, source-map, tooling or guide acceptance items are implemented.

The earlier [runtime adapter record](./2026-10-21-markdown-template-contract.md)
describes the first compatibility slice over Ox Content's generated JavaScript
module. That adapter remains valid for static HTML. The contract below is the
required boundary for complete Vue syntax and authored-source mapping.

## Consumed Ox Content output

Vize consumes one versioned, framework-neutral `VueTemplateResult` produced by
Ox Content from the same parse that produces its `TransformResult`. Vize does
not parse Markdown, infer fence boundaries, or recover components from rendered
HTML. The minimum result is:

```ts
interface VueTemplateResult {
  version: 1;
  template: string;
  frontmatter: Record<string, unknown>;
  toc: TocEntry[];
  mappings: Array<{
    generatedStart: number;
    generatedEnd: number;
    sourceStart: number;
    sourceEnd: number;
  }>;
}
```

`template` is Vue template source, not browser-normalized HTML. Ox Content owns
Markdown, frontmatter, GFM and MDC/MDX recognition. Vize owns Vue parsing and
DOM, SSR and Vapor compilation of `template`. Both `.md` modules and
`<template lang="md">` use this result, so their syntax cannot drift.

The existing `html`, `frontmatter` and `toc` generated-module exports remain a
compatibility input while Ox Content publishes this result. `html` alone is not
the final contract: it cannot carry case-preserving component structure or a
complete authored-Markdown map. Ox Content's framework-specific runtime
component output is also not the compiler input because it renders HTML through
`innerHTML` instead of exposing a Vue template to Vize.

## Vue syntax

Ox Content classifies syntax from the Markdown parse and emits these authored
regions unchanged in `template`:

- registered PascalCase or kebab-case component elements, including props and
  children;
- `{{ expression }}` interpolation in Markdown text and inline HTML;
- Vue directive attributes such as `v-if`, `v-for`, `v-bind`, `v-on` and their
  shorthand forms on authored elements.

Only those classified regions become Vue syntax. Vize then treats them exactly
like the equivalent regions of an ordinary `.vue` template. Generated Markdown
markup is ordinary static template markup. Unknown component-like text remains
Markdown text; Vize does not guess component identity from a lowercased HTML
tag.

## Inert code

Ox Content owns fenced and indented code classification. Their contents are
emitted as escaped text inside generated `<pre><code>` markup. In particular,
`<`, `>`, `&`, `{` and `}` cannot reach the Vue parser as component,
interpolation or directive syntax. Vize never scans the authored Markdown for
fences and never unescapes a code payload before compilation.

The mapping for a code payload may still point diagnostics or editor selection
back to its authored text, but it must not make the payload executable. Inline
code follows the same inert rule.

## Frontmatter and positions

Ox Content removes YAML frontmatter before producing `template` and returns the
parsed object as `frontmatter`. An imported `.md` module exposes that object as
the named export `frontmatter`, as the template binding `frontmatter`, and on
the default component's public instance. Values remain `unknown` until a
project supplies a schema. `<template lang="md">` does not accept an embedded
frontmatter block; SFC metadata remains owned by the surrounding `.vue` file.

Every generated range that can produce a Vize diagnostic, hover, definition or
source map must have an authored Markdown mapping. Vize composes `mappings`
with its normal template map and reports the `.md` or Markdown block position.
Generated-template offsets are never user-facing positions.

This resolves the design item in #8909. Compiler, Vite, type-checker, linter,
formatter, LSP, fixture and guide delivery remain separate acceptance work.
