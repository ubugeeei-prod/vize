# vize_atelier_ssr

`vize_atelier_ssr` compiles Vue templates for server-side rendering output.

## Highlights

- SSR-specific step configuration
- String-oriented code generation with SSR helpers
- Shared Relief AST and Atelier Core lane

## Key Entry Points

- `compile_ssr`
- `compile_ssr_with_options`
- `SsrCompilerOptions`
- `SsrCodegenResult`

## Related Crates

- `vize_atelier_core` provides shared parsing and lane infrastructure
- `vize_atelier_sfc` can route template blocks through SSR compilation paths
- `@vizejs/vite-plugin` and Nuxt integration rely on this backend for SSR builds

## License

MIT
