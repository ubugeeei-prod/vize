# prettier-plugin-vize

Use Vize Glyph to format Vue SFCs and Art files through Prettier.

Not yet published to npm: it is an experimental workspace package until it joins
the release.

```sh
npm install --save-dev --save-exact prettier@3.9.10 prettier-plugin-vize
```

Add the plugin to your Prettier configuration:

```json
{
  "plugins": ["prettier-plugin-vize"],
  "printWidth": 100,
  "singleQuote": true,
  "quoteProps": "preserve"
}
```

```sh
prettier --write "src/**/*.{vue,art}"
prettier --check "src/**/*.{vue,art}"
```

The plugin infers `vize` for `.vue` and `vize-art` for `.art`. Explicit
`--parser vize` also works; `vue` is an alias for editor configurations.
Prettier owns configuration discovery, `.prettierignore`, file selection,
line-ending emission and cursor tracking. It does not read `vize.config`.

## Options

Glyph's existing compatibility preset is the base. The following Prettier
options override it directly:

`printWidth`, `tabWidth`, `useTabs`, `semi`, `singleQuote`, `jsxSingleQuote`,
`quoteProps`, `objectWrap`, `trailingComma`, `arrowParens`, `bracketSpacing`,
`bracketSameLine`, and `singleAttributePerLine`.

`endOfLine` supports `lf`, `crlf`, `cr` and `auto`. Prettier normalizes input
before parsing; the native pass uses LF and a literal-line Doc lets Prettier
emit the requested ending without adding indentation or trimming literal text.
The native formatter runs once per parsed file. Cursor offsets use Prettier's
UTF-16 API, including Unicode and CRLF input.

`vizeVueVersion` accepts `"2"` or `"3"` (default). Block, attribute and import
sorting remain disabled.

Use `quoteProps: "preserve"` with Vize's
`script/consistent-props-emits-key-quotes` lint rule. This leaves the
props/emits-specific convention to lint and prevents format-on-save from
removing required quotes without changing unrelated object literals.

`objectWrap: "preserve"` (default) keeps an object multiline when a newline
follows its opening `{`. Other objects collapse when they fit the print width.
`objectWrap: "collapse"` puts objects on one line when they fit the print width;
comments and width constraints can still require multiple lines.

The shared script formatter supports `printWidth` from 1 to 320 and
`tabWidth` from 0 to 24. Other values produce an error. Whole-file formatting
is supported; partial `rangeStart`/`rangeEnd` formatting produces an error.

These options accept only their listed defaults because the native API has no
equivalent override: `htmlWhitespaceSensitivity: "css"`,
`vueIndentScriptAndStyle: false`, `embeddedLanguageFormatting: "auto"`,
`proseWrap: "preserve"`,
`experimentalTernaries: false`, and `experimentalOperatorPosition: "end"`.
An unsupported value produces an error instead of being silently ignored.

## Ignore directives and pragmas

Use `.prettierignore` to exclude files. Prettier's `--require-pragma` and
`--insert-pragma` work with leading `<!-- @format -->` or
`<!-- @prettier -->` comments. With `--check-ignore-pragma`, leading
`<!-- @noprettier -->` and `<!-- @noformat -->` comments skip the file.

JavaScript `// prettier-ignore` comments follow the shared script formatter's
existing behavior. HTML node, attribute and range `prettier-ignore` directives
and CSS ignore regions are unsupported and produce an error. Comment-like
strings and attribute values remain data.

## Art support

The plugin uses the same SFC formatter for `.art`: script and style blocks,
the existing `<art>` container and its Vue-like variant markup. Other custom
block bodies remain opaque under the existing engine's behavior. This does
not introduce formatting support for additional Art engines or languages.

Prettier's debug AST check observes this adapter's formatted document; it is
not an independent Vue or Art semantic validator. The package's consumer tests
separately compare parsed script/style structure and actual Vue rendering.

## Application conformance

The source-native consumer workflow installs the packed public adapter in an
isolated project and formats every SFC from the Vite Musea and JSX/TSX example
applications. The shared native formatter supplies the exact output oracle.
Installed API output, CLI writes, repeat formatting, and rejected-option writes
are checked, and complete source/output packets must agree on Node 22.0.0 and
24.14.0 using the same adapter tarball and authenticated native image.

The workflow retains per-file hashes, complete inputs and outputs, and observed
API/native/CLI timings. This covers the listed application sources and host
versions; the package remains experimental on the formatter's independent v0 line.
