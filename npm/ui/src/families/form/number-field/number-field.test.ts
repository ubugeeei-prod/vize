import assert from "node:assert/strict";

import { test, vi } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import NumberField from "./number-field.vue";
import NumberFieldDecrement from "./number-field-decrement.vue";
import NumberFieldIncrement from "./number-field-increment.vue";
import NumberFieldInput from "./number-field-input.vue";
import type {
  NumberFieldExpose,
  NumberFieldProps,
  NumberFieldSlotState,
  NumberFieldValue,
} from "./number-field-types.ts";
import FieldRoot from "../field/field.vue";
import FieldLabel from "../field/field-label.vue";
import type { FieldRootSlotState } from "../field/field-types.ts";
import LocaleProvider from "../../i18n/locale/locale-provider.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountNumberField(
  props: NumberFieldProps & Record<string, unknown> = {},
  record: readonly string[] = ["update:modelValue", "change"],
) {
  return mountInteraction(NumberField, {
    props: { ariaLabel: "Quantity", ...props },
    record,
    slots: {
      default: (state: NumberFieldSlotState) => [
        h(NumberFieldDecrement),
        h(NumberFieldInput, { placeholder: "0" }),
        h(NumberFieldIncrement),
        h("output", null, `${state.state}:${state.formattedValue}`),
      ],
    },
  });
}

function parts(root: HTMLElement) {
  const input = root.querySelector('[data-vize-ui="number-field-input"]');
  const increment = root.querySelector('[data-vize-ui="number-field-increment"]');
  const decrement = root.querySelector('[data-vize-ui="number-field-decrement"]');
  assert.ok(input instanceof HTMLInputElement);
  assert.ok(increment instanceof HTMLButtonElement);
  assert.ok(decrement instanceof HTMLButtonElement);
  return { input, increment, decrement };
}

async function type(input: HTMLInputElement, text: string): Promise<void> {
  input.focus();
  input.value = text;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
}

async function blur(input: HTMLInputElement): Promise<void> {
  input.dispatchEvent(new FocusEvent("blur"));
  await nextTick();
  await nextTick();
}

async function key(input: HTMLInputElement, name: string): Promise<boolean> {
  const event = new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true });
  input.dispatchEvent(event);
  await nextTick();
  await nextTick();
  return event.defaultPrevented;
}

function pointer(target: Element, type: string): PointerEvent {
  const event = new PointerEvent(type, { bubbles: true, cancelable: true, button: 0 });
  target.dispatchEvent(event);
  return event;
}

test("renders an APG spinbutton with locale text, bounds, and form hooks", () => {
  const handle = mountNumberField({
    id: "quantity",
    name: "quantity",
    defaultValue: 1234.5,
    min: 0,
    max: 5000,
    required: true,
  });
  const root = handle.root();
  const { input, increment, decrement } = parts(root);
  const hidden = root.querySelector('input[type="hidden"]');

  assert.equal(root.getAttribute("data-vize-ui"), "number-field");
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-state"), "in-range");
  assert.equal(root.getAttribute("data-required"), "true");
  assert.ok(handle.getByRole("spinbutton", { name: "Quantity" }) === input);
  assert.equal(input.id, "quantity");
  assert.equal(input.type, "text");
  assert.equal(input.value, "1,234.5");
  assert.equal(input.getAttribute("inputmode"), "decimal");
  assert.equal(input.getAttribute("aria-valuenow"), "1234.5");
  assert.equal(input.getAttribute("aria-valuemin"), "0");
  assert.equal(input.getAttribute("aria-valuemax"), "5000");
  assert.equal(input.getAttribute("aria-valuetext"), "1,234.5");
  assert.equal(input.getAttribute("autocomplete"), "off");
  assert.equal(input.required, true);
  assert.equal(input.name, "");
  assert.ok(hidden instanceof HTMLInputElement);
  assert.equal(hidden.name, "quantity");
  assert.equal(hidden.value, "1234.5");
  assert.equal(increment.getAttribute("aria-label"), "Increase");
  assert.equal(increment.getAttribute("aria-controls"), "quantity");
  assert.equal(increment.tabIndex, -1);
  assert.equal(decrement.getAttribute("aria-label"), "Decrease");
  assert.equal(root.querySelector("output")?.textContent, "in-range:1,234.5");
  handle.unmount();
});

test("typing keeps partial text and commits the parsed value on blur", async () => {
  const handle = mountNumberField();
  const { input } = parts(handle.root());

  assert.equal(input.value, "");
  assert.equal(handle.root().getAttribute("data-state"), "empty");
  await type(input, "-");
  assert.equal(input.value, "-");
  await type(input, "1,234.");
  assert.deepEqual(handle.recorded(), []);
  await blur(input);

  assert.equal(input.value, "1,234");
  assert.equal(handle.exposes<NumberFieldExpose>().value, 1234);
  assert.deepEqual(handle.recorded(), [
    { event: "update:modelValue", payload: [1234] },
    { event: "change", payload: [1234, null, "blur"] },
  ]);

  await type(input, "");
  await blur(input);
  assert.equal(handle.exposes<NumberFieldExpose>().value, null);
  assert.equal(handle.root().getAttribute("data-empty"), "true");
  handle.unmount();
});

test("rejects characters that can never form a number", async () => {
  const handle = mountInteraction(NumberField, {
    props: { ariaLabel: "Count", defaultValue: 4, min: 0 },
    slots: {
      default: () => h(NumberFieldInput, { onReject: (text: string) => rejected.push(text) }),
    },
  });
  const rejected: string[] = [];
  const input = handle.root().querySelector("input");
  assert.ok(input instanceof HTMLInputElement);

  await type(input, "4a");
  assert.equal(input.value, "4");
  await type(input, "-4");
  assert.equal(input.value, "4", "min >= 0 rejects the minus sign");
  assert.deepEqual(rejected, ["4a", "-4"]);
  assert.equal(input.getAttribute("inputmode"), "decimal");
  handle.unmount();
});

test("arrow, page, home, and end keys step along the grid within bounds", async () => {
  const handle = mountNumberField({ defaultValue: 7, min: 0, max: 50, step: 5 });
  const { input } = parts(handle.root());
  const exposed = () => handle.exposes<NumberFieldExpose>();

  assert.equal(await key(input, "ArrowUp"), true);
  assert.equal(exposed().value, 10, "off-grid values move to the next grid value");
  await key(input, "ArrowDown");
  assert.equal(exposed().value, 5);
  await key(input, "PageUp");
  assert.equal(exposed().value, 50, "largeStep defaults to step * 10 and clamps");
  await key(input, "PageDown");
  assert.equal(exposed().value, 0);
  await key(input, "End");
  assert.equal(exposed().value, 50);
  assert.equal(handle.root().getAttribute("data-state"), "max");
  await key(input, "Home");
  assert.equal(exposed().value, 0);
  assert.equal(handle.root().getAttribute("data-state"), "min");
  assert.equal(await key(input, "a"), false);
  assert.deepEqual(
    handle
      .recorded()
      .filter((entry) => entry.event === "change")
      .map((entry) => entry.payload[2]),
    ["keyboard", "keyboard", "keyboard", "keyboard", "keyboard", "keyboard"],
  );
  handle.unmount();
});

test("unbounded fields keep native Home and End caret movement", async () => {
  const handle = mountNumberField({ defaultValue: 2 });
  const { input } = parts(handle.root());

  assert.equal(await key(input, "Home"), false);
  assert.equal(await key(input, "End"), false);
  assert.equal(input.getAttribute("aria-valuemin"), null);
  assert.equal(input.getAttribute("aria-valuemax"), null);
  assert.equal(input.getAttribute("inputmode"), "text", "signed fields need a minus key");
  await key(input, "ArrowDown");
  await key(input, "ArrowDown");
  await key(input, "ArrowDown");
  assert.equal(handle.exposes<NumberFieldExpose>().value, -1);
  handle.unmount();
});

test("typed text is the stepping origin and precision stays decimal-safe", async () => {
  const handle = mountNumberField({ defaultValue: 0.1, step: 0.1 });
  const { input } = parts(handle.root());

  await key(input, "ArrowUp");
  assert.equal(handle.exposes<NumberFieldExpose>().value, 0.2);
  await type(input, "12");
  await key(input, "ArrowUp");
  assert.equal(handle.exposes<NumberFieldExpose>().value, 12.1);
  assert.equal(input.value, "12.1");
  handle.unmount();
});

test("commits clamp by default and snap when requested", async () => {
  const clamped = mountNumberField({ min: 0, max: 100 });
  const clampedInput = parts(clamped.root()).input;
  await type(clampedInput, "150");
  await blur(clampedInput);
  assert.equal(clamped.exposes<NumberFieldExpose>().value, 100);
  clamped.unmount();

  const free = mountNumberField({ min: 0, max: 100, clampOnCommit: false });
  const freeInput = parts(free.root()).input;
  await type(freeInput, "150");
  await blur(freeInput);
  assert.equal(free.exposes<NumberFieldExpose>().value, 150);
  free.unmount();

  const snapped = mountNumberField({ min: 0, step: 5, snapOnCommit: true });
  const snappedInput = parts(snapped.root()).input;
  await type(snappedInput, "12");
  await blur(snappedInput);
  assert.equal(snapped.exposes<NumberFieldExpose>().value, 10);
  await type(snappedInput, "13");
  await key(snappedInput, "Enter");
  assert.equal(snapped.exposes<NumberFieldExpose>().value, 15);
  assert.equal(snapped.recorded().at(-1)?.payload[2], "enter");
  snapped.unmount();
});

test("formats and parses currency, percent, and unit styles per locale", async () => {
  const euro = mountNumberField({
    defaultValue: 1234.5,
    locale: "de-DE",
    formatOptions: { style: "currency", currency: "EUR" },
  });
  const euroInput = parts(euro.root()).input;
  assert.equal(euroInput.value, "1.234,50 €");
  await type(euroInput, "2.000,25 €");
  await blur(euroInput);
  assert.equal(euro.exposes<NumberFieldExpose>().value, 2000.25);
  assert.equal(euro.exposes<NumberFieldExpose>().locale, "de-DE");
  euro.unmount();

  const percent = mountNumberField({ defaultValue: 0.25, formatOptions: { style: "percent" } });
  const percentInput = parts(percent.root()).input;
  assert.equal(percentInput.value, "25%");
  await key(percentInput, "ArrowUp");
  assert.equal(percent.exposes<NumberFieldExpose>().value, 0.26, "percent steps by 1%");
  await type(percentInput, "50");
  await blur(percentInput);
  assert.equal(percent.exposes<NumberFieldExpose>().value, 0.5);
  assert.equal(percentInput.value, "50%");
  percent.unmount();

  const speed = mountNumberField({
    defaultValue: 88,
    formatOptions: { style: "unit", unit: "kilometer-per-hour" },
  });
  const speedInput = parts(speed.root()).input;
  assert.equal(speedInput.value, "88 km/h");
  assert.equal(speedInput.getAttribute("aria-valuetext"), "88 km/h");
  speed.unmount();

  const yen = mountNumberField({
    defaultValue: 1200,
    min: 0,
    locale: "ja-JP",
    formatOptions: { style: "currency", currency: "JPY" },
  });
  const yenInput = parts(yen.root()).input;
  assert.equal(yenInput.getAttribute("inputmode"), "numeric");
  await type(yenInput, "12.5");
  assert.equal(yenInput.value, "￥1,200", "zero-fraction currencies reject decimals");
  yen.unmount();
});

test("triggers step on click, disable at bounds, and repeat while held", async () => {
  vi.useFakeTimers();
  try {
    const handle = mountNumberField({
      defaultValue: 0,
      min: 0,
      max: 10,
      holdDelay: 300,
      holdInterval: 50,
    });
    const { input, increment, decrement } = parts(handle.root());

    assert.equal(decrement.disabled, true);
    assert.equal(decrement.getAttribute("data-disabled"), "true");
    increment.click();
    await nextTick();
    assert.equal(
      handle.exposes<NumberFieldExpose>().value,
      1,
      "assistive-technology click steps once",
    );

    const down = pointer(increment, "pointerdown");
    assert.equal(down.defaultPrevented, true);
    assert.ok(document.activeElement === input);
    await nextTick();
    assert.equal(handle.exposes<NumberFieldExpose>().value, 2);
    vi.advanceTimersByTime(299);
    assert.equal(handle.exposes<NumberFieldExpose>().value, 2);
    vi.advanceTimersByTime(1);
    await nextTick();
    assert.equal(increment.getAttribute("data-holding"), "true");
    vi.advanceTimersByTime(100);
    assert.equal(handle.exposes<NumberFieldExpose>().value, 5);
    pointer(increment, "pointerup");
    increment.click();
    await nextTick();
    vi.advanceTimersByTime(500);
    assert.equal(
      handle.exposes<NumberFieldExpose>().value,
      5,
      "release stops and click is not doubled",
    );
    assert.equal(increment.getAttribute("data-holding"), null);

    pointer(increment, "pointerdown");
    vi.advanceTimersByTime(2000);
    await nextTick();
    assert.equal(handle.exposes<NumberFieldExpose>().value, 10, "holding stops at max");
    assert.equal(increment.disabled, true);
    pointer(increment, "pointerleave");
    assert.ok(
      handle
        .recorded()
        .every((entry) => entry.event !== "change" || entry.payload[2] === "trigger"),
    );
    handle.unmount();
  } finally {
    vi.useRealTimers();
  }
});

test("wheel stepping is opt-in and requires focus", async () => {
  const wheel = (target: Element, deltaY: number) => {
    const event = new WheelEvent("wheel", { deltaY, bubbles: true, cancelable: true });
    target.dispatchEvent(event);
    return event.defaultPrevented;
  };
  const passive = mountNumberField({ defaultValue: 1 });
  const passiveInput = parts(passive.root()).input;
  passiveInput.focus();
  passiveInput.dispatchEvent(new FocusEvent("focus"));
  assert.equal(wheel(passiveInput, -1), false);
  assert.equal(passive.exposes<NumberFieldExpose>().value, 1);
  passive.unmount();

  const opted = mountNumberField({ defaultValue: 1, allowWheel: true });
  const optedInput = parts(opted.root()).input;
  await nextTick();
  assert.equal(wheel(optedInput, -1), false, "unfocused inputs let the page scroll");
  optedInput.dispatchEvent(new FocusEvent("focus"));
  assert.equal(wheel(optedInput, -1), true);
  assert.equal(opted.exposes<NumberFieldExpose>().value, 2);
  wheel(optedInput, 1);
  assert.equal(opted.exposes<NumberFieldExpose>().value, 1);
  assert.equal(opted.recorded().at(-1)?.payload[2], "wheel");
  opted.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountNumberField({ modelValue: 3 });
  const { input, increment } = parts(handle.root());

  increment.click();
  await nextTick();
  await nextTick();
  assert.deepEqual(handle.recorded()[0], { event: "update:modelValue", payload: [4] });
  assert.equal(input.value, "3");
  await handle.wrapper.setProps({ modelValue: 4 });
  assert.equal(input.value, "4");
  await handle.wrapper.setProps({ modelValue: null });
  assert.equal(input.value, "");
  handle.unmount();
});

test("submits the raw number and restores the default on form reset", async () => {
  const value = ref<NumberFieldValue>(null);
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          NumberField,
          {
            name: "price",
            ariaLabel: "Price",
            defaultValue: 1500,
            formatOptions: { style: "currency", currency: "USD" },
            "onUpdate:modelValue": (next: NumberFieldValue) => {
              value.value = next;
            },
          },
          { default: () => h(NumberFieldInput) },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  const input = form.querySelector('[role="spinbutton"]');
  assert.ok(input instanceof HTMLInputElement);

  assert.equal(input.value, "$1,500.00");
  assert.equal(new FormData(form).get("price"), "1500");
  await type(input, "$99.5");
  await blur(input);
  assert.equal(value.value, 99.5);
  assert.equal(new FormData(form).get("price"), "99.5");
  form.reset();
  await nextTick();
  await nextTick();
  assert.equal(input.value, "$1,500.00");
  assert.equal(new FormData(form).get("price"), "1500");
  handle.unmount();
});

test("disabled and read-only fields keep availability semantics", async () => {
  const disabled = mountNumberField({ defaultValue: 5, disabled: true, name: "n" });
  const disabledParts = parts(disabled.root());
  assert.equal(disabledParts.input.disabled, true);
  assert.equal(disabledParts.increment.disabled, true);
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(
    disabled.root().querySelector<HTMLInputElement>('input[type="hidden"]')?.disabled,
    true,
  );
  disabled.unmount();

  const readOnly = mountNumberField({ defaultValue: 5, readOnly: true });
  const readOnlyParts = parts(readOnly.root());
  assert.equal(readOnlyParts.input.readOnly, true);
  assert.equal(readOnlyParts.input.getAttribute("aria-readonly"), "true");
  assert.equal(readOnlyParts.increment.disabled, true);
  await key(readOnlyParts.input, "ArrowUp");
  await type(readOnlyParts.input, "9");
  await blur(readOnlyParts.input);
  assert.equal(readOnly.exposes<NumberFieldExpose>().value, 5);
  assert.equal(readOnlyParts.input.value, "5");
  assert.deepEqual(readOnly.recorded(), []);
  readOnly.unmount();
});

test("exposes focus, setValue, increment, decrement, commit, and reset", async () => {
  const handle = mountNumberField({ defaultValue: 2, min: 0, max: 10, step: 2 });
  const { input } = parts(handle.root());
  const api = handle.exposes<NumberFieldExpose>();

  api.focus();
  assert.ok(document.activeElement === input);
  assert.equal(api.increment(2), true);
  assert.equal(api.value, 6);
  assert.equal(api.decrement(), true);
  assert.equal(api.value, 4);
  assert.equal(api.setValue(99), true);
  assert.equal(api.value, 10, "setValue clamps");
  assert.equal(api.canIncrement, false);
  assert.equal(api.setValue(Number.NaN), true);
  assert.equal(api.value, null);
  await type(input, "8");
  assert.equal(api.inputText, "8");
  assert.equal(api.commit(), true);
  assert.equal(api.value, 8);
  assert.equal(api.reset(), true);
  assert.equal(api.value, 2);
  assert.ok(api.input === input);
  assert.ok(api.root === handle.root());
  handle.unmount();
});

test("binds Field fieldProps to wire the label, description, and error", () => {
  const handle = mountInteraction(FieldRoot, {
    props: { id: "age", name: "age", invalid: true },
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Age" }),
        h(NumberField, { ...fieldProps, name: "age" }, { default: () => h(NumberFieldInput) }),
      ],
    },
  });
  const input = handle.getByRole("spinbutton", { name: "Age" });

  assert.equal(input.id, "age");
  assert.equal(input.getAttribute("aria-invalid"), "true");
  assert.equal(input.getAttribute("aria-errormessage"), "age-error");
  assert.equal(input.getAttribute("aria-describedby"), "age-error");
  assert.equal(
    handle.root().querySelector('[data-vize-ui="number-field"]')?.getAttribute("data-state"),
    "invalid",
  );
  handle.unmount();
});

test("parts require a NumberField provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(() => mountInteraction(NumberFieldInput), /VIZE_UI_CONTEXT_MISSING: NumberField/);
    assert.throws(
      () => mountInteraction(NumberFieldIncrement),
      /VIZE_UI_CONTEXT_MISSING: NumberField/,
    );
  } finally {
    console.warn = warn;
  }
});

test("inherits the locale from the nearest LocaleProvider", () => {
  const handle = mountInteraction(LocaleProvider, {
    props: { locale: "de-DE" },
    slots: {
      default: () =>
        h(
          NumberField,
          { ariaLabel: "Betrag", defaultValue: 1234.5 },
          { default: () => h(NumberFieldInput) },
        ),
    },
  });
  const input = handle.getByRole("spinbutton", { name: "Betrag" });
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.value, "1.234,5");
  handle.unmount();
});
