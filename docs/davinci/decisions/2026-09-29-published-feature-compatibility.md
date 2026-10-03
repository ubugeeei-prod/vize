# Published differential Cargo feature compatibility

Decision for [#6832](https://github.com/ubugeeei-prod/vize/issues/6832), 2026-09-29.

The level rename changed the differential test feature to
`legacy-differential`. The prior `davinci-differential` spelling had already
been published in Cargo manifests. Keep it as a deprecated alias in each
affected crate, with exactly the new feature's dependency and `cfg` behavior:

| Published crate      | Deprecated feature         | Forwarded feature         |
| -------------------- | -------------------------- | ------------------------- |
| `vize_atelier_core`  | `davinci-differential`     | `legacy-differential`     |
| `vize_atelier_dom`   | `davinci-differential`     | `legacy-differential`     |
| `vize_atelier_jsx`   | `davinci-differential`     | `legacy-differential`     |
| `vize_atelier_ssr`   | `davinci-differential`     | `legacy-differential`     |
| `vize_atelier_vapor` | `davinci-differential`     | `legacy-differential`     |
| `vize_canon`         | `davinci-differential`     | `legacy-differential`     |
| `vize_croquis`       | `davinci-differential`     | `legacy-differential`     |
| `vize_atelier_sfc`   | `davinci-differential`     | `legacy-differential`     |
| `vize_atelier_sfc`   | `davinci-dom-differential` | `legacy-dom-differential` |
| `vize_l1`            | `davinci-differential`     | `legacy-differential`     |
| `vize_l1_to_l2`      | `davinci-differential`     | `legacy-differential`     |
| `vize_patina`        | `davinci-differential`     | `legacy-differential`     |

Manifest aliases and the published SFC benchmark composite retain the old
spelling. Rust `cfg` sites and corpus
target requirements use the new feature, so enabling either spelling arms the
same comparator and required test targets. The SFC DOM alias is separate from
the retained-AST differential lane, as it was before the rename. All aliases
stay off by default and do not change product output or serialized strings.

For alpha/preview crates, the [support policy](../../release/support-policy.md)
requires a full minor release with the manifest alias and a release note before
removal. `vize_atelier_vapor`, `vize_l1` and `vize_l1_to_l2` are experimental,
but keep the same alias as a courtesy to existing Cargo consumers. Cargo
features cannot carry a Rust `#[deprecated]` attribute; the manifest comments,
this decision and the release note are the deprecation notice. Internal level
naming remains the goal of #6832; a published build option is a temporary compatibility
boundary, not a new internal crate/module or a wire-format exception.

## Existing PR #7213 residual repair

The accepted main `574002ec` retains the published DOM alias in
`davinci-production-bench`, but enables only `vize_atelier_ssr/legacy-differential`.
The SSR alias forwards the old name to the new one; enabling the new name does
not enable the old published feature. This repair adds the old SSR edge beside
the new one. All normal level dependencies and product output remain unchanged.
The independent pending SFC setup adapter's real normal L2 edge must be preserved
when this own-only feature change is replayed onto its merged source.

The differential rename tool is now TypeScript. It preserves exactly one
published benchmark block and exactly one occurrence of each published edge,
while still renaming stray selectors. Missing or duplicate blocks or members
fail before any tracked file is written. Tests assert the complete protected
edges after rewriting, idempotence, failed-write atomicity and actual locked
Cargo feature declarations. This addresses the retained #7213 review threads
`PRRT_kwDOQvjuaM6nB6Mz` and `PRRT_kwDOQvjuaM6nB6ND` without dismissing their
reviews. No new Python source is introduced. The unchanged migration scanner
updates only thirteen manifest row coordinates; its 368-line SFC surface stays
unchanged.

The old #7220 tokenizer compatibility source already entered main through
#7269. The maintainer closed #7220 as superseded at 2026-10-03 07:15:04 UTC,
with its original `dd127028` source unchanged and not merged. The
[closure comment](https://github.com/ubugeeei-prod/vize/pull/7220#issuecomment-5966675899)
preserves the original branch, seven exact #7269 source blobs and review TODO.
Its unresolved interpolation witness finding remains valid: calling the
default callback directly populated the counter before tokenizing input with no
interpolation. The shared baseline/current witness now asserts that direct API
call separately, resets its counter and tokenizes an original `{{value}}`
input. Its final count must come from tokenization. This preserves all public
methods, state discriminants, constants, entity and error checks.

Sixteen local source commands pass, including seven feature laws, nine existing
SemVer workflow policy laws and the existing shell dispatch law. The first run's
generated coordinate mismatch and missing borrowed TOML dependency are retained
and corrected. Three unchanged Rust-script baseline tests remain locally
blocked by `Operation not permitted`; they are not counted as passed or weakened.
The shared tokenizer witness passes through the actual current public reexport
and identical retained lexer sources; omitting tokenization makes its final
interpolation count fail. This selected-module proof does not execute the old
published package or the whole current Armature crate.

The final source candidate replays only these nine paths onto accepted
`7f1c7d4f`, after the original Program invocation/comment prefix actually
merged. All functional source and the thirteen derived coordinate updates
remain identical to the reviewed `640104f6` preparation. Incoming TS,
native, security and zero-Vue reporter changes stay intact. The existing
#7213 branch still requires an exact original-head guard before publication;
no new PR or Stack is opened for this residual repair.

These are source and qualified selected-module checks, not release or whole
published-baseline acceptance. Exact-head Actions, actual feature-enabled Cargo
and published-consumer execution, unchanged instruction gates and protected
terminal merge remain required. Existing failed campaigns and review findings
are preserved; no product default or fix-history issue closes here.

## Actual SFC composition for the same PR

SFC #7497 actually merged at 2026-10-03 08:53:50 UTC as signed
`21f3e560`. The exact `313df437` feature repair passed required, full and
genuine instruction Actions, but its queue composition conflicted only in the
generated SFC migration shard. The Cargo manifest and central decisions merged
cleanly. The unchanged scanner regenerates all 375 incoming data rows, shifting
only fourteen manifest coordinates by one line; the real normal L2 dependency
and every incoming setup, capture, template and runtime source remain intact.
The feature tool, seven laws and shared tokenizer witness are byte-identical
to the reviewed and tested source. No new PR or default path is introduced.

The exact `313df437` hosted logs also pass all four baseline tooling assertions,
including three that were locally sandbox-blocked. The release-only SemVer job
was skipped on this normal branch, so neither external tokenizer Cargo consumer
was executed there. Preserve that limit and the earlier selected lexer proof.
Six bounded union commands pass, including unchanged feature laws, canonical
scanner, locked metadata and source caps. The same-PR successor still requires
its own configured checks, genuine instruction run and protected actual merge;
old-source green results do not establish successor acceptance.

## Consolidated decision placement

The exact 5392c1bd configured checks and genuine instruction campaign passed.
Its prospective queue composition then conflicted only between our tokenizer
qualification paragraph and an adjacent incoming style-custody paragraph.
The same #7213 source now moves all of that qualification, without shortening
or removing any decision, into its existing owned published-feature paragraph.
Every other incoming decision line, including the tokenizer and container
paragraphs, stays exact. The companion retains every earlier proof and failure.

This is an isolated source composition on accepted signed 9f42382f. All owned
functional source remains byte-identical to 5392c1bd; neither old green
checks nor selected tokenizer runtime evidence establishes this successor's
acceptance. Fresh configured checks, one genuine instruction campaign and
protected actual merge remain required. External published/current tokenizer
consumers remain unexecuted on the ordinary branch; no product default or
fix-history gate closes.
