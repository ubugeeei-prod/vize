## Area

Linter, `a11y/no-aria-hidden-on-focusable` (error in `happy-path`, `nuxt`, `ecosystem`, `opinionated`)

## Version

`vize` 0.432.0 (macOS arm64, Node 26.6.0)

## Minimal reproduction

`MySelect.vue`

```vue
<template>
  <div class="my-select">
    <!-- 1. inert: not focusable, not in the accessibility tree -->
    <input class="sizer" readonly aria-hidden="true" inert />
    <!-- 2. inside an inert subtree -->
    <div inert>
      <button type="button" aria-hidden="true">x</button>
    </div>
    <!-- 3. disabled form control: not focusable -->
    <button type="button" disabled aria-hidden="true">x</button>
    <!-- 4. hidden input: not rendered -->
    <input type="hidden" name="token" aria-hidden="true" />
    <!-- 5. tabindex="-1": not in the tab order -->
    <input class="sizer" readonly tabindex="-1" aria-hidden="true" />
    <!-- 6. focusable: should be reported -->
    <button type="button" aria-hidden="true">y</button>
  </div>
</template>
```

`vize.config.json`

```json
{ "linter": { "preset": "incremental", "rules": { "a11y/no-aria-hidden-on-focusable": "error" } } }
```

```sh
vize lint -f plain MySelect.vue
```

## Actual

```
Patina lint report: 6 errors in 1 file

MySelect.vue
  MySelect.vue:4:5 error a11y/no-aria-hidden-on-focusable aria-hidden="true" must not be used on focusable elements
    Help:
      Remove aria-hidden="true". tabindex="-1" is still programmatically focusable
  MySelect.vue:7:7 error a11y/no-aria-hidden-on-focusable aria-hidden="true" must not be used on focusable elements
    Help:
      Remove aria-hidden="true". tabindex="-1" is still programmatically focusable
  MySelect.vue:10:5 error a11y/no-aria-hidden-on-focusable aria-hidden="true" must not be used on focusable elements
    Help:
      Remove aria-hidden="true". tabindex="-1" is still programmatically focusable
  MySelect.vue:12:5 error a11y/no-aria-hidden-on-focusable aria-hidden="true" must not be used on focusable elements
    Help:
      Remove aria-hidden="true". tabindex="-1" is still programmatically focusable
  MySelect.vue:14:5 error a11y/no-aria-hidden-on-focusable aria-hidden="true" must not be used on focusable elements
    Help:
      Remove aria-hidden="true". tabindex="-1" is still programmatically focusable
  MySelect.vue:16:5 error a11y/no-aria-hidden-on-focusable aria-hidden="true" must not be used on focusable elements
    Help:
      Remove aria-hidden="true". tabindex="-1" is still programmatically focusable
```

Every element is reported, and the help text talks about `tabindex="-1"` on elements that do not
have a `tabindex` at all (lines 4, 7, 10, 12, 16).

## Expected

- **Lines 4 and 7: no error.** `inert` makes the element and its whole subtree non-focusable,
  including programmatic `focus()`, and removes it from the accessibility tree. The HTML spec says:
  "Inert nodes generally cannot be focused, and user agents do not expose the inert nodes to
  accessibility APIs or assistive technologies"
  (https://html.spec.whatwg.org/multipage/interaction.html#the-inert-attribute). So nothing is
  both hidden from AT and focusable, which is the thing this rule exists to catch.
- **Line 10: no error.** A `disabled` `<button>` / `<input>` / `<select>` / `<textarea>` is not
  focusable (HTML "focusable area" excludes disabled form controls).
- **Line 12: no error.** `<input type="hidden">` is not rendered and never focusable.
- **Line 16: error** (a focusable button hidden from AT), as now.
- The help text should only mention `tabindex="-1"` when the element actually has it.

Source: `is_focusable_markup_element` in `crates/vize_patina/src/rules/a11y/markup_helpers.rs`
returns `true` for every `button` / `input` / `select` / `textarea` / `summary` and for any
`tabindex >= -1`, without looking at `inert` (on the element or an ancestor), `disabled`, or
`type="hidden"`.

## Line 14 (`tabindex="-1"`): please reconsider (question, follow-up to #5943)

#5943 was resolved by keeping the error for `tabindex="-1"` and changing the help text. The rule's
doc comment says it is "Based on eslint-plugin-vuejs-accessibility no-aria-hidden-on-focusable",
but that plugin lists exactly this case as passing:

```vue
<!-- docs/rules/no-aria-hidden-on-focusable.md, "Succeed" -->
<button tabindex="-1" aria-hidden="true">Press</button>
```

(its `hasFocusableElements()` returns `tabindex !== "-1"` for interactive elements). WAI-ARIA's
"Using ARIA", 4th rule, also shows `<button tabindex="-1" aria-hidden="true">press me</button>` as
the way to hide an element that cannot be seen or interacted with
(https://www.w3.org/TR/using-aria/#fourth), and axe-core's `aria-hidden-focus` passes content made
unfocusable with `tabindex="-1"`. A decorative "sizer" `<input readonly tabindex="-1"
aria-hidden="true">` inside a custom select is a common pattern; today the only way to satisfy the
rule is to drop `aria-hidden`, which exposes a meaningless text field to screen readers. Would you
consider matching the upstream plugin here, or offering an option?
