<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_fresco.svg" alt="vize_fresco logo" width="120" height="120" /><br>
  vize_fresco
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_fresco` is the terminal UI foundation used by Vize's TUI-oriented experiments.

## Highlights

- Cross-platform terminal primitives
- Deterministic color, Unicode, redirected-output, interactive, and narrow-layout profiles
- Flexbox-style layout via `taffy`
- Render tree, buffer, and text measurement utilities
- Stable-keyed, virtualized diagnostic master-detail workspace state
- Deterministic headless screen, semantic-tree, focus, cursor, and announcement snapshots
- Optional NAPI bindings through the `napi` feature

## Key Entry Points

- `BoxNode`, `TextNode`, `InputNode`
- `DiagnosticWorkspaceState`, `DiagnosticWorkspaceLayout`, `VirtualListState`
- `LayoutEngine`, `FlexStyle`, `Rect`
- `RenderTree`, `RenderNode`
- `HeadlessRenderer`, `HeadlessPresentation`, `HeadlessSnapshot`
- `Backend`, `Buffer`, `Cursor`
- `TerminalCapabilities`, `TerminalCapabilityProbe`, `TerminalProfileOptions`

## Related Crates

- `@vizejs/fresco` and `@vizejs/fresco-native` expose this crate to JavaScript consumers
- The crate is independent from the Vue compiler pipeline, but lives in the same workspace

## License

MIT
