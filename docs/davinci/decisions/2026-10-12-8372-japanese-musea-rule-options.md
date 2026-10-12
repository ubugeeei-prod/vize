# Japanese Musea rule options

Refs [#8372](https://github.com/ubugeeei-prod/vize/issues/8372) and
[#8365](https://github.com/ubugeeei-prod/vize/issues/8365).

Add a complete authored Japanese version of `rules/options-musea.md` so a
reader can connect the configuration to the existing Bad/Good examples.
Preserve all three original fenced examples, the original API tokens,
the relative Rule Options link, and both English bookmarks.

Explain the boundary between selection and token data. The CLI implicitly
selects `musea/prefer-design-tokens` when its token list is non-empty, unless
that rule is explicitly disabled. The Rust builder supplies the token
inventory; it does not enable the rule. Add the explicit `off` precedence
sentence to the English guide, without changing any configuration recipe.

The implementation authorities are
[`entry_rules.rs`](../../../crates/vize/src/commands/lint/entry_rules.rs)
and [`musea_config.rs`](../../../crates/vize_patina/src/linter/musea_config.rs).
The examples retain their current Vite Plus configuration and do not
introduce a new API, default rule, preset or installed support claim.

Mark only this guide as available in both locales in the More coverage
ledger. A regeneration law keeps the complete authored Japanese input and
compares every original recipe, ordered inline token and link destination.
Native compiled anchors and actual public EN/JA desktop/mobile reading
remain required after exact-head Actions and the protected merge queue.

This bounded guide does not complete either documentation issue or prove
installed CLI, released Musea or whole-toolchain acceptance. Track those
requirements separately in their existing issues.
