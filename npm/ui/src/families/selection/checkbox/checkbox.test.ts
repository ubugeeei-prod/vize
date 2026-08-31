import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { getCheckboxState } from "./checkbox-state.ts";
import CheckboxControl from "./checkbox-control.vue";
import { mountInteraction } from "../../../testing/mount.ts";

test("gives the mixed visual state precedence", () => {
  assert.equal(getCheckboxState(false, false), "unchecked");
  assert.equal(getCheckboxState(true, false), "checked");
  assert.equal(getCheckboxState(false, true), "indeterminate");
  assert.equal(getCheckboxState(true, true), "indeterminate");
});

test("renders a native checkbox with an accessible name and focus control", () => {
  const handle = mountInteraction(CheckboxControl, { props: { ariaLabel: "Terms" } });
  const checkbox = handle.getByRole("checkbox", { name: "Terms" });

  assert.ok(checkbox instanceof HTMLInputElement);
  assert.equal(checkbox.type, "checkbox");
  assert.equal(checkbox.getAttribute("data-vize-ui"), "checkbox");
  assert.equal(checkbox.getAttribute("data-state"), "unchecked");
  assert.equal(checkbox.checked, false);

  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === checkbox, "exposed focus() must focus the input");
  handle.unmount();
});

test("reports aria-checked across unchecked, checked, and mixed states", async () => {
  const handle = mountInteraction(CheckboxControl, { props: { ariaLabel: "Terms" } });
  const checkbox = handle.getByRole("checkbox");

  assert.equal(checkbox.getAttribute("aria-checked"), "false");

  handle.exposes<{ setChecked: (value: boolean) => boolean }>().setChecked(true);
  await nextTick();
  assert.equal(checkbox.getAttribute("aria-checked"), "true");
  assert.equal(checkbox.getAttribute("data-state"), "checked");

  await handle.wrapper.setProps({ indeterminate: true });
  assert.equal(checkbox.getAttribute("aria-checked"), "mixed");
  assert.equal(checkbox.getAttribute("data-state"), "indeterminate");
  assert.equal((checkbox as HTMLInputElement).indeterminate, true);
  handle.unmount();
});

test("toggles with a pointer click and emits model before change", async () => {
  const handle = mountInteraction(CheckboxControl, {
    props: { ariaLabel: "Terms" },
    record: ["update:modelValue", "change"],
  });
  const checkbox = handle.getByRole("checkbox") as HTMLInputElement;

  await handle.click(checkbox);
  assert.equal(checkbox.checked, true);

  await handle.click(checkbox);
  assert.equal(checkbox.checked, false);

  const recorded = handle.recorded();
  assert.deepEqual(
    recorded.map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", true],
      ["change", true],
      ["update:modelValue", false],
      ["change", false],
    ],
  );
  assert.ok(recorded[1]?.payload[1] instanceof Event, "change must carry the native event");
  handle.unmount();
});

test("toggles with Space like a native checkbox", async () => {
  const handle = mountInteraction(CheckboxControl, { props: { ariaLabel: "Terms" } });
  const checkbox = handle.getByRole("checkbox") as HTMLInputElement;
  checkbox.focus();

  const space = await handle.press(checkbox, " ");
  assert.equal(space.activated, true);
  assert.equal(checkbox.checked, true);
  assert.equal(handle.wrapper.emitted("change")?.length, 1);
  handle.unmount();
});

test("clicking the associated label toggles the checkbox", async () => {
  const changes: boolean[] = [];
  const LabeledCheckbox = defineComponent({
    setup: () => () =>
      h("label", [
        h(CheckboxControl, { onChange: (value: boolean) => changes.push(value) }),
        "Agree",
      ]),
  });
  const handle = mountInteraction(LabeledCheckbox);
  const checkbox = handle.getByRole("checkbox", { name: "Agree" }) as HTMLInputElement;

  await handle.click(handle.root());
  assert.equal(checkbox.checked, true);
  assert.deepEqual(changes, [true]);
  handle.unmount();
});

test("controlled: the parent-provided value always wins", async () => {
  const handle = mountInteraction(CheckboxControl, {
    props: { ariaLabel: "Terms", modelValue: false },
    record: ["update:modelValue"],
  });
  const checkbox = handle.getByRole("checkbox") as HTMLInputElement;

  await handle.click(checkbox);
  await nextTick();
  assert.deepEqual(
    handle.recorded().map((emit) => emit.payload[0]),
    [true],
  );
  assert.equal(checkbox.checked, false, "an unaccepted request must revert to the prop value");

  await handle.wrapper.setProps({ modelValue: true });
  assert.equal(checkbox.checked, true);
  assert.equal(checkbox.getAttribute("aria-checked"), "true");
  handle.unmount();
});

test("uncontrolled: defaultChecked seeds state and reset restores it", async () => {
  const handle = mountInteraction(CheckboxControl, {
    props: { ariaLabel: "Terms", defaultChecked: true },
  });
  const checkbox = handle.getByRole("checkbox") as HTMLInputElement;
  assert.equal(checkbox.checked, true);

  await handle.click(checkbox);
  assert.equal(checkbox.checked, false);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[false]]);

  handle.exposes<{ reset: () => boolean }>().reset();
  await nextTick();
  assert.equal(checkbox.checked, true, "reset must restore the default state");
  handle.unmount();
});

test("indeterminate announces mixed and requests clearing on toggle", async () => {
  const handle = mountInteraction(CheckboxControl, {
    props: { ariaLabel: "Terms", indeterminate: true },
    record: ["update:indeterminate", "change"],
  });
  const checkbox = handle.getByRole("checkbox") as HTMLInputElement;
  assert.equal(checkbox.getAttribute("aria-checked"), "mixed");
  await nextTick();
  assert.equal(checkbox.indeterminate, true, "the mixed state must reach the native element");

  await handle.click(checkbox);
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:indeterminate", false],
      ["change", true],
    ],
  );

  await handle.wrapper.setProps({ indeterminate: false });
  assert.equal(checkbox.getAttribute("aria-checked"), "true");
  assert.equal(checkbox.indeterminate, false);
  handle.unmount();
});

test("disabled checkbox ignores pointer and keyboard activation", async () => {
  const handle = mountInteraction(CheckboxControl, {
    props: { ariaLabel: "Terms", disabled: true },
  });
  const checkbox = handle.getByRole("checkbox") as HTMLInputElement;
  assert.ok(checkbox.hasAttribute("disabled"));

  await handle.click(checkbox);
  const space = await handle.press(checkbox, " ");
  assert.equal(space.activated, false);
  assert.equal(checkbox.checked, false);
  assert.equal(handle.wrapper.emitted("change"), undefined);
  assert.ok((await handle.tab()) === null, "a disabled checkbox must leave the tab order");
  handle.unmount();
});
