# vize_atelier_core

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_atelier_core` contains the shared transform lane and code generation infrastructure used by the
DOM, Vapor, and SSR compilers.

## Highlights

- Core lane and step APIs
- Shared Vue template code generation
- Runtime helper resolution
- Re-exports for the Relief AST and Armature parser APIs

## Key Entry Points

- `lane::transform`
- `steps`
- `generate`
- `RuntimeHelpers`
- `lane::TransformContext`
- `lane::DirectiveTransform`
- `lane::NodeTransform`

## Related Crates

- `vize_armature` parses templates
- `vize_atelier_dom`, `vize_atelier_vapor`, and `vize_atelier_ssr` provide platform-specific backends
- `vize_atelier_sfc` orchestrates full `.vue` compilation on top of these building blocks

## License

MIT
