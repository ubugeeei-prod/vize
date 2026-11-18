## Summary

The Real Project Matrix's misskey typecheck was configuration-only because of 22 TS2307 on `*.glsl` imports. They never came from Vize.

- Two steps pin `CORSA_PATH` to the matrix's tsgo 7.0.2: "Check real-project LSP lifecycle" and "Classify pinned Misskey generated diagnostics". Two did not: "Exercise real projects with every core tool" and "Enforce typechecker baseline divergence". Without a pin, Vize's runtime resolver selects the project's declared native TypeScript (stable `@typescript/typescript` first, then a declared `@typescript/native-preview`).
- misskey declares `@typescript/native-preview` **7.0.0-dev.20260116.1**. That build reports the same 22 TS2307 on misskey's own `packages/frontend/tsconfig.json` with no Vize involved, while `vite-plugin-glsl/ext.d.ts` (`declare module '*.glsl'`) is in the program. tsgo 7.0.2 fixed it.
- Evidence: Linux trace 44215275012 ran `vize check` on the full misskey corpus with the matrix flags and the pinned checker, with servers 1, 8 and default: 0 diagnostics and 0 glsl, while the unpinned core-tool step in the same job reported 22. The same pair reproduces on macOS: pinned 0, auto-detected 22. It was never platform-specific.

## Change

Pin both steps to `${{ github.workspace }}/node_modules/@typescript/typescript-linux-x64/lib/tsc`, the checker the LSP and classify steps already use. Every surface then qualifies Vize on one supported checker. The workflow pin test asserts both pins before comparing the original workflow.

Auto-detection stays covered where it belongs, by resolver unit tests in `crates/vize_carton/src/corsa_resolver/`: `resolves_declared_native_preview_platform_package`, `resolves_declared_preview_through_a_pnpm_link`, `prefers_typescript_seven_runtime_over_native_preview` and `checkout_cache_never_overrides_a_declared_preview`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
