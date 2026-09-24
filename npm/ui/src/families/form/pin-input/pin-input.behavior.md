# PinInput behavior contract

Normative state x input -> outcome table for `pin-input.vue` and
`pin-input-field.vue` (`@vizejs/ui/pin-input`). The model is the joined code
(no gaps); a literal `length` types the `complete` characters tuple. Every row
is proven by the named test in `pin-input.test.ts` or `pin-input-ssr.test.ts`;
compile-only assertions live in `pin-input.types.test-d.ts`.

| #    | State           | Input                                      | Outcome                                                                                                                                                     | Proven by                                                                         |
| ---- | --------------- | ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| PI1  | helpers         | sanitize / write / remove / narrow         | accepted characters are filtered (NFKC folds full-width), writes clamp to the end and truncate, removals shift left, tuples narrow by length                | `sanitizes, writes, removes, and narrows code characters`                         |
| PI2  | named, seeded   | render                                     | labelled `role="group"`; fields with deterministic ids, `one-time-code` autocomplete on the first field, `inputmode`, per-field labels, hidden joined value | `renders a labelled group of one-time-code fields with form hooks`                |
| PI3  | empty           | typing                                     | characters fill the first gap, focus advances, rejects are ignored, `complete(value, tuple)` fires once, typing over a field replaces it                    | `typing fills fields left to right, advances focus, and emits a typed completion` |
| PI4  | any             | paste / SMS autofill                       | pasted or autofilled codes are filtered and distributed across fields; focus lands on the last written field                                                | `paste and autofill distribute characters across fields`                          |
| PI5  | filled          | Backspace / Delete / arrows / Home / End   | Backspace clears the focused character or deletes backwards from an empty field; Delete removes forward; arrows/Home/End move focus; Tab stays native       | `Backspace, Delete, arrows, Home, and End edit and move between fields`           |
| PI6  | options         | mask / alphanumeric / pattern / controlled | `mask` renders password fields, `type="alphanumeric"` uses a text keyboard, `pattern` overrides accepted characters, controlled values win                  | `masking, alphanumeric codes, custom patterns, and controlled values`             |
| PI7  | in a form       | submit / reset                             | hidden input submits the joined code; fields are `required` only until complete; form reset restores `defaultValue`                                         | `submits the joined code, requires completion, and restores defaults on reset`    |
| PI8  | disabled        | edits                                      | every field is natively disabled and edits are ignored                                                                                                      | `disabled codes disable every field and ignore edits`                             |
| PI9  | imperative      | expose / missing provider                  | `focus` (first empty field by default), `setValue`, `clear`, state; fields outside a PinInput throw `VIZE_UI_CONTEXT_MISSING`                               | `exposes focus, setValue, clear, and state; fields need a PinInput`               |
| PI10 | SSR / hydration | isolated requests                          | byte-identical markup, hydration without diagnostics, and interactive fields                                                                                | `renders byte-identical code fields and hydrates without mismatches`              |

## Public extension contract

| Surface         | Contract                                                                                      |
| --------------- | --------------------------------------------------------------------------------------------- |
| Parts           | `root` (group), `field` (one native input per character).                                     |
| Data attributes | `data-vize-ui`, root `data-state`/`data-complete`; field `data-index`/`data-filled`.          |
| Slots           | Root default slot receives `PinInputSlotState`; render a `PinInputField` per `indexes` entry. |

The subpath is tree-shakable and ships no CSS; those package contracts are
pinned by `distribution.test.ts`, `check:size`, and `check:tree-shaking`.
