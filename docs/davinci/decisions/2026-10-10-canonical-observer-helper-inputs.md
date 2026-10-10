# Select actual canonical observer helper inputs

Tracking: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), with the
[paired decision](https://github.com/ubugeeei-prod/vize/issues/6830#issuecomment-6094042385).

The source-selector audit following [#8379](https://github.com/ubugeeei-prod/vize/pull/8379)
found four actual observer inputs absent from the canonical selector:

- `canonical-corpus-native-walk.mjs` authenticates and executes the original
  Rust collector and validates its complete ordered vector.
- `canonical-corpus-logical-walk.mjs` supplies the committed symlink graph and
  exact owned kernel-boundary checks used by that native walker.
- `canonical-corpus-io-probe.mjs` runs the required complete source IO probe.
- `canonical-corpus-io-probe.rs` is read and compiled into that probe's actual
  Rust executable.

All four live in `tools/support/compat/github/`. At the unchanged #8379 head,
each individual path returned `canonicalCorpusRequired=false`. Tooling checks
alone cannot establish the complete canonical result after changing these
producers. Select these exact helper paths in both PR and merge-queue contexts;
the existing source-selection law now covers each path.

Keep the genuine #8379 qualification separate. This follow-up changes only
selection and its controls. Readers, logical aliases, full vector/source bytes,
oracles, observer outcomes, aggregate rejection and resource ceilings remain
unchanged. It adds no validation layer or product pipeline stage.

TODO: obtain this follow-up's exact-head Actions and protected actual merge.
The four helper selectors remain undelivered until then. Whole #6830 and
#7951 runtime/release closure are not inferred from this bounded correction.
