# vize_musea

`vize_musea` is the Rust core for Musea art files, docs, palette generation, autogen, and VRT
support.

## Highlights

- Parse `*.art.vue` files into `ArtDescriptor`
- Transform art files into Vue and Storybook-compatible outputs
- Generate Markdown docs, catalogs, and prop palettes
- Autogenerate art variants and VRT configuration data

## Key Entry Points

- `parse_art`
- `transform_to_csf`
- `transform_to_vue`
- `docs::{generate_component_doc, generate_catalog}`
- `palette::generate_palette`
- `autogen::generate_art_file`

## Notes

The gallery UI and dev-server integration live in the JavaScript package
`@vizejs/vite-plugin-musea`. This crate focuses on the Rust-side parsing and generation pipeline.

## Related Crates

- `vize_vitrine` exposes Musea functionality to Node.js and WASM
- `vize_patina` includes Musea-specific lint rules
- `@vizejs/musea-mcp-server` consumes Musea metadata for AI integrations

## License

MIT
