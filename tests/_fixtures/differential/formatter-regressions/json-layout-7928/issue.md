# JSON layout regression #7928

Source: <https://github.com/ubugeeei-prod/vize/issues/7928>, reported by
`ubugeeei` against Vize 0.432.0.

The `reported-config` case retains the report's complete `config.json` input.
`vize fmt --write --no-config config.json` must leave its short arrays and
objects collapsed and preserve the blank line before `nested`.

All expected whole outputs were independently captured with the repository's
cached Oxfmt 0.63.0 API, using strict JSON and JSONC with `trailingComma: "none"`.
The controls cover source-expanded objects, collapsed source arrays, member
prefixes and commas at the width boundary, bracket spacing, tab columns,
normalized blank lines, CRLF, JSONC comments, numeric-array fill, Unicode display
width, comment-adjacent gaps and inline block-comment suffixes.

Rust public API and CLI integration tests consume every case and assert whole
output, fixed points or check/write/check behavior. Strict JSON controls also
assert unchanged parsed values. This legacy formatter corpus does not establish
native Davinci formatter support.
