# Published differential Cargo feature compatibility

Decision for [#6832](https://github.com/ubugeeei-prod/vize/issues/6832), 2026-09-29.

The level rename changed the differential test feature to
`legacy-differential`. The prior `davinci-differential` spelling had already
been published in Cargo manifests. Keep it as a deprecated alias in each
affected crate, with exactly the new feature's dependency and `cfg` behavior:

| Published crate | Deprecated feature | Forwarded feature |
| --- | --- | --- |
| `vize_atelier_core` | `davinci-differential` | `legacy-differential` |
| `vize_atelier_dom` | `davinci-differential` | `legacy-differential` |
| `vize_atelier_jsx` | `davinci-differential` | `legacy-differential` |
| `vize_atelier_ssr` | `davinci-differential` | `legacy-differential` |
| `vize_atelier_vapor` | `davinci-differential` | `legacy-differential` |
| `vize_canon` | `davinci-differential` | `legacy-differential` |
| `vize_croquis` | `davinci-differential` | `legacy-differential` |
| `vize_atelier_sfc` | `davinci-differential` | `legacy-differential` |
| `vize_atelier_sfc` | `davinci-dom-differential` | `legacy-dom-differential` |
| `vize_l1` | `davinci-differential` | `legacy-differential` |
| `vize_l1_to_l2` | `davinci-differential` | `legacy-differential` |
| `vize_patina` | `davinci-differential` | `legacy-differential` |

Only the manifest alias retains the old spelling. Rust `cfg` sites and corpus
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
