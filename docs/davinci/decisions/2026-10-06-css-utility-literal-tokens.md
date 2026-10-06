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

TODO: independent source review; fresh exact-head hosted source/full Rust and
whole CLI qualification; protected merge-queue qualification and actual signed
merge; next official release inclusion and installed-public original replay.
While v0.434 is publishing, preparation stays private and no new PR or queue
admission is authorized until the release owner explicitly lifts that hold.
