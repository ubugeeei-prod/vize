import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import { Kanban } from "./kanban.ts";

type Lane = "todo" | "done";

const Probe = defineComponent({
  setup: () => () =>
    h("div", [
      h(Kanban<string, Lane>, {
        id: "board",
        ariaLabel: "Board",
        columns: [
          { id: "todo", title: "To do" },
          { id: "done", title: "Done", limit: 3 },
        ],
        modelValue: { todo: ["a", "b"], done: [] },
        getCardKey: (card: string) => card,
      }),
    ]),
});

test("renders identical board markup across SSR requests and hydrates cleanly", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /aria-roledescription="board"/);
  assert.match(left, /data-card-key="a"/);
  assert.equal(left.match(/tabindex="0"/g)?.length, 1);
  assert.doesNotMatch(left, /data-dragging/);

  const host = document.createElement("div");
  host.innerHTML = left;
  document.body.append(host);
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.deepEqual(diagnostics, []);
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
  }
});
