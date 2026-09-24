import assert from "node:assert/strict";

import { h } from "vue";

import TagsInputInput from "./tags-input-input.vue";
import TagsInputItem from "./tags-input-item.vue";
import TagsInputItemDelete from "./tags-input-item-delete.vue";
import TagsInputItemText from "./tags-input-item-text.vue";
import TagsInputRoot from "./tags-input-root.vue";
import type { TagsInputSlotState } from "./tags-input-types.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

function renderTagsInputFixture() {
  return h(
    TagsInputRoot,
    { ariaLabel: "Labels", defaultValue: ["bug", "docs"], id: "issue-labels", name: "labels" },
    {
      default: (state: TagsInputSlotState<string>) => [
        ...state.tags.map((tag, index) =>
          h(TagsInputItem, { index, key: index, value: tag }, () => [
            h(TagsInputItemText),
            h(TagsInputItemDelete, null, () => "x"),
          ]),
        ),
        h(TagsInputInput, { placeholder: "Add label" }),
      ],
    },
  );
}

function assertTagsInputServerMarkup(html: string): void {
  assert.match(html, /^<div/);
  assert.match(html, /data-vize-ui="tags-input"/);
  assert.match(html, /data-state="filled"/);
  assert.match(html, /id="issue-labels"/);
  assert.match(html, /id="issue-labels-tag-0"/);
  assert.match(html, /aria-label="Labels"/);
  assert.match(html, /aria-roledescription="tag"/);
  assert.match(html, /data-vize-ui="tags-input-item-text"/);
  assert.match(html, /aria-label="Remove docs"/);
  assert.match(html, /name="labels" value="bug"/);
}

function assertTagsInputHydratedDom(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="tags-input"]');
  const input = host.querySelector<HTMLInputElement>('[data-vize-ui="tags-input-input"]');
  const items = host.querySelectorAll<HTMLElement>('[data-vize-ui="tags-input-item"]');
  const hidden = host.querySelectorAll<HTMLInputElement>('input[type="hidden"][name="labels"]');

  assert.ok(root instanceof HTMLDivElement);
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.id, "issue-labels");
  assert.equal(input.getAttribute("aria-label"), "Labels");
  assert.equal(items.length, 2);
  assert.equal(items[1]?.getAttribute("aria-label"), "docs");
  assert.deepEqual(
    [...hidden].map((field) => field.value),
    ["bug", "docs"],
  );
}

const fixture = (name: string, file: string): RuntimeFixture => ({
  name,
  sourceFile: `families/form/tags-input/${file}`,
  render: renderTagsInputFixture,
  assertServerMarkup: assertTagsInputServerMarkup,
  assertHydratedDom: assertTagsInputHydratedDom,
});

export const tagsInputRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("tags-input", "tags-input-root.vue"),
  fixture("tags-input-item", "tags-input-item.vue"),
  fixture("tags-input-item-text", "tags-input-item-text.vue"),
  fixture("tags-input-item-delete", "tags-input-item-delete.vue"),
  fixture("tags-input-input", "tags-input-input.vue"),
];
