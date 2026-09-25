import assert from "node:assert/strict";

import { h } from "vue";
import type { VNode } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { createRichTextCommands } from "./rich-text-commands.ts";
import { richTextDocFromText } from "./rich-text-model.ts";
import { createDefaultRichTextSchema } from "./rich-text-schema.ts";
import type { RichTextDefaultSchema } from "./rich-text-schema.ts";
import RichTextBubbleMenu from "./rich-text-bubble-menu.vue";
import RichTextContent from "./rich-text-content.vue";
import RichTextRoot from "./rich-text-root.vue";
import RichTextToolbar from "./rich-text-toolbar.vue";
import RichTextToolbarButton from "./rich-text-toolbar-button.vue";

const schema = createDefaultRichTextSchema();
const commands = createRichTextCommands(schema);

function editor(): VNode {
  return h("div", [
    h(
      RichTextRoot<RichTextDefaultSchema>,
      { id: "runtime-editor", schema, defaultValue: richTextDocFromText(schema, "Hello\nWorld") },
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
  ]);
}

function fixture(file: string, name: string, check: (host: HTMLElement) => void): RuntimeFixture {
  return {
    name,
    sourceFile: `families/editor/rich-text/${file}`,
    render: editor,
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="${name}"`));
      assert.match(html, /<p data-rt-path="1" data-rt-textblock="">World<\/p>/u);
    },
    assertHydratedDom(host) {
      const element = host.querySelector(`[data-vize-ui="${name}"]`);
      assert.ok(element instanceof HTMLElement, `${name} must hydrate`);
      check(element);
    },
  };
}

export const richTextRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("rich-text-bubble-menu.vue", "rich-text-bubble-menu-host", (host) =>
    assert.equal(host.hidden, true),
  ),
  fixture("rich-text-content.vue", "rich-text-content", (content) =>
    assert.equal(content.getAttribute("role"), "textbox"),
  ),
  fixture("rich-text-root.vue", "rich-text", (root) => assert.equal(root.id, "runtime-editor")),
  fixture("rich-text-toolbar.vue", "rich-text-toolbar", (toolbar) =>
    assert.equal(toolbar.getAttribute("aria-controls"), "runtime-editor-content"),
  ),
  fixture("rich-text-toolbar-button.vue", "rich-text-toolbar-button", (button) =>
    assert.equal(button.getAttribute("tabindex"), "0"),
  ),
];
