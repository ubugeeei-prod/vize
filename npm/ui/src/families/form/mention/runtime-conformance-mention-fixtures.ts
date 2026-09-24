import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import MentionContent from "./mention-content.vue";
import MentionEditable from "./mention-editable.vue";
import MentionEmpty from "./mention-empty.vue";
import MentionInput from "./mention-input.vue";
import MentionItem from "./mention-item.vue";
import MentionRoot from "./mention-root.vue";
import type { MentionSlotState } from "./mention-types.ts";

const handles: readonly string[] = ["ada", "alan"];

function renderMentionFixture() {
  return h(
    MentionRoot<string>,
    { defaultOpen: true, defaultValue: "Ping @a", id: "composer", items: handles },
    {
      default: (state: MentionSlotState<string>) => [
        h(MentionInput, { ariaLabel: "Message", name: "message" }),
        h(MentionEditable, { ariaLabel: "Notes" }, () => "Notes"),
        h(MentionContent, null, () => [
          ...handles.map((item) => h(MentionItem<string>, { key: item, value: item }, () => item)),
          h(MentionEmpty, null, () => `Nobody (${state.query})`),
        ]),
      ],
    },
  );
}

function assertMentionServerMarkup(html: string): void {
  assert.match(html, /^<div id="composer"/);
  assert.match(html, /data-vize-ui="mention"/);
  assert.match(html, /data-vize-ui="mention-input"/);
  assert.match(html, /id="composer-field"/);
  assert.match(html, /aria-autocomplete="list"/);
  assert.match(html, /data-vize-ui="mention-editable"/);
  assert.match(html, /data-vize-ui="mention-content-host"/);
  assert.match(html, /data-state="closed"/);
  assert.match(html, />Ping @a<\/textarea>/);
}

function assertMentionHydratedDom(host: HTMLElement): void {
  const field = host.querySelector('[data-vize-ui="mention-input"]');
  assert.ok(field instanceof HTMLTextAreaElement);
  assert.equal(field.value, "Ping @a");
  assert.equal(field.name, "message");
  assert.equal(field.getAttribute("aria-autocomplete"), "list");
  const editable = host.querySelector('[data-vize-ui="mention-editable"]');
  assert.ok(editable instanceof HTMLDivElement);
  assert.equal(editable.getAttribute("contenteditable"), "true");
}

const mentionSourceFiles = [
  "mention-content.vue",
  "mention-editable.vue",
  "mention-empty.vue",
  "mention-input.vue",
  "mention-item.vue",
  "mention-root.vue",
] as const;

export const mentionRuntimeFixtures: readonly RuntimeFixture[] = mentionSourceFiles.map((file) => ({
  name: file.replace(".vue", ""),
  sourceFile: `families/form/mention/${file}`,
  render: renderMentionFixture,
  assertServerMarkup: assertMentionServerMarkup,
  assertHydratedDom: assertMentionHydratedDom,
}));
