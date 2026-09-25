import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import type { Component } from "vue";
import { renderToString } from "vue/server-renderer";

import MentionContent from "./mention-content.vue";
import MentionEditable from "./mention-editable.vue";
import MentionEmpty from "./mention-empty.vue";
import MentionInput from "./mention-input.vue";
import MentionItem from "./mention-item.vue";
import MentionRoot from "./mention-root.vue";
import type { MentionSlotState } from "./mention-types.ts";

function probe(editable: boolean): Component {
  return defineComponent({
    name: "MentionSsrProbe",
    setup: () => () =>
      h(
        MentionRoot<string>,
        { defaultValue: "Hello @ada", items: ["ada", "alan"] },
        {
          default: (state: MentionSlotState<string>) => [
            editable
              ? h(MentionEditable, { ariaLabel: "Notes" }, () => "Hello @ada")
              : h(MentionInput, { ariaLabel: "Message", name: "message" }),
            h(MentionContent, null, () => [
              ...state.filteredItems.map((item) =>
                h(MentionItem<string>, { key: item, value: item }, () => item),
              ),
              h(MentionEmpty, null, () => "Nobody"),
            ]),
          ],
        },
      ),
  });
}

async function assertHydrates(component: Component): Promise<void> {
  const serverHtml = await renderToString(createSSRApp(component));
  const host = document.createElement("div");
  host.innerHTML = serverHtml;
  document.body.append(host);
  const serverRoot = host.firstElementChild;
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(component);
  let mounted = false;
  try {
    app.mount(host);
    mounted = true;
    assert.ok(host.firstElementChild === serverRoot);
    await nextTick();
    assert.deepEqual(diagnostics, []);
  } finally {
    if (mounted) app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
}

test("renders byte-identical Mention textarea markup across isolated requests", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(false))),
    renderToString(createSSRApp(probe(false))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  const html = outputs[0] ?? "";
  assert.match(html, /data-vize-ui="mention"/);
  assert.match(html, /<textarea[^>]*id="vize-v-\d+-mention-field"/);
  assert.match(html, /aria-autocomplete="list"/);
  assert.match(html, /name="message"/);
  assert.match(html, />Hello @ada<\/textarea>/);
  assert.doesNotMatch(html, /role="listbox"/, "no token is active on the server");
});

test("renders byte-identical contenteditable markup", async () => {
  const outputs = await Promise.all([
    renderToString(createSSRApp(probe(true))),
    renderToString(createSSRApp(probe(true))),
  ]);
  assert.equal(outputs[0], outputs[1]);
  assert.match(outputs[0] ?? "", /contenteditable="true"[^>]*role="textbox"/);
});

test("hydrates textarea and contenteditable Mentions without mismatches", async () => {
  await assertHydrates(probe(false));
  await assertHydrates(probe(true));
});
