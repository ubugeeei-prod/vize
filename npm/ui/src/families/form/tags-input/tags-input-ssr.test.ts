import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h } from "vue";
import { renderToString } from "vue/server-renderer";

import TagsInputInput from "./tags-input-input.vue";
import TagsInputItem from "./tags-input-item.vue";
import TagsInputItemDelete from "./tags-input-item-delete.vue";
import TagsInputItemText from "./tags-input-item-text.vue";
import TagsInputRoot from "./tags-input-root.vue";
import type { TagsInputSlotState } from "./tags-input-types.ts";

const SsrProbe = defineComponent({
  name: "TagsInputSsrProbe",
  setup: () => () =>
    h(
      TagsInputRoot,
      {
        ariaLabel: "Topics",
        defaultValue: ["vue", "vite"],
        editable: true,
        name: "topics",
        required: true,
      },
      {
        default: (state: TagsInputSlotState<string>) => [
          ...state.tags.map((tag, index) =>
            h(TagsInputItem, { index, key: index, value: tag }, () => [
              h(TagsInputItemText),
              h(TagsInputItemDelete, null, () => "x"),
            ]),
          ),
          h(TagsInputInput, { placeholder: "Add topic" }),
        ],
      },
    ),
});

test("renders byte-identical TagsInput markup across isolated SSR requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(SsrProbe)),
    renderToString(createSSRApp(SsrProbe)),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";

  assert.match(html, /^<div/);
  assert.match(html, /data-vize-ui="tags-input"/);
  assert.match(html, /data-state="filled"/);
  assert.match(html, /data-count="2"/);
  assert.match(html, /id="vize-v-\d+-tags-input"/);
  assert.match(html, /id="vize-v-\d+-tags-input-tag-0"/);
  assert.match(html, /aria-roledescription="tag"/);
  assert.match(html, /aria-label="Remove vite"/);
  assert.match(html, /type="hidden" name="topics" value="vue"/);
  assert.match(html, /aria-required="true"/);
});

test("hydrates tags, ids, and hidden inputs without replacing SSR nodes", async () => {
  const serverHtml = await renderToString(createSSRApp(SsrProbe));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const serverIds = [...host.querySelectorAll("[data-vize-ui='tags-input-item']")].map(
    (item) => item.id,
  );

  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(SsrProbe);
  let mounted = false;

  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    assert.deepEqual(
      [...host.querySelectorAll("[data-vize-ui='tags-input-item']")].map((item) => item.id),
      serverIds,
    );
    assert.equal(host.querySelectorAll("input[type='hidden']").length, 2);
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
