<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_atelier_vapor.svg" alt="vize_atelier_vapor logo" width="120" height="120" /><br>
  vize_atelier_vapor
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_vapor` compiles Vue templates for Vapor mode.

## Highlights

- Vapor-specific IR generation
- Code generation helpers for direct DOM-oriented updates
- Shared parser and transform lane with the rest of the Vize compiler stack

## Key Entry Points

- `compile_vapor`
- `transform_to_ir`
- `generate_vapor`
- `VaporCompilerOptions`

## Related Crates

- `vize_atelier_core` provides shared steps and parser access
- `vize_atelier_sfc` delegates Vapor template compilation here
- `vize_patina` includes Vapor-oriented lint rules that align with this backend

## License

MIT
