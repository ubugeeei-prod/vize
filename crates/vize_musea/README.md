<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_musea.svg" alt="vize_musea logo" width="120" height="120" /><br>
  vize_musea
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_musea` is the Rust core for Musea art files, docs, palette generation, autogen, and VRT
support.

## Highlights

- Parse `*.art.vue` files into `ArtDescriptor`
- Infer the target from imported component tags and read optional `defineArtMeta(options)` metadata
- Transform art files into Vue and Storybook-compatible outputs
- Generate Markdown docs, catalogs, and prop palettes
- Autogenerate art variants and VRT configuration data

## Art Authoring

The preferred `.art.vue` shape uses ordinary Vue imports and component tags:

```vue
<script setup lang="ts">
import Button from "./Button.vue";

defineArtMeta({
  category: "UI",
  tags: ["button"],
});
</script>

<art>
  <variant name="Primary" default>
    <Button variant="primary">Click</Button>
  </variant>
</art>
```

The target and default title are inferred as `Button`. `defineArtMeta()` is optional, while
`defineArt(Button)` remains an explicit escape hatch for ambiguous compositions.
`<art title="..." component="...">` remains supported for compatibility.

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
