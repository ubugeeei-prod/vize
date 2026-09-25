# Autocomplete Behavior

## Contract

Autocomplete is a thin, typed preset over Combobox for address- and
search-style inputs. `autocomplete-root.vue` wraps `ComboboxRoot` with free
text (`strict: false`), list autocompletion, open-on-focus, and a recent-history
list that is shown while the query is empty. History is controllable
(`v-model:history`), de-duplicated under `by`, capped by `maxHistory`, and can
persist through an injectable `historyStorage` adapter that is read only after
mount; slot state exposes `forget` and `clearHistory`. All
Combobox parts (input, content, items, async loading, virtualization) apply.

## Normative Behavior

| #   | State            | Input                          | Outcome                                                                                       | Proven by                                                                    |
| --- | ---------------- | ------------------------------ | --------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| A1  | any              | render / focus                 | Combobox root marked `data-vize-ui-preset="autocomplete"`, list autocomplete, opens on focus  | `is a free-text combobox preset that opens on focus and marks itself`        |
| A2  | typing           | Enter on a highlighted match   | selects it, records it in history, emits `submit(text, value)`                                | `choosing a suggestion records it in history and emits submit`               |
| A3  | empty query      | focus                          | suggestions are the history (newest first, de-duplicated, capped); typing switches to results | `an empty query shows history first, newest first, deduplicated and capped`  |
| A4  | free text        | Enter with nothing highlighted | emits `submit(text, null)`; `fromText` records search-style history; the model is untouched   | `free-text Enter submits the text and fromText records search-style history` |
| A5  | history present  | slot `clearHistory`            | empties history and returns to plain suggestions                                              | `slot clearHistory empties the history`                                      |
| A6  | `historyStorage` | mount / change                 | reads after mount, writes every change                                                        | `history storage is read after mount and written on change`                  |
| A7  | pure helpers     | push / remove / storage errors | deterministic; storage failures are swallowed                                                 | `history helpers are pure and storage failures are swallowed`                |

## SSR

History is rendered from `defaultHistory` on the server; storage is read only
after mount, so markup is byte-identical and hydrates without warnings
(`renders byte-identical Autocomplete markup with history suggestions`,
`hydrates Autocomplete without mismatches or node replacement`).
