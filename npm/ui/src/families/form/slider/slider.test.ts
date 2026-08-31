import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import Slider from "./slider.vue";
import { getSliderState } from "./slider-state.ts";
import type { SliderExpose, SliderSlotState } from "./slider-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function dispatchRangeInput(input: HTMLInputElement, value: string): void {
  input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true, cancelable: true }));
}

function dispatchChange(input: HTMLInputElement): void {
  input.dispatchEvent(new Event("change", { bubbles: true, cancelable: true }));
}

function sliderFormValue(form: HTMLFormElement, name = "volume"): FormDataEntryValue | null {
  return new FormData(form).get(name);
}

test("normalizes finite bounds, step, orientation, direction, and state", () => {
  assert.deepEqual(
    getSliderState({
      value: 8.7,
      min: 0,
      max: 10,
      step: 0.5,
      orientation: "vertical",
      direction: "rtl",
      required: true,
    }),
    {
      value: 8.5,
      min: 0,
      max: 10,
      step: 0.5,
      percent: 85,
      orientation: "vertical",
      direction: "rtl",
      disabled: false,
      readOnly: false,
      required: true,
      invalid: false,
      state: "in-range",
    },
  );
  assert.deepEqual(
    getSliderState({
      value: Number.NaN,
      min: Number.NEGATIVE_INFINITY,
      max: -1,
      step: 0,
      disabled: true,
    }),
    {
      value: 0,
      min: 0,
      max: 1,
      step: 1,
      percent: 0,
      orientation: "horizontal",
      direction: "ltr",
      disabled: true,
      readOnly: false,
      required: false,
      invalid: false,
      state: "disabled",
    },
  );
});

test("renders a named native range input with form and accessibility hooks", () => {
  const handle = mountInteraction(Slider, {
    props: {
      id: "volume-slider",
      name: "volume",
      defaultValue: 40,
      min: 0,
      max: 100,
      step: 5,
      required: true,
      orientation: "vertical",
      dir: "rtl",
      ariaLabel: "Volume",
      ariaDescribedby: "volume-help",
      ariaErrormessage: "volume-error",
      ariaInvalid: true,
      ariaValueText: "40 percent",
    },
    slots: {
      default: (state: SliderSlotState) => `${state.value}:${state.orientation}:${state.direction}`,
    },
  });
  const root = handle.root();
  const input = handle.getByRole("slider", { name: "Volume" }) as HTMLInputElement;

  assert.equal(root.tagName, "SPAN");
  assert.equal(root.getAttribute("data-vize-ui"), "slider");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "invalid");
  assert.equal(root.getAttribute("data-orientation"), "vertical");
  assert.equal(root.getAttribute("data-dir"), "rtl");
  assert.equal(root.getAttribute("data-value"), "40");
  assert.equal(root.getAttribute("data-min"), "0");
  assert.equal(root.getAttribute("data-max"), "100");
  assert.equal(root.getAttribute("data-step"), "5");
  assert.equal(root.getAttribute("data-percent"), "40");
  assert.equal(root.getAttribute("data-required"), "true");
  assert.equal(root.getAttribute("data-invalid"), "true");
  assert.equal(root.style.getPropertyValue("--vize-slider-percent"), "40%");
  assert.equal(input.id, "volume-slider");
  assert.equal(input.type, "range");
  assert.equal(input.name, "volume");
  assert.equal(input.value, "40");
  assert.equal(input.min, "0");
  assert.equal(input.max, "100");
  assert.equal(input.step, "5");
  assert.equal(input.required, true);
  assert.equal(input.dir, "rtl");
  assert.equal(input.getAttribute("aria-orientation"), "vertical");
  assert.equal(input.getAttribute("aria-describedby"), "volume-help");
  assert.equal(input.getAttribute("aria-errormessage"), "volume-error");
  assert.equal(input.getAttribute("aria-invalid"), "true");
  assert.equal(input.getAttribute("aria-valuetext"), "40 percent");
  assert.equal(input.getAttribute("orient"), "vertical");
  assert.equal(input.getAttribute("data-vize-ui"), "slider-input");
  assert.equal(input.getAttribute("part"), "control");
  assert.equal(root.textContent, "40:vertical:rtl");

  handle.exposes<SliderExpose>().focus();
  assert.ok(handle.activeElement() === input, "exposed focus() must focus the range input");
  handle.unmount();
});

test("uncontrolled slider updates native form value and slot state", async () => {
  const recorded: [event: string, value: number, nativeEvent?: unknown][] = [];
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          Slider,
          {
            ariaLabel: "Volume",
            defaultValue: 25,
            max: 100,
            min: 0,
            name: "volume",
            step: 5,
            onChange: (value: number, nativeEvent: Event) =>
              recorded.push(["change", value, nativeEvent]),
            onInput: (value: number, nativeEvent: Event) =>
              recorded.push(["input", value, nativeEvent]),
            "onUpdate:modelValue": (value: number) => recorded.push(["update:modelValue", value]),
          },
          {
            default: (state: SliderSlotState) => `${state.value}:${state.percent}`,
          },
        ),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const input = handle.getByRole("slider", { name: "Volume" }) as HTMLInputElement;

  assert.equal(sliderFormValue(form), "25");
  dispatchRangeInput(input, "58");
  await nextTick();
  assert.equal(input.value, "60");
  assert.equal(sliderFormValue(form), "60");
  assert.equal(handle.root().textContent, "60:60");

  dispatchChange(input);
  await nextTick();
  assert.deepEqual(
    recorded.map(([event, value]) => [event, value]),
    [
      ["update:modelValue", 60],
      ["input", 60],
      ["change", 60],
    ],
  );
  assert.ok(recorded[1]?.[2] instanceof Event);
  assert.ok(recorded[2]?.[2] instanceof Event);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(Slider, {
    props: { ariaLabel: "Volume", modelValue: 20, name: "volume", min: 0, max: 100 },
    record: ["update:modelValue", "input"],
  });
  const input = handle.getByRole("slider") as HTMLInputElement;

  dispatchRangeInput(input, "70");
  await nextTick();

  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", 70],
      ["input", 70],
    ],
  );
  assert.equal(input.value, "20");

  await handle.wrapper.setProps({ modelValue: 70 });
  assert.equal(input.value, "70");
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(Slider, {
          ariaLabel: "Volume",
          defaultValue: 30,
          name: "volume",
        }),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const input = handle.getByRole("slider", { name: "Volume" }) as HTMLInputElement;

  assert.equal(input.value, "30");
  dispatchRangeInput(input, "80");
  await nextTick();
  assert.equal(input.value, "80");

  form.reset();
  await nextTick();
  assert.equal(input.value, "30");
  assert.equal(sliderFormValue(form), "30");
  handle.unmount();
});

test("disabled and read-only sliders keep availability semantics", async () => {
  const disabled = mountInteraction(Slider, {
    props: { ariaLabel: "Volume", defaultValue: 50, disabled: true, name: "volume" },
  });
  const disabledInput = disabled.getByRole("slider") as HTMLInputElement;
  const disabledForm = document.createElement("form");
  disabledForm.append(disabled.root());
  assert.equal(disabledInput.disabled, true);
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(new FormData(disabledForm).get("volume"), null);
  assert.ok((await disabled.tab()) === null);
  disabledForm.remove();
  disabled.unmount();

  const readOnly = mountInteraction(Slider, {
    props: { ariaLabel: "Volume", defaultValue: 50, name: "volume", readOnly: true },
    record: ["update:modelValue", "input", "change"],
  });
  const readOnlyInput = readOnly.getByRole("slider") as HTMLInputElement;
  assert.equal(readOnly.root().getAttribute("data-state"), "readonly");
  assert.equal(readOnlyInput.getAttribute("aria-readonly"), "true");
  assert.ok((await readOnly.tab()) === readOnlyInput);

  dispatchRangeInput(readOnlyInput, "90");
  dispatchChange(readOnlyInput);
  await nextTick();
  assert.equal(readOnlyInput.value, "50");
  assert.deepEqual(readOnly.recorded(), []);

  const keydown = new KeyboardEvent("keydown", {
    key: "ArrowRight",
    bubbles: true,
    cancelable: true,
  });
  readOnlyInput.dispatchEvent(keydown);
  assert.equal(keydown.defaultPrevented, true);
  readOnly.unmount();
});

test("exposes focus, setValue, stepUp, stepDown, reset, and normalized state", async () => {
  const handle = mountInteraction(Slider, {
    props: { ariaLabel: "Volume", defaultValue: 20, max: 100, min: 0, step: 5 },
    slots: {
      default: (state: SliderSlotState) => `${state.state}:${state.value}:${state.step}`,
    },
  });
  const input = handle.getByRole("slider") as HTMLInputElement;
  const exposed = handle.exposes<SliderExpose>();

  assert.equal(exposed.value, 20);
  assert.equal(exposed.min, 0);
  assert.equal(exposed.max, 100);
  assert.equal(exposed.step, 5);
  assert.equal(exposed.percent, 20);
  assert.equal(handle.root().textContent, "in-range:20:5");

  assert.equal(exposed.setValue(42), true);
  await nextTick();
  assert.equal(input.value, "40");
  assert.equal(exposed.value, 40);

  assert.equal(exposed.stepUp(3), true);
  await nextTick();
  assert.equal(input.value, "55");

  assert.equal(exposed.stepDown(20), true);
  await nextTick();
  assert.equal(input.value, "0");
  assert.equal(exposed.state, "min");

  input.blur();
  exposed.focus();
  assert.ok(handle.activeElement() === input);

  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(input.value, "20");
  handle.unmount();
});
