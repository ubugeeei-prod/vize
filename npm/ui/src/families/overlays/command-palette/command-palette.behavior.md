# CommandPalette behavior contract

Normative behavior for `@vizejs/ui/command-palette`, an APG combobox with a listbox popup
built on the `command` router, `collection` + `composite-navigation` (active descendant),
`live-region`, `shortcut`, and the Dialog parts. Every row names its proving test.

| State x input                                                         | Observable outcome                                                                                                                                                                | Proven by                                                                                 |
| --------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| any render                                                            | The input is `role="combobox"` with `aria-autocomplete="list"`, `aria-expanded`, `aria-controls` to the listbox, and `aria-activedescendant` on the first enabled option.         | `renders combobox and listbox semantics with an active descendant`                        |
| options and groups                                                    | Options are `role="option"` with `aria-selected` for the active one, `aria-disabled`, and `aria-keyshortcuts`; groups are `role="group"` labelled by their heading.               | `renders combobox and listbox semantics with an active descendant`                        |
| typing in the input                                                   | Items are scored against label and keywords; non-matching items and empty groups get `hidden`; the first visible option becomes active; `update:search` fires.                    | `typing filters items and groups, resets the active item, and announces the count`        |
| search changes after mount                                            | A polite live region announces the visible result count (`resultsLabel`).                                                                                                         | `typing filters items and groups, resets the active item, and announces the count`        |
| no visible results, not loading                                       | `CommandPaletteEmpty` is shown and the listbox reports `data-empty`.                                                                                                              | `typing filters items and groups, resets the active item, and announces the count`        |
| ArrowDown / ArrowUp / Home / End                                      | The active option moves, skipping disabled and hidden options, wrapping when `loop`.                                                                                              | `arrow keys, Home, and End move the active option and skip disabled ones`                 |
| Enter                                                                 | The active option is selected: item `select(value, event)`, root `select(commandId, event)`; composing (IME) keystrokes are ignored.                                              | `Enter selects the active item, IME composition is ignored, and Escape clears the search` |
| Escape                                                                | A non-empty search is cleared and the key is consumed; an empty search lets Escape reach an enclosing dialog.                                                                     | `Enter selects the active item, IME composition is ignored, and Escape clears the search` |
| pointer over / click on an option                                     | `pointermove` activates it; `mousedown` is prevented to keep focus in the input; click selects unless disabled.                                                                   | `pointer movement activates options and clicks select without stealing focus`             |
| router commands                                                       | The root slot receives filtered `commands`; items with `command` take its title/keywords, reflect `when` as disabled, run with source `"palette"`, and update `recent`.           | `router commands render through the slot, run with the palette source, and track recents` |
| `shouldFilter=false`, custom `filter`, `loading`, controlled `search` | Local filtering is skipped, custom scores decide visibility, the listbox is `aria-busy` with the loading part shown and empty hidden, and controlled search waits for the parent. | `shouldFilter=false, custom filters, loading, and controlled search are honored`          |
| closed palette (`open=false`)                                         | The listbox is hidden, `aria-expanded="false"`, no active descendant; ArrowDown requests `update:open(true)`.                                                                     | `closed palettes hide the list and arrows reopen it`                                      |
| CommandPaletteDialog, `Mod+K`                                         | The global shortcut toggles the dialog (Command on Apple, Control elsewhere); a selection closes it when `closeOnSelect`; `shortcut=null` disables.                               | `the dialog toggles with Mod+K and closes after a selection`                              |
| default filter                                                        | Exact > prefix > word prefix > substring > subsequence; case and diacritics are ignored; keywords count slightly less.                                                            | `default filter ranks exact, prefix, word, substring, and subsequence matches`            |
| parts outside the root                                                | Mounting throws `VIZE_UI_CONTEXT_MISSING`.                                                                                                                                        | `palette parts require a root provider`                                                   |
| SSR                                                                   | Isolated requests render byte-identical markup; the closed dialog renders no content; the shortcut listener attaches only after mount.                                            | `renders byte-identical palette markup across isolated SSR requests`                      |
| hydration                                                             | Hydration reuses server nodes with zero warnings and then activates the first visible option.                                                                                     | `hydrates without mismatches and activates the first visible option`                      |
| public types                                                          | Router id unions flow into recent ids and `select`; item `value` types the item `select` payload.                                                                                 | `src/families/overlays/command-palette/command-palette.types.test-d.ts`                   |

## Components

| Component                     | Contract                                                                                                      |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `command-palette-root.vue`    | Generic over router command ids; owns search, open, recent, scoring, active option, and the result announcer. |
| `command-palette-input.vue`   | Combobox input: typing, arrows, Home/End, Enter, Escape, IME-safe.                                            |
| `command-palette-list.vue`    | Listbox popup with `aria-busy` and `data-empty`.                                                              |
| `command-palette-group.vue`   | Labelled option group hidden when it has no visible options (unless `forceMount`).                            |
| `command-palette-item.vue`    | Generic over `value`; option with keywords, router `command`, `shortcut`, and `select`.                       |
| `command-palette-empty.vue`   | Fallback shown when nothing matches and nothing is loading.                                                   |
| `command-palette-loading.vue` | `progressbar` shown while `loading`.                                                                          |
| `command-palette-dialog.vue`  | Dialog composition with the `Mod+K` toggle; closes after selection.                                           |

## Parts And Data

| Target | Public contract                                                                         |
| ------ | --------------------------------------------------------------------------------------- |
| Root   | `data-vize-ui="command-palette-root"`, `part="root"`, `data-state`, `data-loading`      |
| Input  | `data-vize-ui="command-palette-input"`, `part="input"`, `data-state`                    |
| List   | `data-vize-ui="command-palette-list"`, `part="list"`, `data-state`, `data-empty`        |
| Group  | `data-vize-ui="command-palette-group"`, heading `part="group-heading"`, `data-empty`    |
| Item   | `data-vize-ui="command-palette-item"`, `part="item"`, `data-state`, `data-command`      |
| Dialog | `data-vize-ui="command-palette-dialog"`, `part="dialog"`, `data-state`, `data-shortcut` |

CommandPalette ships no stylesheet; the announcer is visually hidden with inline styles.
