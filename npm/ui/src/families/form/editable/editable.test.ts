import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import Editable from "./editable.vue";
import EditableInput from "./editable-input.vue";
import EditablePreview from "./editable-preview.vue";
import EditableTrigger from "./editable-trigger.vue";
import type { EditableExpose, EditableSlotState } from "./editable-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function mountEditable(props: Record<string, unknown> = {}) {
  return mountInteraction(Editable, {
    props: { ariaLabel: "Title", defaultValue: "Draft", ...props },
    record: ["update:modelValue", "update:editing", "edit", "submit", "cancel"],
    slots: {
      default: (state: EditableSlotState) => [
        h(EditablePreview),
        h(EditableInput),
        h(EditableTrigger, { action: "edit" }, { default: () => "Edit" }),
        h(EditableTrigger, { action: "submit" }, { default: () => "Save" }),
        h(EditableTrigger, { action: "cancel" }, { default: () => "Cancel" }),
        h("output", `${state.state}:${state.value}:${state.draft}`),
      ],
    },
  });
}

function parts(root: HTMLElement) {
  const preview = root.querySelector('[data-vize-ui="editable-preview"]');
  const input = root.querySelector('[data-vize-ui="editable-input"]');
  const trigger = (action: string) => {
    const button = root.querySelector(`[data-action="${action}"]`);
    assert.ok(button instanceof HTMLButtonElement);
    return button;
  };
  assert.ok(preview instanceof HTMLElement);
  assert.ok(input instanceof HTMLInputElement);
  return {
    preview,
    input,
    edit: trigger("edit"),
    save: trigger("submit"),
    cancel: trigger("cancel"),
  };
}

async function settle(): Promise<void> {
  await nextTick();
  await nextTick();
}

async function key(target: HTMLElement, name: string): Promise<boolean> {
  const event = new KeyboardEvent("keydown", { key: name, bubbles: true, cancelable: true });
  target.dispatchEvent(event);
  await settle();
  return event.defaultPrevented;
}

async function type(input: HTMLInputElement, text: string): Promise<void> {
  input.value = text;
  input.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
}

function events(handle: ReturnType<typeof mountEditable>): string[] {
  return handle.recorded().map((entry) => entry.event);
}

test("renders a focusable preview with the input and editing triggers hidden", () => {
  const handle = mountEditable({ id: "title", name: "title" });
  const root = handle.root();
  const { preview, input, edit, save, cancel } = parts(root);

  assert.equal(root.getAttribute("data-vize-ui"), "editable");
  assert.equal(root.getAttribute("data-state"), "preview");
  assert.equal(preview.textContent, "Draft");
  assert.equal(preview.getAttribute("role"), "button");
  assert.equal(preview.tabIndex, 0);
  assert.equal(preview.hidden, false);
  assert.equal(input.hidden, true);
  assert.equal(input.id, "title");
  assert.equal(input.getAttribute("aria-label"), "Title");
  assert.equal(edit.hidden, false);
  assert.equal(save.hidden, true);
  assert.equal(cancel.hidden, true);
  assert.equal(root.querySelector<HTMLInputElement>('input[type="hidden"]')?.value, "Draft");
  handle.unmount();
});

test("focusing the preview enters edit mode and focuses the selected input", async () => {
  const handle = mountEditable();
  const { preview, input, save } = parts(handle.root());

  preview.focus();
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "editing");
  assert.equal(preview.hidden, true);
  assert.equal(input.hidden, false);
  assert.equal(save.hidden, false);
  assert.ok(document.activeElement === input);
  assert.equal(input.selectionStart, 0);
  assert.equal(input.selectionEnd, 5);
  assert.deepEqual(events(handle), ["update:editing", "edit"]);
  handle.unmount();
});

test("Enter submits, Escape cancels, and focus returns to the preview without reopening", async () => {
  const handle = mountEditable();
  const { preview, input } = parts(handle.root());

  preview.focus();
  await settle();
  await type(input, "Final");
  assert.equal(handle.root().querySelector("output")?.textContent, "editing:Draft:Final");
  assert.equal(await key(input, "Enter"), true);
  assert.equal(handle.exposes<EditableExpose>().value, "Final");
  assert.equal(handle.root().getAttribute("data-state"), "preview");
  assert.ok(document.activeElement === preview, "focus returns to the preview");
  assert.equal(
    handle.root().getAttribute("data-state"),
    "preview",
    "returning focus does not re-open",
  );
  assert.deepEqual(
    handle.recorded().find((entry) => entry.event === "submit"),
    {
      event: "submit",
      payload: ["Final", "Draft"],
    },
  );

  await key(preview, "Enter");
  assert.equal(handle.root().getAttribute("data-state"), "editing", "Enter on the preview edits");
  await type(input, "Discarded");
  assert.equal(await key(input, "Escape"), true);
  assert.equal(handle.exposes<EditableExpose>().value, "Final");
  assert.equal(preview.textContent, "Final");
  assert.deepEqual(
    handle.recorded().find((entry) => entry.event === "cancel"),
    { event: "cancel", payload: ["Final", "Discarded"] },
  );
  handle.unmount();
});

test("blur submits by default and submitMode controls Enter and blur", async () => {
  const both = mountEditable();
  const bothParts = parts(both.root());
  bothParts.preview.focus();
  await settle();
  await type(bothParts.input, "Blurred");
  bothParts.input.dispatchEvent(new FocusEvent("blur"));
  await settle();
  assert.equal(both.exposes<EditableExpose>().value, "Blurred");
  both.unmount();

  const enterOnly = mountEditable({ submitMode: "enter" });
  const enterParts = parts(enterOnly.root());
  enterParts.preview.focus();
  await settle();
  await type(enterParts.input, "Kept draft");
  enterParts.input.dispatchEvent(new FocusEvent("blur"));
  await settle();
  assert.equal(enterOnly.exposes<EditableExpose>().editing, true, "blur does not submit");
  enterOnly.unmount();

  const none = mountEditable({ submitMode: "none" });
  const noneParts = parts(none.root());
  noneParts.preview.focus();
  await settle();
  assert.equal(await key(noneParts.input, "Enter"), false);
  assert.equal(none.exposes<EditableExpose>().editing, true);
  none.unmount();
});

test("triggers edit, submit, and cancel while keeping focus in the input", async () => {
  const handle = mountEditable({ activationMode: "none" });
  const { preview, input, edit, save, cancel } = parts(handle.root());

  assert.equal(preview.tabIndex, 0, "the preview stays reachable for Enter/F2");
  preview.focus();
  await settle();
  assert.equal(
    handle.exposes<EditableExpose>().editing,
    false,
    "activationMode none ignores focus",
  );
  preview.click();
  await settle();
  assert.equal(handle.exposes<EditableExpose>().editing, false, "and ignores clicks");
  edit.click();
  await settle();
  assert.ok(document.activeElement === input);
  await type(input, "Via button");
  const down = new PointerEvent("pointerdown", { bubbles: true, cancelable: true, button: 0 });
  save.dispatchEvent(down);
  assert.equal(down.defaultPrevented, true);
  save.click();
  await settle();
  assert.equal(handle.exposes<EditableExpose>().value, "Via button");
  edit.click();
  await settle();
  await type(input, "nope");
  cancel.click();
  await settle();
  assert.equal(handle.exposes<EditableExpose>().value, "Via button");
  handle.unmount();
});

test("click and double-click activation modes", async () => {
  const click = mountEditable({ activationMode: "click" });
  const clickParts = parts(click.root());
  clickParts.preview.focus();
  await settle();
  assert.equal(click.exposes<EditableExpose>().editing, false);
  clickParts.preview.click();
  await settle();
  assert.equal(click.exposes<EditableExpose>().editing, true);
  click.unmount();

  const dbl = mountEditable({ activationMode: "dblclick" });
  const dblParts = parts(dbl.root());
  dblParts.preview.click();
  await settle();
  assert.equal(dbl.exposes<EditableExpose>().editing, false);
  dblParts.preview.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
  await settle();
  assert.equal(dbl.exposes<EditableExpose>().editing, true);
  await key(dblParts.input, "Escape");
  await key(dblParts.preview, "F2");
  assert.equal(dbl.exposes<EditableExpose>().editing, true, "F2 edits in any activation mode");
  dbl.unmount();
});

test("controlled value and editing win until the parent accepts them", async () => {
  const handle = mountEditable({ modelValue: "A", editing: false });
  const { preview, input } = parts(handle.root());

  preview.focus();
  await settle();
  assert.equal(input.hidden, true, "controlled editing stays closed");
  await handle.wrapper.setProps({ editing: true });
  await settle();
  assert.equal(input.hidden, false);
  await type(input, "B");
  await key(input, "Enter");
  assert.equal(preview.textContent, "A", "controlled value wins until accepted");
  await handle.wrapper.setProps({ modelValue: "B" });
  assert.equal(preview.textContent, "B");
  handle.unmount();
});

test("placeholder, disabled, and read-only states", async () => {
  const empty = mountEditable({ defaultValue: "", placeholder: "Untitled" });
  const emptyParts = parts(empty.root());
  assert.equal(emptyParts.preview.textContent, "Untitled");
  assert.equal(emptyParts.preview.getAttribute("data-empty"), "true");
  assert.equal(emptyParts.input.placeholder, "Untitled");
  empty.unmount();

  for (const flag of ["disabled", "readOnly"] as const) {
    const locked = mountEditable({ [flag]: true });
    const lockedParts = parts(locked.root());
    assert.equal(lockedParts.preview.getAttribute("aria-disabled"), "true");
    assert.equal(lockedParts.edit.disabled, true);
    lockedParts.preview.focus();
    await key(lockedParts.preview, "Enter");
    assert.equal(locked.exposes<EditableExpose>().edit(), false);
    assert.equal(
      locked.root().getAttribute("data-state"),
      flag === "disabled" ? "disabled" : "readonly",
    );
    assert.deepEqual(locked.recorded(), []);
    locked.unmount();
  }
});

test("submits the committed value with a form and exposes edit, submit, cancel, setValue", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          Editable,
          { name: "title", defaultValue: "One", ariaLabel: "Title" },
          { default: () => [h(EditablePreview), h(EditableInput)] },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  assert.equal(new FormData(form).get("title"), "One");
  handle.unmount();

  const api = mountEditable();
  const exposed = api.exposes<EditableExpose>();
  assert.equal(exposed.edit(), true);
  assert.equal(exposed.edit(), false, "already editing");
  await settle();
  await type(parts(api.root()).input, "Two");
  assert.equal(exposed.draft, "Two");
  assert.equal(exposed.submit(), true);
  assert.equal(exposed.value, "Two");
  assert.equal(exposed.submit(), false, "not editing");
  exposed.edit();
  exposed.cancel();
  assert.equal(exposed.editing, false);
  assert.equal(exposed.setValue("Three"), true);
  assert.equal(exposed.value, "Three");
  assert.ok(exposed.root === api.root());
  api.unmount();
});

test("parts require an Editable provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(() => mountInteraction(EditablePreview), /VIZE_UI_CONTEXT_MISSING: Editable/);
    assert.throws(() => mountInteraction(EditableInput), /VIZE_UI_CONTEXT_MISSING: Editable/);
    assert.throws(
      () => mountInteraction(EditableTrigger, { props: { action: "edit" } }),
      /VIZE_UI_CONTEXT_MISSING: Editable/,
    );
  } finally {
    console.warn = warn;
  }
});
