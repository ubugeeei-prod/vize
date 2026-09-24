# TagsInput Behavior

## Contract

TagsInput is a headless compound field that turns typed or pasted text into a
typed tag list. `tags-input-root.vue` (`generic="T"`) owns the value, parsing,
validation, duplicate detection, the `max` limit, native form association, and
form reset. `tags-input-item.vue` renders one focusable tag, handles roving
arrow-key focus, deletion, and inline editing. `tags-input-item-text.vue`
renders the resolved tag text, `tags-input-item-delete.vue` renders a
non-tabbable remove button, and `tags-input-input.vue` renders the native text
input that commits candidates.

The text input is the single Tab stop. Tags are `tabindex="-1"` elements with
`role="group"`, `aria-roledescription="tag"`, and an accessible name equal to
the tag text; `group` is used because each tag contains its own remove button,
which a `button` or `option` role would not permit. Styling is consumer-owned
through parts, slots, CSS, and data attributes.

Without `parseTag`, tags are the trimmed typed strings; non-string tag types
must provide `parseTag` (and usually `tagText`, which also produces each
submitted form value).

## Public Surface

| Surface                       | Contract                                                                                                                                                                                                                          |
| ----------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TagsInputRoot` props         | `id`, `modelValue`, `defaultValue`, `parseTag`, `tagText`, `by`, `validate`, `delimiters`, `addOnPaste`, `addOnBlur`, `allowDuplicates`, `max`, `editable`, `disabled`, `readonly`, `required`, `name`, `form`, `dir`, aria props |
| `TagsInputRoot` emits         | `update:modelValue(tags)`, `add(tag, index, source)`, `remove(tag, index, source)`, `edit(tag, previous, index)`, `invalid(event)`                                                                                                |
| `TagsInputRoot` slots         | `default(state)` with `tags`, `inputValue`, `count`, `full`, `disabled`, `readonly`, `invalid`, `state`                                                                                                                           |
| `TagsInputRoot` expose        | `element`, `id`, `tags`, `inputValue`, `state`, `add`, `addTag`, `remove`, `clear`, `setInputValue`, `focus`, `reset`                                                                                                             |
| `TagsInputItem` props / slots | `value`, `index`, `disabled`, `editLabel`; `default(state)`                                                                                                                                                                       |
| `TagsInputItem` expose        | `element`, `focus`, `remove`, `edit`                                                                                                                                                                                              |
| `TagsInputItemText`           | `default(state)` slot; falls back to the resolved tag text                                                                                                                                                                        |
| `TagsInputItemDelete`         | `label` (prefix, default `"Remove"`), `ariaLabel` override; `default(state)` slot                                                                                                                                                 |
| `TagsInputInput`              | `placeholder`, `autocomplete`; expose `element`, `focus`                                                                                                                                                                          |
| Parts                         | `root`, `item`, `item-text`, `item-delete`, `item-edit`, `input`                                                                                                                                                                  |
| Data attributes               | root `data-vize-ui="tags-input"`, `data-state`, `data-count`, `data-full`, `data-invalid`, `data-disabled`, `data-readonly`; item `data-state`, `data-index`, `data-active`, `data-disabled`                                      |

## Normative Behavior

| #   | State                     | Input                                       | Outcome                                                                                                       | Proving test                                                                                                                                |
| --- | ------------------------- | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| T1  | any                       | render                                      | tags, ids (`<id>-tag-<index>`), aria wiring, remove names, and data attributes are published                  | `renders tags, input semantics, ids, and data attributes`                                                                                   |
| T2  | input has text            | Enter                                       | trimmed text is parsed and appended; default prevented; input clears; `add(..., "enter")`                     | `Enter and delimiter keys commit trimmed text while empty Enter passes through`                                                             |
| T3  | input empty               | Enter                                       | not prevented, so native implicit form submission still works                                                 | `Enter and delimiter keys commit trimmed text while empty Enter passes through`                                                             |
| T4  | any                       | delimiter key                               | default prevented and the text commits with source `"delimiter"`                                              | `Enter and delimiter keys commit trimmed text while empty Enter passes through`                                                             |
| T5  | any                       | input event containing delimiters           | complete segments commit; the trailing fragment stays in the input                                            | `typed delimiters commit complete segments and keep the trailing fragment`                                                                  |
| T6  | `addOnPaste`              | paste with delimiters, line breaks, or tabs | default prevented; every segment commits with source `"paste"`; undelimited paste stays native                | `pasting delimited or multi-line text splits it into tags`                                                                                  |
| T7  | candidate rejected        | commit                                      | `invalid` emits `parse`, `duplicate`, `max`, or `invalid` (+ validator message); rejected text stays editable | `duplicates, max, parse failures, and validators emit invalid events`                                                                       |
| T8  | `by` / `allowDuplicates`  | commit equal tag                            | a property name or comparator defines equality; `allowDuplicates` accepts equals                              | `allowDuplicates and by keys control duplicate detection`                                                                                   |
| T9  | caret at start, tags      | Backspace / ArrowLeft in input              | focus moves to the last tag (first Backspace focuses, the next one deletes)                                   | `keyboard matrix moves focus between tags and removes with Backspace and Delete`                                                            |
| T10 | text before caret         | Backspace / ArrowLeft in input              | native caret editing; focus stays in the input                                                                | `Backspace and ArrowLeft keep caret editing when text precedes the caret`                                                                   |
| T11 | tag focused               | ArrowLeft / ArrowRight / Home / End         | previous / next tag (no wrap), next past the last tag and End focus the input, Home the first tag             | `keyboard matrix moves focus between tags and removes with Backspace and Delete`                                                            |
| T12 | tag focused               | Backspace / Delete                          | removes the tag; Backspace focuses the previous tag, Delete the tag now in the slot, else the input           | `keyboard matrix moves focus between tags and removes with Backspace and Delete`                                                            |
| T13 | `dir="rtl"`               | arrows                                      | horizontal arrows are mirrored                                                                                | `rtl direction mirrors horizontal arrows`                                                                                                   |
| T14 | any                       | remove button click                         | removes with source `"delete-button"`; focus stays on a tag or returns to the input                           | `the delete button removes its tag and keeps focus in the field`                                                                            |
| T15 | `editable`                | Enter / F2 / double-click on tag            | inline edit input opens focused; Enter commits (`edit`), invalid keeps editing, Escape cancels, blur commits  | `editable tags edit inline with Enter, F2, and double-click`                                                                                |
| T16 | `disabled` / `readonly`   | any change                                  | disabled removes tags from focus and submission; readonly keeps them focusable and submitted; nothing changes | `disabled and readonly fields block every change`                                                                                           |
| T17 | `name` in a form          | submit / reset                              | one hidden entry per tag (`tagText`); `required` blocks empty submission; form reset restores `defaultValue`  | `submits one form entry per tag, validates required, and restores on form reset`                                                            |
| T18 | controlled                | commit                                      | emits the requested list and waits for the parent                                                             | `controlled tags wait for the parent to accept updates`                                                                                     |
| T19 | `addOnBlur` / composition | blur / IME Enter or delimiter               | blur commits only with `addOnBlur`; keys during composition never commit                                      | `addOnBlur commits on blur and IME composition never commits`                                                                               |
| T20 | any                       | pointerdown on empty root space             | default prevented and focus moves to the input                                                                | `pointerdown on empty root space focuses the input`                                                                                         |
| T21 | any                       | exposed API                                 | `add`, `addTag`, `remove`, `clear`, `setInputValue`, `focus`, `reset`, item `focus`/`edit` share one state    | `exposed root and item methods share state`                                                                                                 |
| T22 | no provider               | mount part alone                            | throws `VIZE_UI_CONTEXT_MISSING`                                                                              | `items and parts require a matching TagsInput provider`                                                                                     |
| T23 | SSR                       | render twice / hydrate                      | byte-identical markup; hydration keeps nodes and ids with no diagnostics                                      | `renders byte-identical TagsInput markup across isolated SSR requests`, `hydrates tags, ids, and hidden inputs without replacing SSR nodes` |
