# CSS utility-class literal token ownership

Issue: [#7980](https://github.com/ubugeeei-prod/vize/issues/7980).

The existing `css/no-utility-classes` rule searches the full inline stylesheet
for its exact and digit-prefix utility patterns. The reported `.pl-3` comment
and `.flex` string value therefore produce findings even though they define
no utility selector.

Use the already-declared `cssparser` tokenizer to identify complete comments,
quoted strings and URL tokens, entering nested blocks explicitly. Exclude
only matches inside those token ranges. Keep the existing raw heuristic,
pattern tables, order, boundary checks, diagnostic text and whole authored
spans everywhere else. Invalid-string/URL token bytes are literals too; the
existing stylesheet parser still decides whether CSS rules run. No new
dependency, public API, pipeline stage or selector-policy expansion is added.

Retain the complete original issue body, both unchanged SFC blocks and JSON
configuration. Ten pinned whole inputs cover the originals, CRLF, escaped
quotes, nested functions/blocks, URL text, true selectors, existing boundary
nonmatches, Unicode, multiple styles and existing inline disable behavior.
Complete API/JSON/plain output vectors were authored before linter execution.
The existing automatic Rust and tooling suites must qualify all ten API rows
and thirty complete CLI processes, with raw failure output retained and zero
unproven native handling credit. Historical goldens and instruction caps stay
unchanged.

Independent technical source review qualified the private source and all
original vectors; it did not execute the product. After the verified v0.434
publication, the release owner lifted the admission hold in
[#6239](https://github.com/ubugeeei-prod/vize/issues/6239#issuecomment-6006766624).
The unchanged behavioral source and corpus are composed with signed main
`143c1d4a9fbe6530cdd06f1f001b816f39c1d00a` for fresh hosted qualification.

TODO: fresh exact-head hosted source/full Rust and whole CLI qualification;
protected merge-queue qualification and actual signed merge; next official
release inclusion and installed-public original replay.

The preceding PR head `4c514a7cc23ebac44359597cc6b9bf781fe2eb85`
qualified all ten original API vectors and thirty complete CLI processes. Its
aggregate stayed red on the shared dependency audit, so it was kept off queue.
Security PR #8080 actually merged as signed-valid
`48cb1d4f35ebd0aa3824817c67752b5e4d0a9dbb` at 2026-10-06T05:04:12Z.
Compose that literal main through a normal merge, conserving all owned behavior,
original inputs/oracles and incoming dependency/workflow/strict UI policy bytes.
This grants no fresh-head execution credit: rerun ordinary Actions and the full
protected suites before independent matched squash auto, actual signed merge
and next installed-public release verification.
