# vize_atelier_core

`vize_atelier_core` contains the shared transform and code generation infrastructure used by the
DOM, Vapor, and SSR compilers.

## Highlights

- Core transform pipeline and `TransformContext`
- Shared Vue template code generation
- Runtime helper resolution
- Re-exports for the Relief AST and Armature parser APIs

## Key Entry Points

- `transform`
- `generate`
- `RuntimeHelpers`
- `TransformContext`
- `DirectiveTransform`
- `NodeTransform`

## Related Crates

- `vize_armature` parses templates
- `vize_atelier_dom`, `vize_atelier_vapor`, and `vize_atelier_ssr` provide platform-specific backends
- `vize_atelier_sfc` orchestrates full `.vue` compilation on top of these building blocks

## License

MIT
