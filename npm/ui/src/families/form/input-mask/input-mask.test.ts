import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick, ref } from "vue";

import {
  INPUT_MASK_DEFAULT_TOKENS,
  createInputMask,
  defineInputMaskTokens,
} from "./input-mask-engine.ts";
import { useInputMask } from "./input-mask-runtime.ts";
import type { InputMaskResult } from "./input-mask-engine.ts";
import type { MaskedInputExpose } from "./input-mask-types.ts";
import MaskedInput from "./masked-input.vue";
import FieldRoot from "../field/field.vue";
import FieldLabel from "../field/field-label.vue";
import type { FieldRootSlotState } from "../field/field-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

const phone = "(999) 999-9999";

async function type(
  input: HTMLInputElement,
  text: string,
  caret = text.length,
  inputType = "insertText",
): Promise<void> {
  input.value = text;
  input.setSelectionRange(caret, caret);
  input.dispatchEvent(new InputEvent("input", { bubbles: true, inputType }));
  await nextTick();
}

test("conforms typed, pasted, and pre-masked text to token slots and literals", () => {
  const mask = createInputMask(phone);
  assert.deepEqual(mask.conform(""), { masked: "", raw: "", complete: false });
  assert.deepEqual(mask.conform("5"), { masked: "(5", raw: "5", complete: false });
  assert.deepEqual(mask.conform("5551234567"), {
    masked: "(555) 123-4567",
    raw: "5551234567",
    complete: true,
  });
  assert.deepEqual(mask.conform("+1 (555) 123-4567 ext"), {
    masked: "(155) 512-3456",
    raw: "1555123456",
    complete: true,
  });
  assert.equal(mask.conform("(555) 12").masked, "(555) 12");
  assert.equal(mask.conform("abc").masked, "");
  assert.equal(mask.slotCount, 10);
  assert.equal(mask.numeric, true);
  assert.equal(mask.caretForRawCount(3), 4);
  assert.equal(mask.caretForRawCount(0), 0);
});

test("supports eager literals, placeholders, escapes, and custom typed tokens", () => {
  const eager = createInputMask(phone, { eager: true });
  assert.equal(eager.conform("555").masked, "(555) ");
  assert.equal(eager.caretForRawCount(3), 6);

  const full = createInputMask("99/99/9999", { lazy: false, placeholderChar: "-" });
  assert.equal(full.conform("12").masked, "12/--/----");
  assert.equal(full.conform("").masked, "--/--/----");
  assert.equal(full.conform("12/--/----").raw, "12");

  const escaped = createInputMask("\\9a-99");
  assert.deepEqual(escaped.conform("x12"), { masked: "9x-12", raw: "x12", complete: true });

  const hex = defineInputMaskTokens({
    H: { pattern: /[0-9a-f]/i, transform: (character: string) => character.toUpperCase() },
  });
  const color = createInputMask("#HHHHHH", { tokens: hex });
  assert.deepEqual(color.conform("ff00aa"), { masked: "#FF00AA", raw: "FF00AA", complete: true });
  assert.equal(color.numeric, false);
  assert.throws(() => defineInputMaskTokens({ HH: { pattern: /x/ } }), /VIZE_UI_INPUT_MASK_TOKEN/);
  assert.ok(INPUT_MASK_DEFAULT_TOKENS["9"].pattern.test("7"));
  assert.equal(createInputMask("aa-**").conform("ab1c2").masked, "ab-1c");
});

test("useInputMask works outside components and reports completion", () => {
  const scope = effectScope();
  const completions: InputMaskResult[] = [];
  const changes: string[] = [];
  const value = ref<string | undefined>(undefined);
  scope.run(() => {
    const controller = useInputMask({
      mask: "9999 9999",
      value,
      onChange: (next) => changes.push(next),
      onComplete: (result) => completions.push(result),
    });
    assert.equal(controller.setValue("1234"), true);
    assert.equal(controller.masked.value, "1234");
    assert.equal(controller.setValue("12345678"), true);
    assert.equal(controller.complete.value, true);
    assert.equal(controller.inputMode.value, "numeric");
    value.value = "87";
    assert.equal(controller.masked.value, "87", "controlled raw value");
    controller.setValue("9");
    assert.equal(controller.masked.value, "87", "controlled value wins");
  });
  scope.stop();
  assert.deepEqual(changes, ["1234", "12345678", "9"]);
  assert.equal(completions.length, 1);
});

test("renders a native text input with mask, form, and accessibility hooks", () => {
  const handle = mountInteraction(MaskedInput, {
    props: {
      mask: phone,
      id: "phone",
      name: "phone",
      defaultValue: "5551234567",
      ariaLabel: "Phone",
      required: true,
      autocomplete: "tel",
    },
  });
  const input = handle.getByRole("textbox", { name: "Phone" });
  assert.ok(input instanceof HTMLInputElement);

  assert.equal(input.id, "phone");
  assert.equal(input.value, "(555) 123-4567");
  assert.equal(input.getAttribute("inputmode"), "numeric");
  assert.equal(input.getAttribute("autocomplete"), "tel");
  assert.equal(input.required, true);
  assert.equal(input.getAttribute("data-vize-ui"), "masked-input");
  assert.equal(input.getAttribute("data-state"), "complete");
  assert.equal(input.getAttribute("data-complete"), "true");
  handle.unmount();
});

test("typing conforms the value, keeps the caret after the slot, and emits once complete", async () => {
  const handle = mountInteraction(MaskedInput, {
    props: { mask: phone, ariaLabel: "Phone" },
    record: ["update:modelValue", "complete"],
  });
  const input = handle.root();
  assert.ok(input instanceof HTMLInputElement);
  input.focus();

  await type(input, "5");
  assert.equal(input.value, "(5");
  assert.equal(input.selectionStart, 2);
  assert.equal(input.getAttribute("data-state"), "incomplete");
  await type(input, "(5x");
  assert.equal(input.value, "(5", "rejected characters are dropped");
  await type(input, "(555) 1234567", 13, "insertFromPaste");
  assert.equal(input.value, "(555) 123-4567");
  assert.equal(input.selectionStart, 14);
  assert.deepEqual(
    handle.recorded().map((entry) => entry.event),
    ["update:modelValue", "update:modelValue", "complete"],
  );
  assert.deepEqual(handle.recorded()[1]?.payload, ["5551234567"]);
  assert.equal(handle.exposes<MaskedInputExpose>().complete, true);
  handle.unmount();
});

test("Backspace and Delete across a literal remove the neighboring slot character", async () => {
  const handle = mountInteraction(MaskedInput, {
    props: { mask: phone, ariaLabel: "Phone", defaultValue: "5551234" },
  });
  const input = handle.root();
  assert.ok(input instanceof HTMLInputElement);
  input.focus();
  assert.equal(input.value, "(555) 123-4");

  // Backspace with the caret after "-" deletes only the literal.
  await type(input, "(555) 1234", 9, "deleteContentBackward");
  assert.equal(input.value, "(555) 124");
  assert.equal(input.selectionStart, 8);
  // Delete with the caret before ") " deletes the literal, so the next digit goes.
  await type(input, "(555 124", 4, "deleteContentForward");
  assert.equal(input.value, "(555) 24");
  assert.equal(input.selectionStart, 4);
  handle.unmount();
});

test("masked value format, placeholders, and controlled values", async () => {
  const handle = mountInteraction(MaskedInput, {
    props: {
      mask: "99/99",
      ariaLabel: "Expiry",
      valueFormat: "masked",
      lazy: false,
      modelValue: "1",
    },
    record: ["update:modelValue"],
  });
  const input = handle.root();
  assert.ok(input instanceof HTMLInputElement);

  assert.equal(input.value, "1_/__");
  await type(input, "12_/__", 2);
  assert.deepEqual(handle.recorded()[0]?.payload, ["12/__"]);
  await nextTick();
  assert.equal(input.value, "1_/__", "controlled value wins until accepted");
  await handle.wrapper.setProps({ modelValue: "12/__" });
  assert.equal(input.value, "12/__");
  handle.unmount();
});

test("form reset restores the default and disabled or read-only states publish", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(MaskedInput, { mask: "9999", name: "pin", ariaLabel: "PIN", defaultValue: "12" }),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  const input = form.querySelector("input");
  assert.ok(input instanceof HTMLInputElement);

  await type(input, "1234");
  assert.equal(new FormData(form).get("pin"), "1234");
  form.reset();
  await Promise.resolve();
  await nextTick();
  assert.equal(input.value, "12");
  assert.equal(new FormData(form).get("pin"), "12");
  handle.unmount();

  const disabled = mountInteraction(MaskedInput, { props: { mask: "99", disabled: true } });
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  disabled.unmount();
  const readOnly = mountInteraction(MaskedInput, { props: { mask: "99", readOnly: true } });
  assert.equal(readOnly.root().getAttribute("data-state"), "readonly");
  assert.equal(readOnly.root().getAttribute("data-state") === "empty", false);
  readOnly.unmount();
});

test("exposes focus, setValue, reset, and mask state", () => {
  const handle = mountInteraction(MaskedInput, {
    props: { mask: "aa-99", ariaLabel: "Code", defaultValue: "ab" },
  });
  const api = handle.exposes<MaskedInputExpose>();

  api.focus();
  assert.ok(document.activeElement === handle.root());
  assert.equal(api.setValue("xy12"), true);
  assert.equal(api.masked, "xy-12");
  assert.equal(api.raw, "xy12");
  assert.equal(api.complete, true);
  assert.equal(api.reset(), true);
  assert.equal(api.masked, "ab");
  assert.ok(api.element === handle.root());
  handle.unmount();
});

test("binds Field fieldProps for label, description, and error wiring", () => {
  const handle = mountInteraction(FieldRoot, {
    props: { id: "zip", name: "zip", invalid: true },
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "ZIP" }),
        h(MaskedInput, { ...fieldProps, mask: "99999" }),
      ],
    },
  });
  const input = handle.getByRole("textbox", { name: "ZIP" });
  assert.equal(input.id, "zip");
  assert.equal(input.getAttribute("aria-invalid"), "true");
  assert.equal(input.getAttribute("aria-errormessage"), "zip-error");
  handle.unmount();
});
