# TSRX parsing: a neutral markup hook in the oxc fork

Issue: #6858. Implements the L1 placement in the
[TSRX Vue profile record](./2026-11-09-6858-tsrx-vue-profile.md).

## Decision

The vendored parser (`upstream/forks/oxc_parser`) gains one framework-neutral
hook for markup written inside JS/TS: a `MarkupGrammar` that the host calls at
a JSX-position `<`, at an `@` the grammar opens at an expression or statement
start, and at an `@` where a function or arrow body starts. The host keeps an
empty placeholder with the region's exact span, and the grammar re-enters the
same parser through a `MarkupCursor` for every embedded expression, statement,
head, binding and parameter list. While a grammar is registered, a line-start
`<tag` ends the previous expression. The hook sites test a parser-config
constant that is true only for the markup entry point, so every other parse
compiles them out.

All TSRX syntax (tags, attributes, text, comments, raw bodies, statement
containers, directives and the `TSRX` diagnostic codes) lives in
`vize_l1::markup::tsrx`. The fork knows no TSRX name or rule.

## Why not scan in L1 and hand JS spans to a stock parser

1. Finding where markup starts needs the host grammar's expression-position
   state: regular expression versus division, TypeScript type contexts, and
   `<T,>` generics versus JSX. A pre-scan in L1 would duplicate that lexer and
   parser state and drift from it.
2. The end of an embedded `{ … }` is unknown until its JavaScript is parsed:
   strings, regular expressions, template literals and nested markup all hide
   braces. The fork's existing embedding API needs the content extent up front.
3. Markup nested in embedded code (`{items.map((i) => <li>{i}</li>)}`) needs
   re-entrancy into the same parser; a one-shot stock parse cannot provide it.

The pinned reference compiler makes the same choice: `@tsrx/core` is an acorn
plugin that extends the JavaScript parser itself.

## Constraints

- The hook stays neutral and small; the patch is listed in the fork's
  `UPSTREAM.md` so an upstream upgrade (#8987) carries it.
- `parse_with_markup` returns an ordinary parse result and never mints
  `ProgramObservation` admission, because its program holds placeholders that
  only the grammar's artifact can interpret.
- The no-grammar path must stay cost-free; the introducing change records a
  base-versus-head parse benchmark (instructions retired within noise).
