# vize_atelier_vapor

`vize_atelier_vapor` compiles Vue templates for Vapor mode.

## Highlights

- Vapor-specific IR generation
- Code generation helpers for direct DOM-oriented updates
- Shared parser and transform pipeline with the rest of the Vize compiler stack

## Key Entry Points

- `compile_vapor`
- `transform_to_ir`
- `generate_vapor`
- `VaporCompilerOptions`

## Related Crates

- `vize_atelier_core` provides shared transforms and parser access
- `vize_atelier_sfc` delegates Vapor template compilation here
- `vize_patina` includes Vapor-oriented lint rules that align with this backend

## License

MIT
