<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_glyph.svg" alt="vize_glyph logo" width="120" height="120" /><br>
  vize_glyph
</h1>

`vize_glyph` formats Vue Single File Components.

## Highlights

- Whole-file SFC formatting
- Block-level formatting helpers for template, script, and style content
- Prettier-like options with Vue-specific attribute and block ordering controls
- An opt-in arena-backed document IR with an iterative layout printer

## Key Entry Points

- `format_sfc`
- `format_sfc_with_allocator`
- `format_template`
- `format_script`
- `format_style`
- `FormatOptions`

The opt-in native document pipeline keeps syntax traversal and layout separate:

- `native_doc::template_document` builds a source-borrowing `Doc` from a retained L1 parse
- `native_doc::print` lays out that `Doc` according to `PrintOptions`

These APIs do not change the default formatter entry points above.

## Related Crates

- `vize` exposes Glyph through `vize fmt`
- `vize_maestro` can use Glyph for LSP formatting
- `vize_atelier_sfc` and `vize_l0` provide parsing and allocation support behind the formatter

## License

MIT
