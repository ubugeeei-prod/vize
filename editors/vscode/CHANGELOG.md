# Changelog

## Unreleased

- Add a Vize status bar entry that opens a setup and troubleshooting action hub.
- Add commands for recommended setup, lint-only setup, server executable selection, and disabling the language server.
- Offer lint-only setup from the first-run prompts and server binary selection when auto-detection fails.
- Highlight RFC 823 `v-when` values as patterns: `const` / `as` declarations, the `_` wildcard, rest, alternatives and guards use the scope names of the reference grammar.
- Fix a less-than comparison in a directive value, such as `v-if="count < limit"`, being read as type arguments, which broke highlighting for the rest of the tag.

## 0.49.0

- Default to opt-in activation for all Vize language server capabilities.
- Add separate lint, typecheck, editor assistance, and formatting switches for incremental adoption alongside `vuejs/language-tools`.
