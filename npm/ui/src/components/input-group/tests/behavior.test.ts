import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import InputGroupAddon from "../input-group-addon.vue";
import InputGroupButton from "../input-group-button.vue";
import InputGroupInput from "../input-group-input.vue";
import InputGroupTextarea from "../input-group-textarea.vue";
import InputGroup from "../input-group.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountGroup(props: Record<string, unknown> = {}) {
  return mountInteraction(InputGroup, {
    props: { label: "Website", ...props },
    slots: {
      default: () => [
        h(InputGroupAddon, () => "https://"),
        h(InputGroupInput, { modelValue: "vize.dev", type: "url" }),
        h(InputGroupTextarea, { modelValue: "Ship it.", rows: 3 }),
        h(InputGroupButton, () => "Copy"),
      ],
    },
  });
}

test("renders addons, fields, and a button in one group", () => {
  const handle = mountGroup();
  const group = handle.getByRole("group", { name: "Website" });

  const field = group.querySelector("[data-vize-ui='input-group-input']");
  assert.equal(group.getAttribute("data-vize-ui"), "input-group");
  assert.equal(group.getAttribute("data-state"), "idle");
  assert.equal(group.querySelector("[data-vize-ui='input-group-addon']")?.textContent, "https://");
  assert.ok(field instanceof HTMLInputElement);
  assert.equal(field.type, "url");
  assert.equal(field.value, "vize.dev");
  assert.equal(
    handle.root().querySelector("[data-vize-ui='input-group-textarea']")?.getAttribute("rows"),
    "3",
  );
  assert.equal(
    handle.getByRole("button", { name: "Copy" }).getAttribute("data-vize-ui"),
    "input-group-button",
  );
  handle.unmount();
});

test("emits the next field value", async () => {
  const handle = mountInteraction(InputGroupInput, { props: { modelValue: "" } });
  const field = handle.getByRole("textbox") as HTMLInputElement;
  field.value = "vize.dev";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();

  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.[0], ["vize.dev"]);
  handle.unmount();
});

test("emits press from the group button", async () => {
  const handle = mountInteraction(InputGroupButton, { slots: { default: () => "Copy" } });
  const button = handle.getByRole("button", { name: "Copy" }) as HTMLButtonElement;

  assert.equal(button.type, "button");
  await handle.click(button);
  assert.equal(handle.wrapper.emitted("press")?.length, 1);
  assert.ok(handle.wrapper.emitted("press")?.[0]?.[0] instanceof MouseEvent);
  handle.unmount();
});

test("disables every control from the group", () => {
  const handle = mountGroup({ disabled: true });
  const group = handle.getByRole("group", { name: "Website" });
  const field = handle.root().querySelector("[data-vize-ui='input-group-input']");
  const area = handle.root().querySelector("[data-vize-ui='input-group-textarea']");
  const button = handle.getByRole("button", { name: "Copy" }) as HTMLButtonElement;

  assert.equal(group.getAttribute("aria-disabled"), "true");
  assert.equal(group.getAttribute("data-state"), "disabled");
  assert.ok(field instanceof HTMLInputElement && field.disabled);
  assert.ok(area instanceof HTMLTextAreaElement && area.disabled);
  assert.equal(button.disabled, true);
  handle.unmount();
});
