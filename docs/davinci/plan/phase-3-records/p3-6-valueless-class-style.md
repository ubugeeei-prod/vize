# P3-6 — Valueless class and style

A bare `class` or `style` beside the same binding was `legacy.binding`.
Official Vapor binds that attribute as the empty string and merges it with
the dynamic value. Quoted `style=""` already did. This slice admits the
bare form. P3-6 stays open: parity, the bench floor, and the legacy lane
are unchanged.

## Contract

- `<div style :style="s">` and `<div :style="s" style>` are native. The
  static value is `""` and leaves the template. Native style follows
  authored order (`["", dynamic]` then `[dynamic, ""]`).
- `<div class :class="c">` and `<div :class="c" class>` are native. Class
  stays static-first on both lanes, the same gap valued class already has.
- The retained lane also keeps the empty string, still static-first, so a
  style written after `:style` does not match the native lane.
- A bare attribute with no dynamic twin is unchanged. Two dynamic
  `:class` or `:style` bindings stay `legacy.binding`.

## Witness

`cargo test -p vize_atelier_vapor --lib valueless_static_class_and_style_merge_as_empty_strings`
