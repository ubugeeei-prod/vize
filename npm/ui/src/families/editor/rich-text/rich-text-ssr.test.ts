import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent, h, nextTick } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  RichTextBubbleMenu,
  RichTextContent,
  RichTextRoot,
  RichTextToolbar,
  RichTextToolbarButton,
  createRichTextCommands,
} from "./rich-text.ts";
import type { RichTextDefaultSchema } from "./rich-text.ts";
import { doc, p, schema, t, ul } from "./rich-text-test-utils.ts";

const commands = createRichTextCommands(schema);
const content = doc(
  p("Hello ", t("world", "bold"), t(" <script>", "italic")),
  ul([p("item")]),
  p(t("link", { type: "link", attrs: { href: "javascript:alert(1)" } })),
);

const Probe = defineComponent({
  setup: () => () =>
    h("div", [
      h(
        RichTextRoot<RichTextDefaultSchema>,
        { id: "ssr-editor", schema, defaultValue: content },
        () => [
          h(RichTextToolbar, null, () =>
            h(
              RichTextToolbarButton,
              { command: commands.toggleMark("bold"), ariaLabel: "Bold" },
              () => "B",
            ),
          ),
          h(RichTextContent, { ariaLabel: "Body" }),
          h(RichTextBubbleMenu, null, () => "menu"),
        ],
      ),
    ]),
});

test("renders the document as static, escaped, sanitized HTML on the server", async () => {
  const [left, right] = await Promise.all([
    renderToString(createSSRApp(Probe)),
    renderToString(createSSRApp(Probe)),
  ]);
  assert.equal(left, right);
  assert.match(left, /role="textbox"/u);
  assert.match(left, /contenteditable="true"/u);
  assert.match(
    left,
    /<p data-rt-path="0" data-rt-textblock="">Hello <strong>world<\/strong><em> &lt;script&gt;<\/em><\/p>/u,
  );
  assert.match(left, /<ul data-rt-path="1"><li data-rt-path="1.0">/u);
  assert.doesNotMatch(left, /javascript:|<script>/u);
  assert.match(left, /data-vize-ui="rich-text-bubble-menu-host" part="bubble-menu-host" hidden/u);
});

test("hydrates the rendered document without mismatches", async () => {
  const html = await renderToString(createSSRApp(Probe));
  const host = document.createElement("div");
  host.innerHTML = html;
  document.body.append(host);
  const serverContent = host.querySelector('[data-vize-ui="rich-text-content"]');
  const diagnostics: string[] = [];
  const originalWarn = console.warn;
  const originalError = console.error;
  console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
  const app = createSSRApp(Probe);
  try {
    app.mount(host);
    await nextTick();
    assert.deepEqual(diagnostics, []);
    assert.ok(
      host.querySelector('[data-vize-ui="rich-text-content"]') === serverContent,
      "content node is reused",
    );
  } finally {
    app.unmount();
    host.remove();
    console.warn = originalWarn;
    console.error = originalError;
  }
});
