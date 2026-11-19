<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_glyph.svg" alt="vize_glyph logo" width="120" height="120" /><br>
  vize_glyph
</h1>

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
- `vize_l1` splits SFC blocks and owns the markup syntax; `vize_l0` provides allocation support

## License

MIT
