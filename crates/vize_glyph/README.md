# vize_glyph

`vize_glyph` formats Vue Single File Components.

## Highlights

- Whole-file SFC formatting
- Block-level formatting helpers for template, script, and style content
- Prettier-like options with Vue-specific attribute and block ordering controls

## Key Entry Points

- `format_sfc`
- `format_sfc_with_allocator`
- `format_template`
- `format_script`
- `format_style`
- `FormatOptions`

## Related Crates

- `vize` exposes Glyph through `vize fmt`
- `vize_maestro` can use Glyph for LSP formatting
- `vize_atelier_sfc` and `vize_carton` provide parsing and allocation support behind the formatter

## License

MIT
