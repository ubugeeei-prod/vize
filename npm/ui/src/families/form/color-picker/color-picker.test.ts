import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import type {
  ColorPickerEyeDropperExpose,
  ColorPickerFieldExpose,
  ColorPickerRootExpose,
} from "./color-picker.ts";
import ColorPickerArea from "./color-picker-area.vue";
import ColorPickerChannelSlider from "./color-picker-channel-slider.vue";
import ColorPickerEyeDropper from "./color-picker-eye-dropper.vue";
import ColorPickerField from "./color-picker-field.vue";
import ColorPickerRoot from "./color-picker-root.vue";
import ColorPickerSwatch from "./color-picker-swatch.vue";
import ColorPickerSwatchGroup from "./color-picker-swatch-group.vue";
import { mountInteraction } from "../../../testing/mount.ts";

type Slots = Record<string, unknown>;

function mountPicker(props: Record<string, unknown>, slots: Slots) {
  return mountInteraction(ColorPickerRoot, {
    props,
    record: ["update:modelValue", "change", "commit"],
    slots,
  });
}

function key(target: Element, value: string, init: KeyboardEventInit = {}): KeyboardEvent {
  const event = new KeyboardEvent("keydown", {
    key: value,
    bubbles: true,
    cancelable: true,
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

function pointer(target: Element, type: string, init: PointerEventInit): void {
  target.dispatchEvent(
    new PointerEvent(type, { bubbles: true, cancelable: true, pointerId: 1, button: 0, ...init }),
  );
}

function stubRect(element: Element, rect: { width: number; height: number }): void {
  element.getBoundingClientRect = () =>
    ({
      x: 0,
      y: 0,
      left: 0,
      top: 0,
      right: rect.width,
      bottom: rect.height,
      width: rect.width,
      height: rect.height,
      toJSON: () => ({}),
    }) satisfies DOMRect;
}

function lastValue(handle: ReturnType<typeof mountPicker>): unknown {
  return handle.wrapper.emitted("update:modelValue")?.at(-1)?.[0];
}

test("renders a labelled root with color custom properties, slot state, and form value", () => {
  const handle = mountPicker(
    { defaultValue: "#3366cc", id: "brand", name: "brand-color" },
    {
      default: (state: { value: string; state: string }) =>
        h("output", { "data-slot-state": state.state }, state.value),
    },
  );
  const root = handle.root();
  const hidden = root.querySelector<HTMLInputElement>("input[type='hidden']");

  assert.equal(root.id, "brand");
  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("data-vize-ui"), "color-picker-root");
  assert.equal(root.getAttribute("data-state"), "interactive");
  assert.equal(root.getAttribute("data-value"), "#3366cc");
  assert.equal(root.style.getPropertyValue("--vize-ui-color-picker-color"), "rgb(51 102 204)");
  assert.equal(root.style.getPropertyValue("--vize-ui-color-picker-hue"), "hsl(220 100% 50%)");
  assert.equal(root.style.getPropertyValue("--vize-ui-color-picker-alpha"), "1");
  assert.equal(root.querySelector("output")?.textContent, "#3366cc");
  assert.equal(root.querySelector("output")?.getAttribute("data-slot-state"), "interactive");
  assert.equal(hidden?.name, "brand-color");
  assert.equal(hidden?.value, "#3366cc");
  handle.unmount();
});

test("area maps pointer drags to saturation and brightness with a committed release", async () => {
  const handle = mountPicker(
    { defaultValue: "hsb(200 10% 10%)", format: "hsb" },
    { default: () => h(ColorPickerArea) },
  );
  const area = handle.root().querySelector<HTMLDivElement>("[data-vize-ui='color-picker-area']");
  const thumb = handle.getByRole("slider", { name: "Saturation and Brightness" });
  assert.ok(area);
  stubRect(area, { width: 200, height: 100 });

  pointer(area, "pointerdown", { clientX: 50, clientY: 25 });
  await nextTick();
  assert.equal(lastValue(handle), "hsb(200 25% 75%)");
  assert.ok(handle.activeElement() === thumb);
  assert.equal(area.getAttribute("data-dragging"), "true");
  pointer(area, "pointermove", { clientX: 400, clientY: -40 });
  await nextTick();
  assert.equal(lastValue(handle), "hsb(200 100% 100%)");
  assert.equal(area.style.getPropertyValue("--vize-ui-color-picker-thumb-x"), "100%");
  assert.equal(area.style.getPropertyValue("--vize-ui-color-picker-thumb-y"), "0%");
  assert.equal(handle.wrapper.emitted("commit"), undefined);
  pointer(area, "pointerup", { clientX: 400, clientY: -40 });
  await nextTick();
  assert.deepEqual(handle.wrapper.emitted("commit")?.[0]?.[0], "hsb(200 100% 100%)");
  assert.equal(area.getAttribute("data-dragging"), null);
  const detail = handle.wrapper.emitted("change")?.[0]?.[1];
  assert.ok(typeof detail === "object" && detail !== null);
  assert.equal(Reflect.get(detail, "source"), "area");
  handle.unmount();
});

test("area thumb exposes 2D slider semantics and keyboard steps", async () => {
  const handle = mountPicker(
    { defaultValue: "hsb(10 50% 50%)", format: "hsb" },
    { default: () => h(ColorPickerArea) },
  );
  const thumb = handle.getByRole("slider", { name: "Saturation and Brightness" });
  assert.equal(thumb.getAttribute("aria-roledescription"), "2D slider");
  assert.equal(thumb.getAttribute("aria-valuenow"), "50");
  assert.equal(thumb.getAttribute("aria-valuetext"), "Saturation 50%, Brightness 50%");
  assert.equal(thumb.tabIndex, 0);

  assert.equal(key(thumb, "ArrowRight").defaultPrevented, true);
  await nextTick();
  assert.equal(lastValue(handle), "hsb(10 51% 50%)");
  key(thumb, "ArrowUp", { shiftKey: true });
  await nextTick();
  assert.equal(lastValue(handle), "hsb(10 51% 60%)");
  key(thumb, "PageDown");
  await nextTick();
  assert.equal(lastValue(handle), "hsb(10 51% 50%)");
  key(thumb, "End");
  await nextTick();
  assert.equal(lastValue(handle), "hsb(10 100% 50%)");
  key(thumb, "Home");
  await nextTick();
  assert.equal(lastValue(handle), "hsb(10 0% 50%)");
  assert.equal(handle.wrapper.emitted("commit")?.length, 5);
  assert.equal(key(thumb, "a").defaultPrevented, false);
  handle.unmount();
});

test("area keeps the hue while dragging through greys and supports custom channels", async () => {
  const handle = mountPicker(
    { defaultValue: "hsb(200 50% 50%)", format: "hsb" },
    {
      default: () => [
        h(ColorPickerArea, { xChannel: "hue", yChannel: "lightness", ariaLabel: "Hue area" }),
        h(ColorPickerArea, { ariaLabel: "Tone" }),
      ],
    },
  );
  const tone = handle.getByRole("slider", { name: "Tone" });
  key(tone, "Home");
  await nextTick();
  assert.equal(lastValue(handle), "hsb(200 0% 50%)");
  key(tone, "End");
  await nextTick();
  assert.equal(lastValue(handle), "hsb(200 100% 50%)", "hue survives a grey intermediate");
  const hueArea = handle.getByRole("slider", { name: "Hue area" });
  assert.equal(hueArea.getAttribute("aria-valuemax"), "360");
  key(hueArea, "ArrowRight");
  await nextTick();
  assert.equal(lastValue(handle), "hsb(201 100% 50%)");
  handle.unmount();
});

test("channel sliders edit hue, alpha, and rgb channels with keyboard and pointer", async () => {
  const handle = mountPicker(
    { defaultValue: "#ff0000", format: "rgb" },
    {
      default: () => [
        h(ColorPickerChannelSlider, { channel: "hue" }),
        h(ColorPickerChannelSlider, { channel: "alpha", orientation: "vertical" }),
        h(ColorPickerChannelSlider, { channel: "green", space: "rgb", ariaLabel: "Green" }),
      ],
    },
  );
  const hue = handle.getByRole("slider", { name: "Hue" });
  const alpha = handle.getByRole("slider", { name: "Alpha" });
  const green = handle.getByRole("slider", { name: "Green" });
  assert.equal(hue.getAttribute("aria-valuemax"), "360");
  assert.equal(hue.getAttribute("aria-valuetext"), "Hue 0°");
  assert.equal(alpha.getAttribute("aria-orientation"), "vertical");
  assert.equal(alpha.getAttribute("aria-valuenow"), "1");

  key(hue, "ArrowRight", { shiftKey: true });
  await nextTick();
  assert.equal(lastValue(handle), "rgb(255 64 0)");
  key(alpha, "ArrowDown");
  await nextTick();
  assert.equal(lastValue(handle), "rgb(255 64 0 / 0.99)");
  key(alpha, "PageDown");
  await nextTick();
  assert.equal(lastValue(handle), "rgb(255 64 0 / 0.89)");
  key(green, "End");
  await nextTick();
  assert.equal(lastValue(handle), "rgb(255 255 0 / 0.89)");

  const track = handle
    .root()
    .querySelector<HTMLDivElement>(
      "[data-vize-ui='color-picker-channel-slider'][data-channel='alpha']",
    );
  assert.ok(track);
  stubRect(track, { width: 10, height: 100 });
  pointer(track, "pointerdown", { clientX: 5, clientY: 75 });
  pointer(track, "pointerup", { clientX: 5, clientY: 75 });
  await nextTick();
  assert.equal(lastValue(handle), "rgb(255 255 0 / 0.25)");
  assert.equal(track.style.getPropertyValue("--vize-ui-color-picker-thumb-percent"), "25%");
  assert.match(
    track.style.getPropertyValue("--vize-ui-color-picker-track-background"),
    /^linear-gradient\(to top, rgb\(255 255 0 \/ 0\), rgb\(255 255 0\)\)$/,
  );
  handle.unmount();
});

test("right-to-left layouts invert horizontal arrows and pointer mapping", async () => {
  const handle = mountPicker(
    { defaultValue: "hsb(100 50% 50%)", format: "hsb", dir: "rtl" },
    { default: () => h(ColorPickerChannelSlider, { channel: "hue" }) },
  );
  const hue = handle.getByRole("slider", { name: "Hue" });
  key(hue, "ArrowRight");
  await nextTick();
  assert.equal(lastValue(handle), "hsb(99 50% 50%)");
  const track = handle.root().querySelector("[data-vize-ui='color-picker-channel-slider']");
  assert.ok(track);
  stubRect(track, { width: 360, height: 10 });
  pointer(track, "pointerdown", { clientX: 90, clientY: 5 });
  await nextTick();
  assert.equal(lastValue(handle), "hsb(270 50% 50%)");
  handle.unmount();
});

test("swatches form a roving radiogroup that selects on click and arrow keys", async () => {
  const handle = mountPicker(
    { defaultValue: "#00ff00" },
    {
      default: () =>
        h(ColorPickerSwatchGroup, { ariaLabel: "Presets" }, () => [
          h(ColorPickerSwatch, { value: "#ff0000", label: "Red" }),
          h(ColorPickerSwatch, { value: "rgb(0 255 0)", label: "Green" }),
          h(ColorPickerSwatch, { value: "#0000ff", label: "Blue", disabled: true }),
          h(ColorPickerSwatch, { value: "hsl(60 100% 50%)" }),
        ]),
    },
  );
  const group = handle.getByRole("radiogroup", { name: "Presets" });
  const red = handle.getByRole("radio", { name: "Red" });
  const green = handle.getByRole("radio", { name: "Green" });
  const blue = handle.getByRole("radio", { name: "Blue" });
  const yellow = handle.getByRole("radio", { name: "#ffff00" });
  await nextTick();

  assert.equal(group.id.endsWith("-swatches"), true);
  assert.equal(green.getAttribute("aria-checked"), "true");
  assert.equal(green.getAttribute("data-state"), "checked");
  assert.equal(green.tabIndex, 0);
  assert.equal(red.getAttribute("tabindex"), "-1");
  assert.equal(blue.getAttribute("tabindex"), "-1");
  assert.equal(blue.getAttribute("aria-disabled"), "true");
  assert.equal(red.style.getPropertyValue("--vize-ui-color-picker-swatch-color"), "rgb(255 0 0)");

  assert.ok((await handle.tab()) === green);
  await handle.press(green, "ArrowRight");
  assert.ok(handle.activeElement() === yellow, "disabled swatches are skipped");
  assert.equal(lastValue(handle), "#ffff00");
  assert.equal(yellow.getAttribute("aria-checked"), "true");
  await handle.click(red);
  assert.equal(lastValue(handle), "#ff0000");
  await handle.click(blue);
  assert.equal(lastValue(handle), "#ff0000");
  key(green, " ");
  await nextTick();
  assert.equal(lastValue(handle), "#00ff00");
  handle.unmount();
});

test("swatch click can be cancelled before selection", async () => {
  const handle = mountPicker(
    { defaultValue: "#000000" },
    {
      default: () =>
        h(ColorPickerSwatchGroup, { ariaLabel: "Presets" }, () =>
          h(ColorPickerSwatch, {
            value: "#ff0000",
            label: "Red",
            onClick: (event: MouseEvent) => event.preventDefault(),
          }),
        ),
    },
  );
  await handle.click(handle.getByRole("radio", { name: "Red" }));
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("field parses on Enter and blur, flags invalid drafts, and reverts them", async () => {
  const invalid: string[] = [];
  const fieldRef = ref<ColorPickerFieldExpose | null>(null);
  const handle = mountPicker(
    { defaultValue: "#336699" },
    {
      default: () =>
        h(ColorPickerField, {
          ariaLabel: "Hex",
          ref: fieldRef,
          onInvalid: (draft: string) => invalid.push(draft),
        }),
    },
  );
  const field = handle.getByRole("textbox", { name: "Hex" });
  assert.ok(field instanceof HTMLInputElement);
  assert.equal(field.value, "#336699");

  field.value = "rgb(255 0 0)";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
  assert.equal(field.getAttribute("data-state"), "valid");
  key(field, "Enter");
  await nextTick();
  assert.equal(lastValue(handle), "#ff0000");
  assert.equal(field.value, "#ff0000");
  assert.equal(handle.wrapper.emitted("commit")?.length, 1);

  field.value = "nope";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
  assert.equal(field.getAttribute("aria-invalid"), "true");
  assert.equal(fieldRef.value?.state, "invalid");
  field.dispatchEvent(new FocusEvent("blur"));
  await nextTick();
  assert.deepEqual(invalid, ["nope"]);
  assert.equal(field.value, "#ff0000");
  assert.equal(field.getAttribute("aria-invalid"), null);

  field.value = "#00f";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  key(field, "Escape");
  await nextTick();
  assert.equal(field.value, "#ff0000");
  field.value = "hsl(120 100% 25%)";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  assert.equal(fieldRef.value?.commit(), true);
  await nextTick();
  assert.equal(lastValue(handle), "#008000");
  handle.unmount();
});

test("field can preserve the current alpha and follow its own format", async () => {
  const handle = mountPicker(
    { defaultValue: "#ff000080" },
    {
      default: () =>
        h(ColorPickerField, { ariaLabel: "Color", format: "rgb", preserveAlpha: true }),
    },
  );
  const field = handle.getByRole("textbox", { name: "Color" });
  assert.ok(field instanceof HTMLInputElement);
  assert.equal(field.value, "rgb(255 0 0 / 0.502)");
  field.value = "#00ff00";
  field.dispatchEvent(new Event("input", { bubbles: true }));
  key(field, "Enter");
  await nextTick();
  assert.equal(lastValue(handle), "#00ff0080");
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountPicker(
    { modelValue: "#000000", format: "hex" },
    { default: () => h(ColorPickerChannelSlider, { channel: "brightness" }) },
  );
  const slider = handle.getByRole("slider", { name: "Brightness" });
  key(slider, "End");
  await nextTick();
  assert.equal(lastValue(handle), "#ffffff");
  assert.equal(handle.root().getAttribute("data-value"), "#000000");
  await handle.wrapper.setProps({ modelValue: "#ffffff" });
  assert.equal(handle.root().getAttribute("data-value"), "#ffffff");
  await handle.wrapper.setProps({ modelValue: "not a color" });
  assert.equal(
    handle.root().getAttribute("data-value"),
    "#ffffff",
    "invalid input keeps last color",
  );
  handle.unmount();
});

test("disabled and read-only roots suppress edits while read-only stays focusable", async () => {
  const disabled = mountPicker(
    { defaultValue: "#123456", disabled: true, name: "c" },
    {
      default: () => [
        h(ColorPickerArea),
        h(ColorPickerChannelSlider, { channel: "hue" }),
        h(ColorPickerField, { ariaLabel: "Hex" }),
      ],
    },
  );
  const area = disabled.getByRole("slider", { name: "Saturation and Brightness" });
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(area.getAttribute("tabindex"), "-1");
  assert.equal(area.getAttribute("aria-disabled"), "true");
  assert.equal(disabled.getByRole("textbox", { name: "Hex" }).hasAttribute("disabled"), true);
  assert.equal(
    disabled.root().querySelector<HTMLInputElement>("input[type='hidden']")?.disabled,
    true,
  );
  key(area, "ArrowUp");
  await nextTick();
  assert.equal(disabled.wrapper.emitted("update:modelValue"), undefined);
  disabled.unmount();

  const readOnly = mountPicker(
    { defaultValue: "#123456", readOnly: true },
    { default: () => h(ColorPickerChannelSlider, { channel: "hue" }) },
  );
  const hue = readOnly.getByRole("slider", { name: "Hue" });
  assert.equal(hue.tabIndex, 0);
  assert.equal(hue.getAttribute("aria-readonly"), "true");
  assert.equal(key(hue, "ArrowUp").defaultPrevented, true);
  const track = readOnly.root().querySelector("[data-vize-ui='color-picker-channel-slider']");
  assert.ok(track);
  stubRect(track, { width: 100, height: 10 });
  pointer(track, "pointerdown", { clientX: 50, clientY: 5 });
  await nextTick();
  assert.equal(readOnly.wrapper.emitted("update:modelValue"), undefined);
  readOnly.unmount();
});

test("exposes typed state and imperative setColor/reset controls", async () => {
  const handle = mountPicker({ defaultValue: "#102030", format: "hsl" }, {});
  const exposed = handle.exposes<ColorPickerRootExpose>();
  assert.equal(exposed.value, "hsl(210 50% 13%)");
  assert.equal(exposed.format, "hsl");
  assert.equal(exposed.state, "interactive");
  assert.equal(exposed.setColor("#ff0000"), true);
  await nextTick();
  assert.equal(exposed.value, "hsl(0 100% 50%)");
  assert.equal(exposed.setColor("bogus"), false);
  assert.equal(exposed.setColor({ hue: 120, saturation: 100, brightness: 100, alpha: 1 }), true);
  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(exposed.value, "hsl(210 50% 13%)");
  assert.ok(exposed.element instanceof HTMLDivElement);
  handle.unmount();
});

test("native form reset restores the default value", async () => {
  const Host = defineComponent({
    setup: () => () =>
      h("form", [
        h(ColorPickerRoot, { defaultValue: "#abcdef", name: "accent" }, () =>
          h(ColorPickerChannelSlider, { channel: "hue" }),
        ),
      ]),
  });
  const handle = mountInteraction(Host);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  key(handle.getByRole("slider", { name: "Hue" }), "Home");
  await nextTick();
  assert.equal(new FormData(form).get("accent"), "#efabab");
  form.reset();
  await nextTick();
  assert.equal(new FormData(form).get("accent"), "#abcdef");
  handle.unmount();
});

test("eye dropper feature-detects, picks, preserves alpha, and reports cancel", async () => {
  const original: unknown = Reflect.get(globalThis, "EyeDropper");
  const results: Array<() => Promise<unknown>> = [
    () => Promise.resolve({ sRGBHex: "#00ff00" }),
    () => Promise.reject(Object.assign(new Error("dismissed"), { name: "AbortError" })),
    () => Promise.reject(new Error("boom")),
  ];
  class FakeEyeDropper {
    open(): Promise<unknown> {
      const next = results.shift();
      return next === undefined ? Promise.resolve(null) : next();
    }
  }
  Reflect.set(globalThis, "EyeDropper", FakeEyeDropper);
  const events: string[] = [];
  const dropperRef = ref<ColorPickerEyeDropperExpose | null>(null);
  try {
    const handle = mountPicker(
      { defaultValue: "#ff000080" },
      {
        default: () =>
          h(
            ColorPickerEyeDropper,
            {
              ref: dropperRef,
              ariaLabel: "Pick color",
              onPick: (_color: unknown, hex: string) => events.push(`pick ${hex}`),
              onCancel: () => events.push("cancel"),
              onError: () => events.push("error"),
            },
            ({ state }: { state: string }) => h("span", state),
          ),
      },
    );
    await nextTick();
    const button = handle.getByRole("button", { name: "Pick color" });
    assert.ok(button instanceof HTMLButtonElement);
    assert.equal(button.disabled, false);
    assert.equal(button.getAttribute("data-state"), "idle");
    await handle.click(button);
    await new Promise((resolve) => setTimeout(resolve, 0));
    assert.equal(lastValue(handle), "#00ff0080");
    assert.equal(await dropperRef.value?.open(), null);
    assert.equal(await dropperRef.value?.open(), null);
    assert.equal(await dropperRef.value?.open(), null);
    assert.deepEqual(events, ["pick #00ff00", "cancel", "error", "error"]);
    handle.unmount();
  } finally {
    Reflect.set(globalThis, "EyeDropper", original);
    if (original === undefined) Reflect.deleteProperty(globalThis, "EyeDropper");
  }
});

test("eye dropper disables or hides itself without platform support", async () => {
  Reflect.deleteProperty(globalThis, "EyeDropper");
  const handle = mountPicker(
    {},
    {
      default: () => [
        h(ColorPickerEyeDropper, { ariaLabel: "Disabled pick" }),
        h(ColorPickerEyeDropper, { ariaLabel: "Hidden pick", unsupported: "hide" }),
      ],
    },
  );
  await nextTick();
  const buttons = handle.root().querySelectorAll<HTMLButtonElement>("button");
  assert.equal(buttons[0]?.disabled, true);
  assert.equal(buttons[0]?.getAttribute("data-state"), "unsupported");
  assert.equal(buttons[0]?.getAttribute("data-supported"), "false");
  assert.equal(buttons[1]?.hidden, true);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const part of [ColorPickerArea, ColorPickerField, ColorPickerSwatchGroup]) {
    assert.throws(() => mountInteraction(part), /VIZE_UI_CONTEXT_MISSING: ColorPicker/);
  }
  assert.throws(
    () =>
      mountInteraction(ColorPickerRoot, {
        slots: { default: () => h(ColorPickerSwatch, { value: "#fff" }) },
      }),
    /VIZE_UI_CONTEXT_MISSING: ColorPickerSwatchGroup/,
  );
});
