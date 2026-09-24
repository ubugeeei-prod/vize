import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import Editable from "./editable.vue";
import EditableInput from "./editable-input.vue";
import EditablePreview from "./editable-preview.vue";
import EditableTrigger from "./editable-trigger.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

const Probe = defineComponent({
  name: "EditableSsrProbe",
  setup: () => () =>
    h(
      Editable,
      { ariaLabel: "Title", defaultValue: "Hello", name: "title", placeholder: "Untitled" },
      {
        default: () => [
          h(EditablePreview),
          h(EditableInput),
          h(EditableTrigger, { action: "edit" }, { default: () => "Edit" }),
          h(EditableTrigger, { action: "submit" }, { default: () => "Save" }),
        ],
      },
    ),
});

test("renders byte-identical inline-edit markup and hydrates without mismatches", async () => {
  const { html, host, dispose } = await renderAndHydrate(Probe);
  try {
    assert.match(html, /^<div part="root" data-vize-ui="editable" data-state="preview"/);
    assert.match(html, /role="button" tabindex="0"/);
    assert.match(html, />(?:<!--\[-->)?Hello(?:<!--\]-->)?<\/span>/);
    assert.match(html, /type="text" value="Hello" hidden/);
    assert.match(html, /type="hidden" name="title" value="Hello"/);

    const preview = host.querySelector<HTMLElement>('[data-vize-ui="editable-preview"]');
    assert.ok(preview);
    preview.focus();
    await nextTick();
    assert.equal(host.firstElementChild?.getAttribute("data-state"), "editing");
  } finally {
    dispose();
  }
});
