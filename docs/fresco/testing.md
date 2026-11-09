# Testing Fresco applications

`@vizejs/fresco/testing` mounts components through Fresco's real Vue renderer without loading the
native terminal binding. The same ESM helpers work in Vitest and in plain Node test suites.

```ts
import assert from "node:assert/strict";
import { test } from "node:test";
import { h } from "vue";
import { getByRole, renderTui } from "@vizejs/fresco/testing";

test("saves the draft", async () => {
  const rendered = renderTui(() =>
    h("box", { "aria-role": "button", "aria-label": "Save draft" }, [h("text", { text: "Save" })]),
  );

  assert.equal(getByRole(rendered.root, "button", { name: "Save draft" }).type, "box");
  assert.equal(rendered.lastFrame(), "Save");
  rendered.unmount();
});
```

Run TypeScript examples directly with `node --import tsx --test`. In Vitest, import `test` from
`vitest`; the Fresco API and assertions remain the same.

## Frames and input

`renderTui` records a plain terminal frame, deterministic screen-reader output, a serializable host
tree, and the exact native protocol nodes for every explicit capture. Its `input` driver covers
keys, printable text, paste, resize, mouse, terminal focus, and IME composition events.

```ts
const rendered = renderTui(() => h(TextInput, { modelValue: "", focused: true }));

await rendered.input.text("hello");
assert.equal(rendered.lastFrame(), "hello");
assert.equal(rendered.frameSnapshot().protocolNodes[0]?.nodeType, "root");
assert.equal(rendered.frameSnapshot().screenReaderOutput, "hello");
rendered.unmount();
```

`Static` output is retained above later live frames, matching non-interactive and CI rendering.
Always call `unmount()` so component watchers and focus registrations are released.

## Tree and layout inspection

Use [`inspectFrescoTree`](./devtools.md#layout-assertions-without-a-terminal) to join a mounted host
tree with recorded or fixture native layout rectangles. The flat immutable result exposes stable
ids, parent/child ids, depth, props, text, and an optional layout for every node. Unmatched layout ids
are reported explicitly instead of being discarded.

## Semantic queries

Use `getByRole`, `getByText`, `getByDescription`, and `getByTestId` when one result is required. The
matching `queryAllBy*` helpers return every match. Role queries can constrain accessible name,
description, and state, so tests can assert the same semantic contract exposed to assistive
technology instead of depending on renderer internals.
