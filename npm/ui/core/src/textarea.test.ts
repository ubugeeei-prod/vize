import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import TextareaControl from "./textarea-control.vue";
import { mountInteraction } from "./testing/mount.ts";

function dispatchTextareaInput(textarea: HTMLTextAreaElement, value: string): void {
  textarea.value = value;
  textarea.dispatchEvent(new Event("input", { bubbles: true, cancelable: true }));
}

function dispatchChange(textarea: HTMLTextAreaElement): void {
  textarea.dispatchEvent(new Event("change", { bubbles: true, cancelable: true }));
}

function dispatchComposition(
  textarea: HTMLTextAreaElement,
  type: "compositionend" | "compositionstart",
) {
  textarea.dispatchEvent(new CompositionEvent(type, { bubbles: true, cancelable: true }));
}

test("renders a named native textarea with form and accessibility attributes", () => {
  const handle = mountInteraction(TextareaControl, {
    props: {
      id: "profile-bio",
      name: "bio",
      ariaLabel: "Bio",
      ariaDescribedby: "bio-help",
      ariaErrormessage: "bio-error",
      ariaInvalid: "grammar",
      autocomplete: "off",
      cols: 40,
      maxlength: 280,
      minlength: 2,
      placeholder: "Short bio",
      required: true,
      rows: 6,
      spellcheck: true,
      wrap: "soft",
    },
  });
  const textarea = handle.getByRole("textbox", { name: "Bio" }) as HTMLTextAreaElement;

  assert.equal(textarea.id, "profile-bio");
  assert.equal(textarea.name, "bio");
  assert.equal(textarea.required, true);
  assert.equal(textarea.getAttribute("autocomplete"), "off");
  assert.equal(textarea.getAttribute("cols"), "40");
  assert.equal(textarea.getAttribute("maxlength"), "280");
  assert.equal(textarea.getAttribute("minlength"), "2");
  assert.equal(textarea.getAttribute("placeholder"), "Short bio");
  assert.equal(textarea.getAttribute("rows"), "6");
  assert.equal(textarea.getAttribute("spellcheck"), "true");
  assert.equal(textarea.getAttribute("wrap"), "soft");
  assert.equal(textarea.getAttribute("aria-describedby"), "bio-help");
  assert.equal(textarea.getAttribute("aria-errormessage"), "bio-error");
  assert.equal(textarea.getAttribute("aria-invalid"), "grammar");
  assert.equal(textarea.getAttribute("data-vize-ui"), "textarea");
  assert.equal(textarea.getAttribute("data-state"), "editable");
  assert.equal(textarea.getAttribute("data-empty"), "true");

  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === textarea, "exposed focus() must focus the textarea");
  handle.unmount();
});

test("uncontrolled textarea emits model before input and reports native change", async () => {
  const handle = mountInteraction(TextareaControl, {
    props: { ariaLabel: "Bio" },
    record: ["update:modelValue", "input", "change"],
  });
  const textarea = handle.getByRole("textbox") as HTMLTextAreaElement;

  dispatchTextareaInput(textarea, "Line one\nLine two");
  await nextTick();
  assert.equal(textarea.value, "Line one\nLine two");
  assert.equal(textarea.getAttribute("data-empty"), "false");

  dispatchChange(textarea);
  await nextTick();

  const recorded = handle.recorded();
  assert.deepEqual(
    recorded.map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", "Line one\nLine two"],
      ["input", "Line one\nLine two"],
      ["change", "Line one\nLine two"],
    ],
  );
  assert.ok(recorded[1]?.payload[1] instanceof Event);
  assert.ok(recorded[2]?.payload[1] instanceof Event);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(TextareaControl, {
    props: { ariaLabel: "Bio", modelValue: "Initial" },
    record: ["update:modelValue", "input"],
  });
  const textarea = handle.getByRole("textbox") as HTMLTextAreaElement;

  dispatchTextareaInput(textarea, "Edited");
  await nextTick();

  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", "Edited"],
      ["input", "Edited"],
    ],
  );
  assert.equal(textarea.value, "Initial");

  await handle.wrapper.setProps({ modelValue: "Edited" });
  assert.equal(textarea.value, "Edited");
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(TextareaControl, {
          ariaLabel: "Bio",
          defaultValue: "Initial",
          name: "bio",
        }),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const textarea = handle.getByRole("textbox", { name: "Bio" }) as HTMLTextAreaElement;

  assert.equal(textarea.value, "Initial");
  dispatchTextareaInput(textarea, "Changed");
  await nextTick();
  assert.equal(textarea.value, "Changed");

  form.reset();
  await nextTick();
  assert.equal(textarea.value, "Initial");
  handle.unmount();
});

test("disabled and read-only textareas keep native availability semantics", async () => {
  const disabled = mountInteraction(TextareaControl, {
    props: { ariaLabel: "Disabled", disabled: true },
  });
  const disabledTextarea = disabled.getByRole("textbox") as HTMLTextAreaElement;
  assert.equal(disabledTextarea.disabled, true);
  assert.equal(disabledTextarea.getAttribute("data-state"), "disabled");
  assert.ok((await disabled.tab()) === null);
  disabled.unmount();

  const readOnly = mountInteraction(TextareaControl, {
    props: { ariaLabel: "Read only", readOnly: true },
  });
  const readOnlyTextarea = readOnly.getByRole("textbox") as HTMLTextAreaElement;
  assert.equal(readOnlyTextarea.readOnly, true);
  assert.equal(readOnlyTextarea.getAttribute("data-state"), "readonly");
  assert.ok((await readOnly.tab()) === readOnlyTextarea);
  readOnly.unmount();
});

test("tracks IME composition without losing the composed multiline value", async () => {
  const handle = mountInteraction(TextareaControl, {
    props: { ariaLabel: "Bio" },
    record: ["update:modelValue", "compositionStart", "compositionEnd"],
  });
  const textarea = handle.getByRole("textbox") as HTMLTextAreaElement;

  dispatchComposition(textarea, "compositionstart");
  await nextTick();
  assert.equal(textarea.getAttribute("data-composing"), "true");
  assert.equal(handle.exposes<{ composing: boolean }>().composing, true);

  textarea.value = "かな\n交じり";
  dispatchComposition(textarea, "compositionend");
  await nextTick();
  assert.equal(textarea.value, "かな\n交じり");
  assert.equal(textarea.getAttribute("data-composing"), "false");
  assert.equal(handle.exposes<{ composing: boolean }>().composing, false);
  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["compositionStart", ""],
      ["update:modelValue", "かな\n交じり"],
      ["compositionEnd", "かな\n交じり"],
    ],
  );
  const recorded = handle.recorded();
  assert.ok(recorded[0]?.payload[1] instanceof CompositionEvent);
  assert.ok(recorded[2]?.payload[1] instanceof CompositionEvent);
  handle.unmount();
});

test("controlled textarea does not rewrite native text during IME composition", async () => {
  const handle = mountInteraction(TextareaControl, {
    props: { ariaLabel: "Bio", modelValue: "" },
    record: ["update:modelValue", "input", "compositionStart", "compositionEnd"],
  });
  const textarea = handle.getByRole("textbox") as HTMLTextAreaElement;

  dispatchComposition(textarea, "compositionstart");
  dispatchTextareaInput(textarea, "か\n");
  await nextTick();

  assert.equal(textarea.value, "か\n");
  assert.equal(textarea.getAttribute("data-empty"), "false");
  assert.equal(handle.exposes<{ composing: boolean }>().composing, true);

  dispatchComposition(textarea, "compositionend");
  await nextTick();

  assert.equal(textarea.value, "");
  assert.equal(textarea.getAttribute("data-empty"), "true");
  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["compositionStart", ""],
      ["update:modelValue", "か\n"],
      ["input", "か\n"],
      ["compositionEnd", "か\n"],
    ],
  );

  await handle.wrapper.setProps({ modelValue: "か\n" });
  assert.equal(textarea.value, "か\n");
  handle.unmount();
});

test("exposes value mutation, selection range, focus, and reset controls", async () => {
  const handle = mountInteraction(TextareaControl, {
    props: { ariaLabel: "Bio", defaultValue: "Initial" },
  });
  const textarea = handle.getByRole("textbox") as HTMLTextAreaElement;
  const exposed = handle.exposes<{
    focus: (options?: FocusOptions) => void;
    reset: () => boolean;
    select: () => void;
    setSelectionRange: (start: number, end: number) => void;
    setValue: (value: string) => boolean;
  }>();

  assert.equal(exposed.setValue("Grace Hopper"), true);
  await nextTick();
  assert.equal(textarea.value, "Grace Hopper");

  textarea.blur();
  exposed.focus();
  assert.ok(handle.activeElement() === textarea);

  exposed.setSelectionRange(6, 12);
  assert.equal(textarea.selectionStart, 6);
  assert.equal(textarea.selectionEnd, 12);

  exposed.select();
  assert.equal(textarea.selectionStart, 0);
  assert.equal(textarea.selectionEnd, "Grace Hopper".length);

  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(textarea.value, "Initial");
  handle.unmount();
});
