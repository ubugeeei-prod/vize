# Stop pinning live formatter witness sources

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The formatter differential helpers recorded where each expectation came from
(a test in `crates/vize_glyph`, a CLI manifest) and then pinned the current
bytes of those live files: whole-file SHA-256 values for the current Rust law
owners, the JSON layout and block-indent owners, the attribute-width owner and
the shared formatter fixture manifest. A neutral edit to any `vize_glyph`
source failed twelve formatter tooling tests, and adding a formatter fixture
failed the CLI union test.

Recorded fixtures under `tests/_fixtures/` stay immutable and keep their hash
checks, as do the preserved original assets. For live files the helpers now
check what the witness is about: the original law function still exists, a
registered law keeps its retained body, and the original thirteen CLI cases
stay first and unchanged in the shared manifest. Whole-file current hashes,
the exact current case count and the custody refusals that exercised them are
removed. With a newline appended to every `vize_glyph` source and `npm/cli`
file, the formatter tooling tests fail exactly as they do on `main`.
