# EmojiPicker Behavior

## Contract

EmojiPicker is a headless, data-free emoji picker: no dataset ships, and the
consumer maps typed items through required `getEmoji`/`getName` accessors plus
optional `getKeywords`, `getCategory`, and `getSkins`. `emoji-picker-root.vue`
is generic over the item type `T`, builds sections (recent first, then
`categories` in order, then unknown categories; a non-empty search collapses
into one "search" section), owns the controllable `skinTone` and `search`, and
emits `select(item, glyph)` with the tone applied.

`emoji-picker-grid.vue` renders the APG `role="grid"` focus owner with
`aria-activedescendant`. Navigation runs over the concatenated rows of every
visible section: each section starts a new row and short rows are padded, so
the listbox-grid `moveInGrid` helper moves in two dimensions and skips padding.
`emoji-picker-category.vue` renders a labelled `role="rowgroup"` with a
`columnheader` label row and `role="row"` chunks, `emoji-picker-item.vue` a
`role="gridcell"`, `emoji-picker-search.vue` a search field,
`emoji-picker-skin-tone.vue` a roving `radiogroup` of six tones,
`emoji-picker-preview.vue` a polite live preview, and `emoji-picker-empty.vue`
the no-results state. No CSS ships.

## Normative Behavior

| #   | State            | Input                                             | Outcome                                                                                                                                                                     | Proven by                                                                                    |
| --- | ---------------- | ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| E1  | any              | render                                            | focusable grid with `aria-colcount`/`aria-rowcount` (item plus label rows), labelled rowgroups, named gridcells                                                             | `renders an APG grid with labelled category rowgroups and gridcells`                         |
| E2  | grid focused     | arrows / Home / End / Ctrl+Home/End / PageUp/Down | focus highlights the first cell; moves in 2-D across section boundaries, skipping padding cells                                                                             | `2-D keyboard navigation crosses section boundaries over padded rows`                        |
| E3  | highlighted cell | Enter / Space / click                             | emits `select(item, glyph)` with the skin tone applied; clicked cells become active                                                                                         | `Enter, Space, and click emit select with the skin tone applied`                             |
| E4  | skin tones       | arrows / Home / End / click                       | roving tabindex radiogroup; arrows wrap and select; glyphs update; `update:skinTone` emits                                                                                  | `the skin-tone radiogroup uses roving focus and updates glyphs`                              |
| E5  | controlled tone  | click a tone                                      | emits and waits for the parent                                                                                                                                              | `controlled skin tone waits for the parent`                                                  |
| E6  | search           | type / Enter / Escape / ArrowDown                 | filters names and keywords accent-insensitively into one section, highlights the first match, Enter selects it, empty state shows, Escape clears, ArrowDown enters the grid | `search filters by name and keyword, highlights the first match, and drives the empty state` |
| E7  | `recent`         | render / navigate                                 | recent items form the first section; repeated items keep unique cell ids                                                                                                    | `recent items form the first section and repeat safely`                                      |
| E8  | pointer          | pointer move                                      | highlights the cell and the live preview shows glyph and name                                                                                                               | `the preview follows the highlighted emoji`                                                  |
| E9  | no provider      | mount a part                                      | throws `VIZE_UI_CONTEXT_MISSING`                                                                                                                                            | `parts require an EmojiPicker provider`                                                      |
| E10 | pure helpers     | sections / glyphs / virtual grid                  | unknown categories follow in first-seen order; padding cells are empty; tones clamp to 0–5                                                                                  | `pure helpers build sections, glyphs, and the padded virtual grid`                           |

## SSR

Cell ids derive from the root id plus section and item positions, so server
markup is byte-identical across isolated requests and hydrates without warnings
(`renders byte-identical EmojiPicker markup across isolated SSR requests`,
`hydrates EmojiPicker without mismatches or node replacement`). Large sets are
filtered in O(n) per keystroke; rows are not virtualized.
