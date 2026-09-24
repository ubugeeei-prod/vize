import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import PinInput from "./pin-input.vue";
import PinInputField from "./pin-input-field.vue";
import {
  isPinCharacters,
  removePinCharacter,
  sanitizePinInput,
  writePinCharacters,
} from "./pin-input-state.ts";
import type { PinInputExpose, PinInputSlotState } from "./pin-input-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function renderFields(state: PinInputSlotState) {
  return state.indexes.map((index) => h(PinInputField, { key: index, index }));
}

function mountPin(props: Record<string, unknown> = {}) {
  return mountInteraction(PinInput, {
    props: { length: 4, ariaLabel: "Verification code", ...props },
    record: ["update:modelValue", "complete"],
    slots: { default: renderFields },
  });
}

function fields(root: Element): HTMLInputElement[] {
  return [...root.querySelectorAll<HTMLInputElement>('[data-vize-ui="pin-input-field"]')];
}

async function settle(): Promise<void> {
  await nextTick();
  await Promise.resolve();
  await nextTick();
}

async function type(input: HTMLInputElement, text: string): Promise<void> {
  input.focus();
  input.value = text;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await settle();
}

async function key(input: HTMLInputElement, name: string): Promise<boolean> {
  const event = new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true });
  input.dispatchEvent(event);
  await settle();
  return event.defaultPrevented;
}

async function paste(input: HTMLInputElement, text: string): Promise<boolean> {
  const event = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(event, "clipboardData", { value: { getData: () => text } });
  input.dispatchEvent(event);
  await settle();
  return event.defaultPrevented;
}

test("sanitizes, writes, removes, and narrows code characters", () => {
  assert.deepEqual(sanitizePinInput("12-3 4ａ５", "numeric"), ["1", "2", "3", "4", "5"]);
  assert.deepEqual(sanitizePinInput("a1-B2", "alphanumeric"), ["a", "1", "B", "2"]);
  assert.deepEqual(sanitizePinInput("0aF9z", "alphanumeric", /[0-9A-F]/), ["0", "F", "9"]);
  assert.equal(writePinCharacters("12", 5, ["3", "4"], 4), "1234", "writes clamp to the end");
  assert.equal(writePinCharacters("1234", 1, ["9"], 4), "1934", "typing replaces in place");
  assert.equal(writePinCharacters("1", 0, ["5", "6", "7", "8", "9"], 4), "5678");
  assert.equal(removePinCharacter("1234", 1), "134");
  assert.equal(removePinCharacter("12", 5), "12");
  assert.equal(isPinCharacters(["1", "2"], 2), true);
  assert.equal(isPinCharacters(["1"], 2), false);
});

test("renders a labelled group of one-time-code fields with form hooks", () => {
  const handle = mountPin({ id: "otp", name: "code", defaultValue: "12", placeholder: "○" });
  const root = handle.root();
  const inputs = fields(root);
  const hidden = root.querySelector('input[type="hidden"]');

  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("aria-label"), "Verification code");
  assert.equal(root.getAttribute("data-state"), "incomplete");
  assert.equal(inputs.length, 4);
  assert.equal(inputs[0]?.id, "otp-0");
  assert.equal(inputs[0]?.getAttribute("autocomplete"), "one-time-code");
  assert.equal(inputs[1]?.getAttribute("autocomplete"), "off");
  assert.equal(inputs[0]?.getAttribute("inputmode"), "numeric");
  assert.equal(inputs[0]?.getAttribute("aria-label"), "Character 1 of 4");
  assert.equal(inputs[3]?.placeholder, "○");
  assert.deepEqual(
    inputs.map((input) => input.value),
    ["1", "2", "", ""],
  );
  assert.deepEqual(
    inputs.map((input) => input.getAttribute("data-filled")),
    ["true", "true", "false", "false"],
  );
  assert.ok(hidden instanceof HTMLInputElement);
  assert.equal(hidden.name, "code");
  assert.equal(hidden.value, "12");
  handle.unmount();
});

test("typing fills fields left to right, advances focus, and emits a typed completion", async () => {
  const handle = mountPin();
  const inputs = fields(handle.root());

  await type(inputs[2] ?? inputs[0]!, "7");
  assert.equal(inputs[0]?.value, "7", "typing into a later empty field fills the first gap");
  assert.ok(document.activeElement === inputs[1]);
  await type(inputs[1]!, "x");
  assert.equal(inputs[1]?.value, "", "rejected characters leave the field empty");
  await type(inputs[1]!, "8");
  await type(inputs[2]!, "9");
  await type(inputs[3]!, "0");
  assert.equal(handle.root().getAttribute("data-state"), "complete");
  assert.deepEqual(handle.recorded().at(-1), {
    event: "complete",
    payload: ["7890", ["7", "8", "9", "0"]],
  });
  assert.ok(document.activeElement === inputs[3], "focus stays on the last field");
  await type(inputs[1]!, "85");
  assert.equal(handle.exposes<PinInputExpose>().value, "7590", "typing over a field replaces it");
  assert.equal(
    handle.recorded().filter((entry) => entry.event === "complete").length,
    1,
    "complete fires once per completion",
  );
  handle.unmount();
});

test("paste and autofill distribute characters across fields", async () => {
  const handle = mountPin({ length: 6 });
  const inputs = fields(handle.root());

  assert.equal(await paste(inputs[0]!, "12 34-56"), true);
  assert.deepEqual(
    inputs.map((input) => input.value),
    ["1", "2", "3", "4", "5", "6"],
  );
  assert.ok(document.activeElement === inputs[5]);
  handle.exposes<PinInputExpose>().clear();
  await settle();
  // iOS SMS autofill writes the whole code into the first field.
  await type(inputs[0]!, "654321");
  assert.equal(handle.exposes<PinInputExpose>().value, "654321");
  handle.unmount();
});

test("Backspace, Delete, arrows, Home, and End edit and move between fields", async () => {
  const handle = mountPin({ defaultValue: "1234" });
  const inputs = fields(handle.root());
  inputs[2]?.focus();

  assert.equal(await key(inputs[2]!, "Backspace"), true);
  assert.equal(handle.exposes<PinInputExpose>().value, "124", "clears the focused character");
  assert.ok(document.activeElement === inputs[2]);
  await key(inputs[3]!, "Backspace");
  assert.equal(handle.exposes<PinInputExpose>().value, "12", "empty fields delete backwards");
  assert.ok(document.activeElement === inputs[2]);
  await key(inputs[0]!, "Delete");
  assert.equal(handle.exposes<PinInputExpose>().value, "2");
  await key(inputs[0]!, "ArrowRight");
  assert.ok(document.activeElement === inputs[1]);
  await key(inputs[1]!, "ArrowLeft");
  assert.ok(document.activeElement === inputs[0]);
  await key(inputs[0]!, "End");
  assert.ok(document.activeElement === inputs[1], "End goes to the first empty field");
  await key(inputs[1]!, "Home");
  assert.ok(document.activeElement === inputs[0]);
  assert.equal(await key(inputs[0]!, "Tab"), false, "Tab keeps native focus order");
  handle.unmount();
});

test("masking, alphanumeric codes, custom patterns, and controlled values", async () => {
  const masked = mountPin({ mask: true, type: "alphanumeric", otp: false });
  const maskedInputs = fields(masked.root());
  assert.equal(maskedInputs[0]?.type, "password");
  assert.equal(maskedInputs[0]?.getAttribute("inputmode"), "text");
  assert.equal(maskedInputs[0]?.getAttribute("autocomplete"), "off");
  await type(maskedInputs[0]!, "a");
  assert.equal(masked.exposes<PinInputExpose>().value, "a");
  masked.unmount();

  const hex = mountPin({ pattern: /[0-9A-F]/ });
  await type(fields(hex.root())[0]!, "g");
  await type(fields(hex.root())[0]!, "F");
  assert.equal(hex.exposes<PinInputExpose>().value, "F");
  hex.unmount();

  const controlled = mountPin({ modelValue: "1" });
  const controlledInputs = fields(controlled.root());
  await type(controlledInputs[1]!, "2");
  assert.deepEqual(controlled.recorded()[0], { event: "update:modelValue", payload: ["12"] });
  assert.equal(controlledInputs[1]?.value, "", "controlled value wins until accepted");
  await controlled.wrapper.setProps({ modelValue: "12" });
  assert.equal(controlledInputs[1]?.value, "2");
  controlled.unmount();
});

test("submits the joined code, requires completion, and restores defaults on reset", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          PinInput,
          { length: 3, name: "pin", defaultValue: "12", required: true, ariaLabel: "PIN" },
          { default: renderFields },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  const inputs = fields(form);

  assert.equal(inputs[0]?.required, true);
  await type(inputs[2]!, "3");
  assert.equal(new FormData(form).get("pin"), "123");
  assert.equal(inputs[0]?.required, false, "complete codes are no longer required per field");
  form.reset();
  await settle();
  assert.equal(new FormData(form).get("pin"), "12");
  assert.deepEqual(
    inputs.map((input) => input.value),
    ["1", "2", ""],
  );
  handle.unmount();
});

test("disabled codes disable every field and ignore edits", async () => {
  const handle = mountPin({ disabled: true, defaultValue: "1" });
  const inputs = fields(handle.root());
  assert.ok(inputs.every((input) => input.disabled));
  assert.equal(handle.root().getAttribute("data-state"), "disabled");
  await key(inputs[0]!, "Backspace");
  await type(inputs[1]!, "2");
  assert.equal(handle.exposes<PinInputExpose>().value, "1");
  assert.deepEqual(handle.recorded(), []);
  handle.unmount();
});

test("exposes focus, setValue, clear, and state; fields need a PinInput", async () => {
  const handle = mountPin({ getFieldLabel: (index: number) => `Digit ${index + 1}` });
  const api = handle.exposes<PinInputExpose>();
  const inputs = fields(handle.root());

  assert.equal(inputs[0]?.getAttribute("aria-label"), "Digit 1");
  assert.equal(api.setValue("9a8b7"), true);
  assert.equal(api.value, "987");
  api.focus();
  assert.ok(document.activeElement === inputs[3], "focus() targets the first empty field");
  api.focus(0);
  assert.ok(document.activeElement === inputs[0]);
  assert.equal(api.clear(), true);
  assert.equal(api.state, "empty");
  assert.deepEqual(api.indexes, [0, 1, 2, 3]);
  assert.ok(api.root === handle.root());
  handle.unmount();

  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(PinInputField, { props: { index: 0 } }),
      /VIZE_UI_CONTEXT_MISSING: PinInput/,
    );
  } finally {
    console.warn = warn;
  }
});
