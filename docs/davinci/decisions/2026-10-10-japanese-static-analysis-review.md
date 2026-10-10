# Japanese static analysis reading review

Related issue: [#8372](https://github.com/ubugeeei-prod/vize/issues/8372).

Review the entire Japanese static analysis guide against the English source,
including the pipeline, presets, migration, cross-file scope, type checking,
runtime choices, Oxlint, and adoption order. Keep product names and developer
terms recognizable: lint, Vapor, Musea, Patina, provide/inject, reactivity, and
Composition API. Repair incomplete pipeline table cells and fragmented prose.
The lower-level cross-file inventory remains distinct from currently exposed
CLI features; no new product support is claimed.

Preserve all twenty complete original source recipes and eleven original
native heading targets. Old Japanese bookmarks remain as invisible aliases,
while headings use natural technical Japanese. Mark the whole document as
reviewed so the translation generator retains this authoring input.

The English and Japanese runtime description also removes the obsolete claim
that the Rust CLI exposes more checker commands than the npm package. Actual
`npm/cli/src/cli.ts` configures the bundled Corsa runtime and calls native
`runCli`; `crates/vize_vitrine/src/napi/cli.rs` forwards to
`vize::cli::run_from_args`. Explicit public runtime environment settings retain
precedence in `npm/cli/src/corsa-runtime.ts`. This is an installation choice,
not a different command surface.

Focused native rendering regressions retain complete recipes, original
bookmarks, rule identities, and the three pipeline table cells. A bounded Docs
browser step captures English/Japanese desktop/mobile reading flows before
the inherited all-site navigation and Open Graph gates. Protected native
Stack delivery and the deployed reading check remain required; the rest of
the site-wide Japanese editorial issue remains unfinished.
