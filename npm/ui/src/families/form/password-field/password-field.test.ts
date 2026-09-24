import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import PasswordField from "./password-field.vue";
import PasswordFieldInput from "./password-field-input.vue";
import PasswordFieldToggle from "./password-field-toggle.vue";
import { estimatePasswordStrength } from "./password-field-strength.ts";
import type { PasswordStrengthEstimate } from "./password-field-strength.ts";
import type { PasswordFieldExpose, PasswordFieldSlotState } from "./password-field-types.ts";
import FieldRoot from "../field/field.vue";
import FieldLabel from "../field/field-label.vue";
import type { FieldRootSlotState } from "../field/field-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function mountPassword(props: Record<string, unknown> = {}) {
  return mountInteraction(PasswordField, {
    props: { ariaLabel: "Password", ...props },
    record: ["update:modelValue", "update:visible", "capsLockChange"],
    slots: {
      default: (state: PasswordFieldSlotState<PasswordStrengthEstimate>) => [
        h(PasswordFieldInput, { placeholder: "Password" }),
        h(PasswordFieldToggle),
        h("output", `${state.state}:${state.capsLock}:${state.strength?.label ?? "none"}`),
      ],
    },
  });
}

function parts(root: HTMLElement) {
  const input = root.querySelector('[data-vize-ui="password-field-input"]');
  const toggle = root.querySelector('[data-vize-ui="password-field-toggle"]');
  assert.ok(input instanceof HTMLInputElement);
  assert.ok(toggle instanceof HTMLButtonElement);
  return { input, toggle };
}

async function type(input: HTMLInputElement, text: string): Promise<void> {
  input.value = text;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
}

function keyWithCapsLock(
  input: HTMLInputElement,
  type: "keydown" | "keyup",
  active: boolean,
): void {
  const event = new KeyboardEvent(type, { key: "A", bubbles: true });
  Object.defineProperty(event, "getModifierState", {
    value: (key: string) => key === "CapsLock" && active,
  });
  input.dispatchEvent(event);
}

test("scores length and character variety with a dependency-free heuristic", () => {
  assert.deepEqual(estimatePasswordStrength(""), {
    score: 0,
    label: "very-weak",
    checks: { length: false, lowercase: false, uppercase: false, digit: false, symbol: false },
  });
  assert.equal(estimatePasswordStrength("abc").label, "very-weak");
  assert.equal(estimatePasswordStrength("aB3$").label, "weak", "short passwords are capped");
  assert.equal(estimatePasswordStrength("abcdefgh").label, "weak");
  assert.equal(estimatePasswordStrength("abcdefg1").label, "fair");
  assert.equal(estimatePasswordStrength("Abcdefg1").label, "strong");
  assert.equal(estimatePasswordStrength("Abcdefg1!xyzXYZ9?").score, 4);
  assert.equal(
    estimatePasswordStrength("パスワードパスワード", { minLength: 4 }).checks.length,
    true,
  );
});

test("renders a native password input with sign-in autocomplete and a toggle", () => {
  const handle = mountPassword({
    id: "pw",
    name: "password",
    required: true,
    defaultValue: "secret",
  });
  const root = handle.root();
  const { input, toggle } = parts(root);

  assert.equal(root.getAttribute("data-vize-ui"), "password-field");
  assert.equal(root.getAttribute("data-state"), "hidden");
  assert.equal(input.id, "pw");
  assert.equal(input.type, "password");
  assert.equal(input.name, "password");
  assert.equal(input.value, "secret");
  assert.equal(input.getAttribute("autocomplete"), "current-password");
  assert.equal(input.getAttribute("autocapitalize"), "off");
  assert.equal(input.getAttribute("spellcheck"), "false");
  assert.equal(input.required, true);
  assert.equal(toggle.type, "button");
  assert.equal(toggle.getAttribute("aria-pressed"), "false");
  assert.equal(toggle.getAttribute("aria-label"), "Show password");
  assert.equal(toggle.getAttribute("aria-controls"), "pw");
  handle.unmount();
});

test("the toggle reveals and hides the password while keeping focus in the input", async () => {
  const handle = mountPassword();
  const { input, toggle } = parts(handle.root());
  input.focus();

  const down = new PointerEvent("pointerdown", { bubbles: true, cancelable: true, button: 0 });
  toggle.dispatchEvent(down);
  assert.equal(down.defaultPrevented, true);
  toggle.click();
  await nextTick();
  assert.equal(input.type, "text");
  assert.equal(toggle.getAttribute("aria-pressed"), "true");
  assert.equal(toggle.getAttribute("aria-label"), "Hide password");
  assert.equal(handle.root().getAttribute("data-state"), "visible");
  assert.ok(document.activeElement === input);
  toggle.click();
  await nextTick();
  assert.equal(input.type, "password");
  assert.deepEqual(
    handle.recorded().map((entry) => [entry.event, entry.payload[0]]),
    [
      ["update:visible", true],
      ["update:visible", false],
    ],
  );
  handle.unmount();
});

test("controlled visibility and value win until the parent accepts them", async () => {
  const handle = mountPassword({ visible: false, modelValue: "a" });
  const { input, toggle } = parts(handle.root());

  toggle.click();
  await nextTick();
  assert.equal(input.type, "password");
  await handle.wrapper.setProps({ visible: true });
  assert.equal(input.type, "text");
  await type(input, "ab");
  await Promise.resolve();
  assert.equal(input.value, "a");
  await handle.wrapper.setProps({ modelValue: "ab" });
  assert.equal(input.value, "ab");
  handle.unmount();
});

test("detects Caps Lock from key events and clears it on blur", async () => {
  const handle = mountPassword();
  const { input } = parts(handle.root());

  keyWithCapsLock(input, "keydown", true);
  await nextTick();
  assert.equal(handle.root().getAttribute("data-caps-lock"), "true");
  assert.match(handle.root().querySelector("output")?.textContent ?? "", /:true:/);
  keyWithCapsLock(input, "keyup", true);
  keyWithCapsLock(input, "keydown", false);
  await nextTick();
  assert.equal(handle.root().getAttribute("data-caps-lock"), null);
  keyWithCapsLock(input, "keydown", true);
  input.dispatchEvent(new FocusEvent("blur"));
  await nextTick();
  assert.deepEqual(
    handle
      .recorded()
      .filter((entry) => entry.event === "capsLockChange")
      .map((entry) => entry.payload[0]),
    [true, false, true, false],
  );
  handle.unmount();
});

test("the strength hook evaluates every change and types the slot", async () => {
  const handle = mountPassword({ evaluateStrength: estimatePasswordStrength });
  const { input } = parts(handle.root());
  const output = () => handle.root().querySelector("output")?.textContent;

  assert.equal(output(), "hidden:false:very-weak");
  await type(input, "Abcdefg1");
  assert.equal(output(), "hidden:false:strong");
  assert.equal(handle.exposes<PasswordFieldExpose<PasswordStrengthEstimate>>().strength?.score, 3);
  handle.unmount();

  const none = mountPassword();
  assert.equal(none.root().querySelector("output")?.textContent, "hidden:false:none");
  none.unmount();
});

test("form reset restores the password and hides it again", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          PasswordField,
          { name: "password", ariaLabel: "Password", defaultValue: "init" },
          { default: () => [h(PasswordFieldInput), h(PasswordFieldToggle)] },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  const { input, toggle } = parts(form);

  await type(input, "changed");
  toggle.click();
  await nextTick();
  assert.equal(new FormData(form).get("password"), "changed");
  form.reset();
  await nextTick();
  await nextTick();
  assert.equal(input.value, "init");
  assert.equal(input.type, "password");
  handle.unmount();
});

test("disabled and read-only fields lock editing and the toggle appropriately", async () => {
  const disabled = mountPassword({ disabled: true });
  const disabledParts = parts(disabled.root());
  assert.equal(disabledParts.input.disabled, true);
  assert.equal(disabledParts.toggle.disabled, true);
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(disabled.exposes<PasswordFieldExpose<undefined>>().setVisible(true), false);
  disabled.unmount();

  const readOnly = mountPassword({ readOnly: true, defaultValue: "keep" });
  const readOnlyParts = parts(readOnly.root());
  assert.equal(readOnlyParts.input.readOnly, true);
  await type(readOnlyParts.input, "other");
  await Promise.resolve();
  assert.equal(readOnlyParts.input.value, "keep");
  readOnlyParts.toggle.click();
  await nextTick();
  assert.equal(readOnlyParts.input.type, "text", "read-only passwords can still be revealed");
  readOnly.unmount();
});

test("exposes focus, setValue, setVisible, toggleVisible, and reset", () => {
  const handle = mountPassword({ defaultValue: "x" });
  const { input } = parts(handle.root());
  const api = handle.exposes<PasswordFieldExpose<undefined>>();

  api.focus();
  assert.ok(document.activeElement === input);
  assert.equal(api.setValue("y"), true);
  assert.equal(api.value, "y");
  assert.equal(api.toggleVisible(), true);
  assert.equal(api.visible, true);
  assert.equal(api.setVisible(true), false);
  api.reset();
  assert.equal(api.value, "x");
  assert.equal(api.visible, false);
  assert.ok(api.root === handle.root());
  handle.unmount();
});

test("binds Field fieldProps and requires a PasswordField provider for parts", () => {
  const handle = mountInteraction(FieldRoot, {
    props: { id: "pw", name: "pw", invalid: true },
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Password" }),
        h(PasswordField, { ...fieldProps }, { default: () => h(PasswordFieldInput) }),
      ],
    },
  });
  const input = handle.root().querySelector("input");
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.id, "pw");
  assert.equal(input.getAttribute("aria-labelledby"), "pw-label");
  assert.equal(input.getAttribute("aria-invalid"), "true");
  assert.equal(input.getAttribute("aria-errormessage"), "pw-error");
  handle.unmount();

  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(PasswordFieldInput),
      /VIZE_UI_CONTEXT_MISSING: PasswordField/,
    );
    assert.throws(
      () => mountInteraction(PasswordFieldToggle),
      /VIZE_UI_CONTEXT_MISSING: PasswordField/,
    );
  } finally {
    console.warn = warn;
  }
});
