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
  "singleQuote": true
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
`quoteProps`, `trailingComma`, `arrowParens`, `bracketSpacing`,
`bracketSameLine`, and `singleAttributePerLine`.

`endOfLine` supports `lf`, `crlf`, `cr` and `auto`. Prettier normalizes input
before parsing; the native pass uses LF and a literal-line Doc lets Prettier
emit the requested ending without adding indentation or trimming literal text.
The native formatter runs once per parsed file. Cursor offsets use Prettier's
UTF-16 API, including Unicode and CRLF input.

`vizeVueVersion` accepts `"2"` or `"3"` (default). Block, attribute and import
sorting remain disabled.

The shared script formatter supports `printWidth` from 1 to 320 and
`tabWidth` from 0 to 24. Other values produce an error. Whole-file formatting
is supported; partial `rangeStart`/`rangeEnd` formatting produces an error.

These options accept only their listed defaults because the native API has no
equivalent override: `htmlWhitespaceSensitivity: "css"`,
`vueIndentScriptAndStyle: false`, `embeddedLanguageFormatting: "auto"`,
`proseWrap: "preserve"`, `objectWrap: "preserve"`,
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
