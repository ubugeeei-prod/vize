# Sortable behavior contract

Normative state × input → outcome table for `@vizejs/ui/sortable`. Every row is
exercised by `src/sortable*.test.ts`; compile-only assertions live in
`src/sortable.types.test-d.ts`.

| #   | State                              | Input                                         | Outcome                                                              | Proven by              |
| --- | ---------------------------------- | --------------------------------------------- | -------------------------------------------------------------------- | ---------------------- |
| S1  | idle                               | items register in any order                   | logical indexes follow document order, not registration order        | ordering test          |
| S2  | idle                               | pointer travels the start distance on an item | `sortstart` reports the item's current index and announces the grab  | pointer lifecycle test |
| S3  | sorting                            | pointer over another item's leading half      | preview projects `before` and the indicator carries the edge line    | pointer preview test   |
| S4  | sorting                            | pointer over another item's trailing half     | preview projects `after` with the moved insertion index              | pointer preview test   |
| S5  | sorting                            | unchanged projection on further movement      | no duplicate preview is emitted                                      | preview dedup test     |
| S6  | sorting over an item               | owning contact releases                       | `sortcommit` reports origin and final indexes exactly once           | pointer commit test    |
| S7  | sorting                            | Escape, blur, hidden document, or cancel()    | `sortcancel` reports the index to return to and announces the return | cancellation tests     |
| S8  | sorting released outside           | owning contact releases over no item          | the sort cancels back to the origin index                            | outside-release test   |
| S9  | idle focus on item                 | Enter or Space without modifiers              | keyboard sort grabs, reports position, and speaks usage instructions | keyboard grab test     |
| S10 | keyboard sorting (vertical)        | ArrowUp and ArrowDown, Home, and End          | destination steps and clamps; each change previews and announces     | keyboard move tests    |
| S11 | keyboard sorting (grid)            | ArrowUp and ArrowDown                         | destination steps by the resolved column count                       | grid keyboard test     |
| S12 | keyboard sorting (horizontal, RTL) | ArrowLeft and ArrowRight                      | logical direction flips with the resolved writing direction          | RTL keyboard test      |
| S13 | keyboard sorting (nesting)         | ArrowRight then ArrowLeft                     | preview nests inside the previous item, then returns to index moves  | nesting keyboard test  |
| S14 | sorting (nesting)                  | pointer inside an item's central band         | preview and commit report `"inside"` with the receiving item's key   | nesting pointer test   |
| S15 | keyboard sorting                   | Enter or Space                                | `sortcommit` reports origin and destination and announces the drop   | keyboard commit test   |
| S16 | disabled item or controller        | any grab attempt                              | no sort starts; an active sort cancels on reactive disablement       | disablement tests      |
| S17 | any sort                           | item registration disposal                    | the owning sort cancels first and both registrations release         | disposal tests         |
| S18 | any sort                           | controller disposal                           | delegated listeners and reactive state release without callbacks     | disposal tests         |
| S19 | concurrent SSR requests            | identical component trees                     | byte-identical markup contains no handlers or DOM reads              | SSR test               |
| S20 | SSR followed by hydration          | keyboard sort                                 | host identity remains and reactive output updates without warning    | hydration test         |
| S21 | public TypeScript API              | mutation or invalid closed union              | compilation rejects misuse                                           | type assertions        |

## Accessibility and layout obligations

Every item host must be keyboard focusable so pointer sorting has an
Enter-grab, arrow-move, Enter-drop equivalent, and every phase speaks through
the underlying drag-and-drop live region with injectable, localizable
builders. Grid and horizontal arrow keys resolve against the configured
writing direction so RTL layouts move logically. The primitive is headless: it
assigns no role or style, and consumers render placeholder and indicator
geometry from the reactive indicator state. Nested trees compose this family
with the drag-and-drop core's innermost-target ownership; `"inside"` previews
and commits carry the receiving item so consumers re-parent their own model.
