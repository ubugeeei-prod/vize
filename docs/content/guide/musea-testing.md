---
title: Musea interaction testing
---

# Musea interaction testing


Author component values directly on the component and browser behavior on its variant. Static
props and attrs seed the Props panel, permalinks, and VRT metadata without duplicating the example.
The entire example stays in one `.art.vue` file:

```art-vue
<script setup lang="ts">
import Counter from "./Counter.vue";
</script>

<art>
  <variant
    name="Interactive"
    :play="async ({ canvas, user, expect }) => {
      await user.click(canvas.getByRole('button', { name: 'Increment' }));
      expect(canvas.getByText('Count: 1')).toBeVisible();
    }"
    default
  >
    <Counter :initial="0" :step="1" aria-label="Counter example" />
  </variant>
</art>
```

Inline `play` expressions are contextually checked as `MuseaPlay`; `canvas`, `user`, `expect`, and
`step` therefore keep their concrete types without parameter annotations. Direct component inputs
are checked by Vue against `Counter` itself. Musea statically serializes
literal values for the Props workspace and never evaluates project code while discovering the
gallery. Dynamic bindings still render normally but are intentionally not persisted as UI state.

For generated or template-less integrations, `:props` and `:attrs` remain available as
compatibility shorthands. Hand-authored Art should prefer values directly on the component tag.

Wrap meaningful phases with `step()` when a test has more than one interaction. Musea records each
step, user action, assertion, target element, and duration in the gallery result timeline:

```art-vue
:play="async ({ canvas, user, expect, step }) => {
  await step('increment', async () => {
    await user.click(canvas.getByRole('button', { name: 'Increment' }));
    expect(canvas.getByText('Count: 1')).toBeVisible();
  });
}"
```

Use `definePlay()` only when multiple variants share an interaction or the function is easier to
read by name:

```art-vue
<script setup lang="ts">
import CheckoutForm from "./CheckoutForm.vue";

const submits = definePlay(async ({ canvas, user, expect }) => {
  await user.click(canvas.getByRole("button", { name: "Submit" }));
  expect(canvas.getByText("Saved")).toBeVisible();
});
</script>

<art>
  <variant name="Valid" :play="submits"><CheckoutForm /></variant>
  <variant name="Prefilled" :play="submits">
    <CheckoutForm email="hello@example.com" />
  </variant>
</art>
```

### CLI and Vitest

Run only colocated play tests—without screenshot or pixel-comparison work—from the CLI:

```sh
musea-vrt test --workers 8
musea-vrt test --json --output .vize
```

The runner launches Chromium once, uses an adaptive worker pool based on available CPU parallelism
(capped at eight), and streams each result as soon as it finishes. Pass `--workers` only when you
want to override that default. JSON output stays in authored order and uses the same structured
trace as the gallery. VRT uses this play state before capture, so an interaction failure also stops
its visual snapshot.

Vitest registers every play function through `test.concurrent` when available. Its watch mode, name
filtering, per-case reporters, retries, and CI annotations therefore keep working:

```ts
import { afterAll, beforeAll, describe, test } from "vitest";
import { loadMuseaArtFiles, registerMuseaTests } from "@vizejs/vite-plugin-musea/test";

const artFiles = await loadMuseaArtFiles({ root: process.cwd() });

registerMuseaTests(
  { describe, test, beforeAll, afterAll },
  {
    artFiles,
    baseUrl: "http://localhost:5173",
    workers: 4,
  },
);
```

The configured Musea/Vite preview server must be running at `baseUrl`; Vitest owns test lifecycle
and reporting, while Musea owns component mounting, scoped queries, interactions, and trace data.

The Test tab runs the function inside the isolated preview and reports its duration and assertion
failure. `canvas` scopes role, text, label, and selector queries to that preview. `user`
supports click, double-click, clear, type, and select operations; `expect` includes common value,
DOM, visibility, text, attribute, class, and checked-state assertions.

### Play context

| Value               | API                                                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `canvas`            | `getByRole`, `getAllByRole`, `queryByRole`, text and label equivalents, `querySelector`, `querySelectorAll`               |
| `canvas.element`    | The preview root `HTMLElement`                                                                                            |
| `user`              | `click`, `dblClick`, `clear`, `type`, `selectOptions`                                                                     |
| `expect(value)`     | `toBe`, `toEqual`, truthiness, containment, DOM presence, visibility, text, value, checked state, attributes, and classes |
| `expect(value).not` | Negates any matcher                                                                                                       |

Every interaction returns a `Promise`, so authored input should be awaited. A Test-tab rerun reloads
the iframe first, preventing state from the previous run from leaking into the next one. The
compatibility alias `userEvent` points to the same typed object as `user`.

### Component inputs and variant behavior

| Surface                  | Contract                               | Purpose                                  |
| ------------------------ | -------------------------------------- | ---------------------------------------- |
| `<Target ...>` bindings  | Target component's normal Vue contract | Props and fallthrough attrs              |
| `<variant :play>`        | `MuseaPlay`                            | Inline interaction or `definePlay()`     |
| `<variant viewport>`     | `WIDTHxHEIGHT` or `WIDTHxHEIGHT@SCALE` | Per-variant VRT viewport                 |

Musea extracts statically serializable target bindings into the gallery, static export, and VRT
worker. Calls and other dynamic bindings still render through Vue but are not serialized. `play` is
syntax-checked as one TypeScript expression before it is embedded. Compatibility `:props` and
`:attrs` shorthands accept JSON-like literals and are merged before explicit target bindings.

When a variant has `play`, `musea-vrt` runs it after preview readiness and before
`capture.beforeCapture`, settling, masks, and screenshot capture. A rejected play fails that VRT
job without creating or updating its baseline. Use a variant without `play` when the baseline
should describe the initial state.

