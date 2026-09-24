# Mention Behavior

## Contract

Mention is a headless trigger-character helper (`@` people, `#` tags, `:`
emoji, …) for native text fields and contenteditable editors.
`mention-core.ts` is DOM-free: `detectMention(text, caret, triggers)` finds the
active token and `applyMentionEdit` replaces it with the text produced by an
insertion transform. `mention-root.vue` is generic over the item type `T`,
inferred from `items` or `loadItems`, and owns the text model, the active
token, filtering, the async loader, and an active-descendant collection built
on the collection registry and composite navigation.

`mention-input.vue` renders a native `<textarea>` (or `<input role="combobox">`
with `as="input"`) and `mention-editable.vue` a `role="textbox"`
contenteditable. Both keep DOM focus and expose the highlighted option through
`aria-activedescendant`, `aria-autocomplete="list"`, and `aria-controls`.
`mention-content.vue` renders the `role="listbox"` popup through Portal,
Presence, and a Positioner anchored to a virtual element at the trigger
character (a mirror element for text fields, a DOM Range for contenteditable),
with a dismissable layer for outside presses and Escape. `mention-item.vue`
renders `role="option"` and `mention-empty.vue` the no-results message. No CSS
is shipped.

## Public Surface

| Surface              | Contract                                                                                                                                                           |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `MentionRoot` props  | `id`, `modelValue`, `defaultValue`, `items`, `loadItems`, `debounce`, `triggers`, `itemText`, `filter`, `insert`, `open`, `defaultOpen`, `loop`, `disabled`        |
| `MentionRoot` emits  | `update:modelValue(text)`, `select(item, trigger)`, `update:query(query)`, `query-change(query, trigger)`, `update:open(open)`, `open-change(open, previous, e)`   |
| `MentionRoot` slot   | `text`, `query`, `trigger`, `match`, `filteredItems`, `open`, `loading`, `status`, `error`, `state`                                                                |
| `MentionRoot` expose | `text`, `match`, `open`, `select`, `dismiss`, `refresh`, `focus`                                                                                                   |
| `MentionInput` props | `as`, `name`, `placeholder`, `rows`, `ariaLabel`, `ariaLabelledby`, `ariaDescribedby`                                                                              |
| `MentionContent`     | positioner props, `forceMount`, `to`, `portalDisabled`, `defer`, `ariaLabel`; emits `escape-key-down`, `pointer-down-outside`, `dismiss`                           |
| Trigger rules        | start of text or after whitespace/punctuation; `@@` escapes; query ends at whitespace unless `allowSpaces`, always at a newline; `pattern`, `minChars`, `maxChars` |

## Normative Behavior

| #   | State             | Input                                       | Outcome                                                                                                                                        | Proven by                                                                                                                |
| --- | ----------------- | ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| M1  | idle              | render                                      | textarea with `aria-autocomplete="list"`, `aria-haspopup="listbox"`, no `aria-controls`, no popup                                              | `renders a textarea with listbox autocomplete semantics and no popup`                                                    |
| M2  | idle              | type a trigger + query                      | opens, wires `aria-controls`, filters by query, highlights the first item, emits `update:query`/`query-change`; empty content shows            | `typing a trigger opens the listbox, filters by query, and highlights the first item`                                    |
| M3  | any               | mid-word trigger / caret leaves token       | never opens mid-word; moving the caret out of the token closes and emits `query-change(null, null)`                                            | `mid-word triggers never open and moving the caret out of the token closes`                                              |
| M4  | open              | ArrowDown/ArrowUp/Home/End/Enter/Tab/Escape | arrows and Home/End move the highlight (`loop` wraps); Enter and Tab insert, move the caret, emit `select`; Escape dismisses until a new token | `keyboard matrix: arrows, Home/End, Enter inserts, Tab inserts, Escape dismisses`                                        |
| M5  | closed / no match | ArrowDown / Enter                           | not consumed, so native editing and form submission proceed                                                                                    | `keys pass through while closed and Enter without a highlight is not consumed`                                           |
| M6  | open              | pointer move / press / click                | highlights, keeps focus in the field, inserts and merges a doubled space                                                                       | `clicking an item inserts it and pointer movement highlights`                                                            |
| M7  | several triggers  | `#` / `@` tokens with `insert`              | each token uses its trigger; the transform receives item and trigger; `as="input"` renders `role="combobox"` with `aria-expanded`              | `insertion transforms and multiple triggers receive the item and trigger`                                                |
| M8  | contenteditable   | type / Enter                                | detects the token from the DOM selection and replaces it in place, emitting the new text                                                       | `contenteditable fields detect tokens from the selection and insert text in place`                                       |
| M9  | `loadItems`       | type / newer query / close                  | loads per query and trigger, publishes `loading`, hides empty content meanwhile, aborts superseded requests, keeps loaded order                | `loadItems loads per query, publishes loading, and aborts superseded requests`                                           |
| M10 | controlled / API  | filter / dismiss / disabled                 | injected filters replace the default; `dismiss()` closes; disabled fields never open                                                           | `controlled text, filter injection, disabled state, and exposed methods`                                                 |
| M11 | no provider       | mount a part                                | throws `VIZE_UI_CONTEXT_MISSING`                                                                                                               | `parts require a Mention provider`                                                                                       |
| M12 | caret helpers     | measure / locate / replace                  | mirror measurement leaves no nodes behind; offsets map across text nodes; editable replacement edits in place                                  | `caret helpers measure text fields and editable content without leaking mirrors`                                         |
| C1  | core              | detect                                      | start and after punctuation; caret-based                                                                                                       | `detects a trigger at the start and after whitespace or punctuation`, `uses the caret position, not the end of the text` |
| C2  | core              | reject                                      | mid-word, `@@`, whitespace, newline, out-of-range caret                                                                                        | `never triggers mid-word, after a repeated trigger, or across whitespace and newlines`                                   |
| C3  | core              | trigger options                             | `pattern`, `minChars`, `allowSpaces`, `maxChars`, multi-character triggers                                                                     | `supports several triggers with patterns, minimum length, and spaces`, `multi-character triggers match as a unit`        |
| C4  | core              | edit / compare / filter                     | default insertion, caret after insertion, doubled-space merge, match equality, accent-insensitive filter                                       | `applies edits with the default insertion and collapses a doubled space`, `compares matches and normalizes filter text`  |

## SSR

No token is active on the server, so the popup renders closed and every id
comes from the deterministic-id primitive. Textarea and contenteditable trees
render byte-identical markup and hydrate without warnings (`renders
byte-identical Mention textarea markup across isolated requests`, `renders
byte-identical contenteditable markup`, `hydrates textarea and contenteditable
Mentions without mismatches`). Caret measurement only runs in handlers and
positioner updates. For `mention-editable.vue` the editor DOM is the source of
truth: its text is read on input and written back on insertion.
