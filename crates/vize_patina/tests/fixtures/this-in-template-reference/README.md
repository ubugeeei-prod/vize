# `this-in-template` token references

Issue [#9224](https://github.com/ubugeeei-prod/vize/issues/9224) was exposed by
[Real Project Matrix 38890448622](https://github.com/ubugeeei-prod/vize/actions/runs/38890448622)
on source `d2506ca6b5e31252973f05981829c0367cacf26b`.

`original-matrix.json` retains all 97 original findings and the identities of the
six complete raw packets. The immutable Actions artifacts and retained local
packets remain the full historical evidence, including unrelated rule failures.
Three complete pinned project sources reproduce repeated tokens, multiline
handlers and a template-literal substitution. Source URLs, revisions and SHA256
digests are retained per original source; no upstream file was changed.

`cases.json` records an independent execution of the pinned ESLint 10.4.1,
eslint-plugin-vue 10.9.2, vue-eslint-parser 10.4.1 and TypeScript parser 8.65.0
baseline. It retains complete messages, suggested fixes, verdict counts and
exact authored UTF-8 token ranges. The adjacent reference files and MIT license
retain the actual installed rule and reserved-word authority. The Node test
replays the baseline; the Rust integration test requires the product's ranges
and full diagnostic fields to agree, without any comparator normalization.

Controls cover computed and optional accesses, nested template literals,
statement and return handlers, CRLF, Unicode, HTML entities, reserved names,
scoped aliases and non-code text. Existing template comment suppression is
tested independently, since the reference processor does not honor that same
HTML comment contract. Candidate Actions must execute the native integration
tests; fixture preparation and historical failures do not qualify the fix.
