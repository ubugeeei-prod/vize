import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import TextInput from "./text-input.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function dispatchTextInput(input: HTMLInputElement, value: string): void {
  input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true, cancelable: true }));
}

function dispatchChange(input: HTMLInputElement): void {
  input.dispatchEvent(new Event("change", { bubbles: true, cancelable: true }));
}

function dispatchComposition(input: HTMLInputElement, type: "compositionend" | "compositionstart") {
  input.dispatchEvent(new CompositionEvent(type, { bubbles: true, cancelable: true }));
}

test("renders a named native input with form and accessibility attributes", () => {
  const handle = mountInteraction(TextInput, {
    props: {
      id: "account-email",
      name: "email",
      type: "email",
      ariaLabel: "Email",
      ariaDescribedby: "email-help",
      ariaErrormessage: "email-error",
      ariaInvalid: true,
      autocomplete: "email",
      enterKeyHint: "next",
      inputMode: "email",
      placeholder: "name@example.com",
      required: true,
    },
  });
  const input = handle.getByRole("textbox", { name: "Email" }) as HTMLInputElement;

  assert.equal(input.id, "account-email");
  assert.equal(input.name, "email");
  assert.equal(input.type, "email");
  assert.equal(input.required, true);
  assert.equal(input.getAttribute("autocomplete"), "email");
  assert.equal(input.getAttribute("enterkeyhint"), "next");
  assert.equal(input.getAttribute("inputmode"), "email");
  assert.equal(input.getAttribute("placeholder"), "name@example.com");
  assert.equal(input.getAttribute("aria-describedby"), "email-help");
  assert.equal(input.getAttribute("aria-errormessage"), "email-error");
  assert.equal(input.getAttribute("aria-invalid"), "true");
  assert.equal(input.getAttribute("data-vize-ui"), "input");
  assert.equal(input.getAttribute("data-state"), "editable");
  assert.equal(input.getAttribute("data-empty"), "true");

  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === input, "exposed focus() must focus the input");
  handle.unmount();
});

test("uncontrolled input emits model before input and reports native change", async () => {
  const handle = mountInteraction(TextInput, {
    props: { ariaLabel: "Name" },
    record: ["update:modelValue", "input", "change"],
  });
  const input = handle.getByRole("textbox") as HTMLInputElement;

  dispatchTextInput(input, "Ada");
  await nextTick();
  assert.equal(input.value, "Ada");
  assert.equal(input.getAttribute("data-empty"), "false");

  dispatchChange(input);
  await nextTick();

  const recorded = handle.recorded();
  assert.deepEqual(
    recorded.map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", "Ada"],
      ["input", "Ada"],
      ["change", "Ada"],
    ],
  );
  assert.ok(recorded[1]?.payload[1] instanceof Event);
  assert.ok(recorded[2]?.payload[1] instanceof Event);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(TextInput, {
    props: { ariaLabel: "Name", modelValue: "Ada" },
    record: ["update:modelValue", "input"],
  });
  const input = handle.getByRole("textbox") as HTMLInputElement;

  dispatchTextInput(input, "Grace");
  await nextTick();

  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", "Grace"],
      ["input", "Grace"],
    ],
  );
  assert.equal(input.value, "Ada");

  await handle.wrapper.setProps({ modelValue: "Grace" });
  assert.equal(input.value, "Grace");
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(TextInput, {
          ariaLabel: "Search",
          defaultValue: "initial",
          name: "query",
        }),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const input = handle.getByRole("textbox", { name: "Search" }) as HTMLInputElement;

  assert.equal(input.value, "initial");
  dispatchTextInput(input, "changed");
  await nextTick();
  assert.equal(input.value, "changed");

  form.reset();
  await nextTick();
  assert.equal(input.value, "initial");
  handle.unmount();
});

test("disabled and read-only inputs keep native availability semantics", async () => {
  const disabled = mountInteraction(TextInput, {
    props: { ariaLabel: "Disabled", disabled: true },
  });
  const disabledInput = disabled.getByRole("textbox") as HTMLInputElement;
  assert.equal(disabledInput.disabled, true);
  assert.equal(disabledInput.getAttribute("data-state"), "disabled");
  assert.ok((await disabled.tab()) === null);
  disabled.unmount();

  const readOnly = mountInteraction(TextInput, {
    props: { ariaLabel: "Read only", readOnly: true },
  });
  const readOnlyInput = readOnly.getByRole("textbox") as HTMLInputElement;
  assert.equal(readOnlyInput.readOnly, true);
  assert.equal(readOnlyInput.getAttribute("data-state"), "readonly");
  assert.ok((await readOnly.tab()) === readOnlyInput);
  readOnly.unmount();
});

test("tracks IME composition without losing the composed value", async () => {
  const handle = mountInteraction(TextInput, {
    props: { ariaLabel: "Name" },
    record: ["update:modelValue", "compositionStart", "compositionEnd"],
  });
  const input = handle.getByRole("textbox") as HTMLInputElement;

  dispatchComposition(input, "compositionstart");
  await nextTick();
  assert.equal(input.getAttribute("data-composing"), "true");
  assert.equal(handle.exposes<{ composing: boolean }>().composing, true);

  input.value = "かな";
  dispatchComposition(input, "compositionend");
  await nextTick();
  assert.equal(input.value, "かな");
  assert.equal(input.getAttribute("data-composing"), "false");
  assert.equal(handle.exposes<{ composing: boolean }>().composing, false);
  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["compositionStart", ""],
      ["update:modelValue", "かな"],
      ["compositionEnd", "かな"],
    ],
  );
  const recorded = handle.recorded();
  assert.ok(recorded[0]?.payload[1] instanceof CompositionEvent);
  assert.ok(recorded[2]?.payload[1] instanceof CompositionEvent);
  handle.unmount();
});

test("controlled input does not rewrite native text during IME composition", async () => {
  const handle = mountInteraction(TextInput, {
    props: { ariaLabel: "Name", modelValue: "" },
    record: ["update:modelValue", "input", "compositionStart", "compositionEnd"],
  });
  const input = handle.getByRole("textbox") as HTMLInputElement;

  dispatchComposition(input, "compositionstart");
  dispatchTextInput(input, "か");
  await nextTick();

  assert.equal(input.value, "か");
  assert.equal(handle.exposes<{ composing: boolean }>().composing, true);

  dispatchComposition(input, "compositionend");
  await nextTick();

  assert.equal(input.value, "");
  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["compositionStart", ""],
      ["update:modelValue", "か"],
      ["input", "か"],
      ["compositionEnd", "か"],
    ],
  );

  await handle.wrapper.setProps({ modelValue: "か" });
  assert.equal(input.value, "か");
  handle.unmount();
});

test("exposes value mutation, selection, focus, and reset controls", async () => {
  const handle = mountInteraction(TextInput, {
    props: { ariaLabel: "Name", defaultValue: "Ada" },
  });
  const input = handle.getByRole("textbox") as HTMLInputElement;
  const exposed = handle.exposes<{
    focus: (options?: FocusOptions) => void;
    reset: () => boolean;
    select: () => void;
    setValue: (value: string) => boolean;
  }>();

  assert.equal(exposed.setValue("Grace"), true);
  await nextTick();
  assert.equal(input.value, "Grace");

  input.blur();
  exposed.focus();
  assert.ok(handle.activeElement() === input);

  exposed.select();
  assert.equal(input.selectionStart, 0);
  assert.equal(input.selectionEnd, "Grace".length);

  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(input.value, "Ada");
  handle.unmount();
});
