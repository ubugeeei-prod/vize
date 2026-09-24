import assert from "node:assert/strict";

import { h } from "vue";

import Editable from "./editable.vue";
import EditableInput from "./editable-input.vue";
import EditablePreview from "./editable-preview.vue";
import EditableTrigger from "./editable-trigger.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    Editable,
    { id: "title", ariaLabel: "Title", defaultValue: "Hello" },
    {
      default: () => [
        h(EditablePreview),
        h(EditableInput),
        h(EditableTrigger, { action: "edit" }, { default: () => "Edit" }),
      ],
    },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div part="root" data-vize-ui="editable"/);
  assert.match(html, /id="title-preview"/);
  assert.match(html, /data-action="edit"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const preview = host.querySelector('[data-vize-ui="editable-preview"]');
  const input = host.querySelector('[data-vize-ui="editable-input"]');
  assert.ok(preview instanceof HTMLElement);
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(preview.textContent, "Hello");
  assert.equal(input.hidden, true);
}

export const editableRuntimeFixtures: readonly RuntimeFixture[] = [
  "editable.vue",
  "editable-preview.vue",
  "editable-input.vue",
  "editable-trigger.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/editable/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
