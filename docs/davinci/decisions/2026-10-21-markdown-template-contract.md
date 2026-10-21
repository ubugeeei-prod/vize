# Markdown template contract

Decision for [#8909](https://github.com/ubugeeei-prod/vize/issues/8909).

## Decision

Ox Content owns Markdown parsing, frontmatter, GFM features and Markdown-to-HTML rendering. Vize consumes the generated module contract rather than parsing Markdown again. The contract is identified by Ox Content's generated-module marker and these JSON-valued named exports:

- `html`: rendered template HTML;
- `frontmatter`: the parsed frontmatter object;
- `toc`: the generated table of contents.

Vize turns that data into an SFC and sends the SFC through its normal native DOM or SSR compiler. The generated component retains the three named exports and returns `frontmatter` and `toc` from `setup()`, so Markdown interpolation can read them and the default component exposes them on its public instance.

The initial slice applies to `.md` modules after `@ox-content/vite-plugin` has transformed them. It deliberately requires the generated marker and valid JSON exports; unrelated JavaScript and malformed producer output pass through unchanged.

The measured generated-module contract is qualified at the real Vite boundary in DOM, SSR and Vapor modes. The build fixture keeps named `html`, `frontmatter` and `toc` exports, renders frontmatter interpolation, and proves fenced Vue expressions never become component-context reads.

For `<template lang="md">`, the SFC container owns only the surrounding block and records the complete Markdown body as a borrowed source slice. Ox Content owns frontmatter delimiters and values, code-fence classification, and every other byte in that slice. Until an explicit Markdown formatter exists, Glyph copies the body byte-for-byte: it does not trim blank lines, rebase indentation, or normalize line endings. The surrounding SFC tags and layout remain SFC formatter syntax. This custody contract does not by itself compile the Markdown block; the later preprocessor slice must obtain rendered output from Ox Content first.

## Vue syntax and inert content

Vue interpolation and directives are recognised only after Ox Content has rendered Markdown to HTML. Raw `{{ expression }}` and Vue directive attributes that survive that rendering are therefore compiled by Vize in the same way as an ordinary SFC template.

Fenced code remains inert because Ox Content owns fence recognition and HTML-escapes tag delimiters before Vize sees the template, while the adapter entity-escapes `{` and `}` only inside generated `<pre><code>` blocks. Browsers render the original braces, while the Vue parser never sees an interpolation delimiter. Vize must never recover or compile Vue syntax from that escaped payload, and it must not infer fence boundaries from authored Markdown itself.

Case-preserving embedded Vue components require a structured Ox Content component/island result. The current generated HTML contract lowercases custom HTML tag names, so this slice does not claim embedded-component support. Vize will consume that structured field once Ox Content publishes it as part of the framework transform result; guessing component identity from lowercased HTML is rejected.

## Source positions

Ox Content's generated module currently has `map: null`. The adapter therefore cannot produce an authored-Markdown source map in this slice. The source-position contract will use Ox Content source spans for rendered nodes and compose them with Vize's template map. Diagnostics, HMR and editor features must use the composed map; generated HTML offsets are not accepted as authored positions.

## Delivery slices

1. Consume real Ox Content generated modules as `.md` Vue components in Vite, preserving named frontmatter and TOC exports. Cover DOM, SSR and Vapor compilation and keep fenced Vue examples inert.
2. Add the structured component/island field upstream and consume it without losing component casing, props, directives or interpolation.
3. Add composed source maps and use them for HMR, compiler diagnostics, linter diagnostics and type checking.
4. Preprocess `<template lang="md">` through the same contract, then cover DOM, SSR and Vapor output.
5. Keep Markdown bytes unchanged in the formatter (the payload custody contract is delivered independently) and add LSP completion, hover and definition mapping through the composed spans.
6. Migrate the documentation site to component imports and publish the runnable guide after every preceding acceptance surface is green.

The issue remains open until all six slices and their differential fixtures pass.
