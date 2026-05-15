# vize_fresco

`vize_fresco` is the terminal UI foundation used by Vize's TUI-oriented experiments.

## Highlights

- Cross-platform terminal primitives
- Flexbox-style layout via `taffy`
- Render tree, buffer, and text measurement utilities
- Optional NAPI bindings through the `napi` feature

## Key Entry Points

- `BoxNode`, `TextNode`, `InputNode`
- `LayoutEngine`, `FlexStyle`, `Rect`
- `RenderTree`, `RenderNode`
- `Backend`, `Buffer`, `Cursor`

## Related Crates

- `@vizejs/fresco` and `@vizejs/fresco-native` expose this crate to JavaScript consumers
- The crate is independent from the Vue compiler pipeline, but lives in the same workspace

## License

MIT
