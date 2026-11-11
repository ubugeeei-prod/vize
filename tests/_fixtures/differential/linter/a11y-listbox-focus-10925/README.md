# Listbox focus ownership, issue #10925

`originalCompositeWidgets.vue.txt` preserves the Vue code fence from
[issue #10925](https://github.com/ubugeeei-prod/vize/issues/10925), including its
script and all three composite widgets. `original-source.json` pins its bytes,
the issue body hash, and the report revision. The `.txt` suffix keeps this
immutable source outside the generated Vue event-handler inventory.

`original-before.json` retains the complete public CLI 0.600.0 process output.
The opinionated route reports exactly one listbox warning, at the authored
opening tag. Its pinned argv passes `--quiet`, which now reports errors only
(#12824), so the expected packet keeps every other field and has no messages
and no warnings; no source file changes.

The W3C APG describes two different focus strategies:

- [Roving tabindex](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/#kbd_roving_tabindex)
  moves DOM focus among the component's children. A listbox whose owned options
  provide a focus contract does not also need a focusable container.
- [aria-activedescendant](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/#kbd_focus_activedescendant)
  retains DOM focus on a focusable owning element. The attribute by itself does
  not make an otherwise unfocusable listbox usable.

The [listbox pattern](https://www.w3.org/WAI/ARIA/apg/patterns/listbox/)
identifies options as belonging to the listbox or one of its owned groups.
This regression admits physical, focusable `role="option"` descendants through
ordinary, presentational, group, and template wrappers. Another listbox,
another widget, a component, or a slot is an ownership boundary. Hidden, inert,
and statically disabled descendants do not establish focus ownership.

`cases.json` retains 37 complete controls across Vue SFC, HTML, JSX, and TSX:
148 whole public-before packets and 148 whole expected packets. Each source
has its own SHA-256. The #12449 compatibility repair removes the remaining
focus warnings because none of these controls has an interactive handler;
their authored roles alone do not create a focus obligation. The source CLI
test verifies every route twice and retains complete process observations.
The Rust controls also exercise the legacy template/SFC dispatch separately.

The matrix preserves existing exact-name behavior, including lowercase
`tabindex` and statically named bound attributes. Static `0` and `-1` are
focusable; bound values remain the existing rule's runtime focus contract.
An unrelated button, an option outside the listbox, a nested listbox's option,
and options under components or slots still do not establish focus ownership.
When a role owner has an interactive handler, these controls continue to
decide whether that owner already has a valid focus strategy.

Existing combobox identity cases remain explicit: static ID references,
bound expression identity, literal-valued binds, multiple `aria-controls`
tokens, missing active descendants, and mismatched bound IDs. Their ownership
classification remains available to handler-bearing widgets; the handlerless
matrix itself is now silent across Vue, HTML, JSX, and TSX.

The baseline is a new derivative capture. No previous snapshots, original
provider archives, hashes, generated event inventories, or other rules change.
