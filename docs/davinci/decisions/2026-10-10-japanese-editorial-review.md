# Japanese editorial review: first reader-facing slice

Issue: [#8372](https://github.com/ubugeeei-prod/vize/issues/8372).

This slice reviews seven Japanese pages: Stability, Diagnostic Types,
Developer Workflows, Comment Annotations, Oxlint, Bundler Integrations and
VS Code. Replace mistranslated product names and technical terms, repair
fragmented sentences, and retain source examples and existing fragments.
Stability now mirrors the current English package and Rust-crate contracts:
`minor` means a minor release, `crate` means a Rust crate, and every API,
support tier and deprecation period remains associated with its original
entry. A regression checks the Japanese contract against the checked
English source rather than permitting stale or incorrectly translated rows.

Translation regeneration previously deleted entire locale directories,
including hand-written and edited translations. Human-authored documents
and documents with `<!-- Reviewed translation; ... -->` are retained as
complete generation inputs. Partial reviews may specify `scope:` in that
marker without claiming the rest of the page was reviewed. Source paths
removed from English are not carried forward. The explicit
`--overwrite-reviewed` option allows a deliberate replacement; ordinary
generation preserves editorial work. `--normalize-only` still repairs
Markdown boundaries while preserving literal source.

## Remaining review

The site-wide #8372 acceptance remains open. The separate More-guides slice
reviews introductions and reading order for eleven other English/Japanese
topics, and navigation and benchmark improvements have separate owners.
The remaining Japanese core bodies, dated blog posts and generated rule
descriptions still require a full reading review; terminology scans and
native rendering alone do not prove that prose is natural. Exact-head
Actions, SSG and deployed reading checks remain required for this slice.
