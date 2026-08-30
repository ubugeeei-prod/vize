import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import type { RadioGroupExpose, RadioGroupItemExpose } from "./radio-group.ts";
import RadioGroup from "./radio-group.vue";
import RadioGroupItem from "./radio-group-item.vue";
import { mountInteraction } from "./testing/mount.ts";

function mountRadioGroup(
  props: Record<string, unknown> = {},
  itemProps: Record<string, unknown> = {},
) {
  return mountInteraction(RadioGroup, {
    props,
    slots: {
      default: (state) => [
        h("output", { "data-group-state": state.state }, String(state.value ?? "")),
        h("label", [
          h(RadioGroupItem, { id: "daily-radio", value: "daily", ...itemProps }),
          "Daily",
        ]),
        h("label", [h(RadioGroupItem, { id: "weekly-radio", value: "weekly" }), "Weekly"]),
      ],
    },
  });
}

function radioFormValue(form: HTMLFormElement, name = "frequency"): FormDataEntryValue | null {
  return new FormData(form).get(name);
}

test("renders native radio group semantics with form and accessibility attributes", () => {
  const handle = mountRadioGroup({
    ariaDescribedby: "frequency-help",
    ariaErrormessage: "frequency-error",
    ariaInvalid: true,
    ariaLabel: "Email frequency",
    defaultValue: "daily",
    id: "frequency",
    name: "frequency",
    orientation: "horizontal",
    required: true,
  });
  const group = handle.getByRole("radiogroup", { name: "Email frequency" });
  const daily = handle.getByRole("radio", { name: "Daily" }) as HTMLInputElement;
  const weekly = handle.getByRole("radio", { name: "Weekly" }) as HTMLInputElement;

  assert.equal(group.id, "frequency");
  assert.equal(group.getAttribute("aria-describedby"), "frequency-help");
  assert.equal(group.getAttribute("aria-errormessage"), "frequency-error");
  assert.equal(group.getAttribute("aria-invalid"), "true");
  assert.equal(group.getAttribute("aria-orientation"), "horizontal");
  assert.equal(group.getAttribute("aria-required"), "true");
  assert.equal(group.getAttribute("data-vize-ui"), "radio-group");
  assert.equal(group.getAttribute("part"), "root");
  assert.equal(group.getAttribute("data-state"), "selected");
  assert.equal(group.getAttribute("data-orientation"), "horizontal");
  assert.equal(group.getAttribute("data-value"), "daily");
  assert.equal(
    group.querySelector("[data-group-state]")?.getAttribute("data-group-state"),
    "selected",
  );
  assert.equal(daily.type, "radio");
  assert.equal(daily.id, "daily-radio");
  assert.equal(daily.name, "frequency");
  assert.equal(daily.value, "daily");
  assert.equal(daily.checked, true);
  assert.equal(daily.required, true);
  assert.equal(daily.getAttribute("data-state"), "checked");
  assert.equal(daily.getAttribute("data-checked"), "true");
  assert.equal(daily.getAttribute("data-invalid"), "true");
  assert.equal(weekly.checked, false);
  handle.unmount();
});

test("uncontrolled radio group selects one item and submits its value", async () => {
  const recorded: [event: string, value: unknown, previous?: unknown, nativeEvent?: unknown][] = [];
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          RadioGroup,
          {
            ariaLabel: "Email frequency",
            name: "frequency",
            onChange: (value: string, previous: unknown, nativeEvent: Event) =>
              recorded.push(["change", value, previous, nativeEvent]),
            "onUpdate:modelValue": (value: unknown) => recorded.push(["update:modelValue", value]),
          },
          () => [
            h("label", [h(RadioGroupItem, { value: "daily" }), "Daily"]),
            h("label", [h(RadioGroupItem, { value: "weekly" }), "Weekly"]),
          ],
        ),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const daily = handle.getByRole("radio", { name: "Daily" }) as HTMLInputElement;
  const weekly = handle.getByRole("radio", { name: "Weekly" }) as HTMLInputElement;

  assert.equal(radioFormValue(form), null);
  await handle.click(weekly);
  assert.equal(weekly.checked, true);
  assert.equal(daily.checked, false);
  assert.equal(radioFormValue(form), "weekly");

  await handle.click(daily);
  assert.equal(daily.checked, true);
  assert.equal(weekly.checked, false);
  assert.equal(radioFormValue(form), "daily");

  assert.deepEqual(
    recorded.map(([event, value, previous]) => [event, value, previous]),
    [
      ["update:modelValue", "weekly", undefined],
      ["change", "weekly", null],
      ["update:modelValue", "daily", undefined],
      ["change", "daily", "weekly"],
    ],
  );
  assert.ok(recorded[1]?.[3] instanceof Event);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountRadioGroup(
    {
      ariaLabel: "Email frequency",
      modelValue: "daily",
      name: "frequency",
    },
    {},
  );
  const daily = handle.getByRole("radio", { name: "Daily" }) as HTMLInputElement;
  const weekly = handle.getByRole("radio", { name: "Weekly" }) as HTMLInputElement;

  await handle.click(weekly);
  await nextTick();

  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [["weekly"]]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(0, 2), ["weekly", "daily"]);
  assert.equal(daily.checked, true);
  assert.equal(weekly.checked, false);

  await handle.wrapper.setProps({ modelValue: "weekly" });
  assert.equal(daily.checked, false);
  assert.equal(weekly.checked, true);
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          RadioGroup,
          {
            ariaLabel: "Email frequency",
            defaultValue: "daily",
            name: "frequency",
          },
          () => [
            h("label", [h(RadioGroupItem, { value: "daily" }), "Daily"]),
            h("label", [h(RadioGroupItem, { value: "weekly" }), "Weekly"]),
          ],
        ),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const daily = handle.getByRole("radio", { name: "Daily" }) as HTMLInputElement;
  const weekly = handle.getByRole("radio", { name: "Weekly" }) as HTMLInputElement;

  assert.equal(daily.checked, true);
  assert.equal(radioFormValue(form), "daily");
  await handle.click(weekly);
  assert.equal(weekly.checked, true);
  assert.equal(radioFormValue(form), "weekly");

  form.reset();
  await nextTick();
  assert.equal(daily.checked, true);
  assert.equal(weekly.checked, false);
  assert.equal(radioFormValue(form), "daily");
  handle.unmount();
});

test("keyboard activation and exposed focus follow native radio behavior", async () => {
  let groupExpose: RadioGroupExpose | null = null;
  let weeklyExpose: RadioGroupItemExpose | null = null;
  const Probe = defineComponent({
    setup: () => () =>
      h(
        RadioGroup,
        {
          ariaLabel: "Email frequency",
          defaultValue: "daily",
          ref: (value) => {
            groupExpose = value as RadioGroupExpose | null;
          },
        },
        () => [
          h("label", [h(RadioGroupItem, { value: "daily" }), "Daily"]),
          h("label", [
            h(RadioGroupItem, {
              ref: (value) => {
                weeklyExpose = value as RadioGroupItemExpose | null;
              },
              value: "weekly",
            }),
            "Weekly",
          ]),
        ],
      ),
  });
  const handle = mountInteraction(Probe);
  const daily = handle.getByRole("radio", { name: "Daily" }) as HTMLInputElement;
  const weekly = handle.getByRole("radio", { name: "Weekly" }) as HTMLInputElement;

  if (groupExpose === null || weeklyExpose === null)
    assert.fail("RadioGroup refs must expose state");

  groupExpose.focus();
  assert.ok(handle.activeElement() === daily);

  weekly.focus();
  const space = await handle.press(weekly, " ");
  assert.equal(space.activated, true);
  assert.equal(weekly.checked, true);
  assert.equal(groupExpose.value, "weekly");
  assert.equal(weeklyExpose.checked, true);

  assert.equal(groupExpose.reset(), true);
  await nextTick();
  assert.equal(groupExpose.value, "daily");
  assert.equal(daily.checked, true);

  assert.equal(groupExpose.setValue(null), true);
  await nextTick();
  assert.equal(daily.checked, false);
  assert.equal(weekly.checked, false);
  handle.unmount();
});

test("disabled groups and disabled items keep native availability semantics", async () => {
  const DisabledProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          RadioGroup,
          {
            ariaLabel: "Email frequency",
            defaultValue: "daily",
            disabled: true,
            name: "frequency",
          },
          () => [
            h("label", [h(RadioGroupItem, { value: "daily" }), "Daily"]),
            h("label", [h(RadioGroupItem, { value: "weekly" }), "Weekly"]),
          ],
        ),
      ]),
  });
  const groupDisabled = mountInteraction(DisabledProbe);
  const disabledForm = groupDisabled.root() as HTMLFormElement;
  const disabledRoot = groupDisabled.getByRole("radiogroup", { name: "Email frequency" });
  const disabledDaily = groupDisabled.getByRole("radio", { name: "Daily" }) as HTMLInputElement;

  assert.equal(disabledRoot.getAttribute("aria-disabled"), "true");
  assert.equal(disabledRoot.getAttribute("data-state"), "disabled");
  assert.equal(disabledDaily.disabled, true);
  assert.equal(disabledDaily.getAttribute("data-state"), "disabled");
  assert.equal(radioFormValue(disabledForm), null);
  await groupDisabled.click(disabledDaily);
  assert.equal(disabledDaily.checked, true);
  assert.equal(groupDisabled.wrapper.emitted("change"), undefined);
  assert.ok((await groupDisabled.tab()) === null);
  groupDisabled.unmount();

  const itemDisabled = mountRadioGroup({ ariaLabel: "Email frequency" }, { disabled: true });
  const disabledItem = itemDisabled.getByRole("radio", { name: "Daily" }) as HTMLInputElement;
  const enabledItem = itemDisabled.getByRole("radio", { name: "Weekly" }) as HTMLInputElement;

  assert.equal(disabledItem.disabled, true);
  assert.equal(disabledItem.getAttribute("data-disabled"), "true");
  assert.ok((await itemDisabled.tab()) === enabledItem);
  itemDisabled.unmount();
});

test("items require a matching group provider", () => {
  assert.throws(
    () => mountInteraction(RadioGroupItem, { props: { value: "daily" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
