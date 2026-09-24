import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import type {
  TagsInputInvalidEvent,
  TagsInputItemExpose,
  TagsInputRootExpose,
  TagsInputSlotState,
} from "./tags-input.ts";
import TagsInputInput from "./tags-input-input.vue";
import TagsInputItem from "./tags-input-item.vue";
import TagsInputItemDelete from "./tags-input-item-delete.vue";
import TagsInputItemText from "./tags-input-item-text.vue";
import TagsInputRoot from "./tags-input-root.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const recordedEvents = ["update:modelValue", "add", "remove", "edit", "invalid"] as const;

function renderTags(state: TagsInputSlotState<unknown>) {
  return [
    ...state.tags.map((tag, index) =>
      h(TagsInputItem, { key: index, value: tag, index }, () => [
        h(TagsInputItemText),
        h(TagsInputItemDelete, null, () => "x"),
      ]),
    ),
    h(TagsInputInput, { placeholder: "Add topic" }),
  ];
}

function mountTags(props: Record<string, unknown> = {}) {
  return mountInteraction(TagsInputRoot, {
    props: { ariaLabel: "Topics", ...props },
    record: recordedEvents,
    slots: { default: renderTags },
  });
}

type Handle = ReturnType<typeof mountTags>;

function input(handle: Handle): HTMLInputElement {
  return handle.getByRole("textbox", { name: "Topics" }) as HTMLInputElement;
}

function tagTexts(handle: Handle): string[] {
  return [...handle.root().querySelectorAll("[data-vize-ui='tags-input-item']")].map(
    (item) => item.getAttribute("aria-label") ?? "",
  );
}

async function key(target: Element, value: string, init: KeyboardEventInit = {}): Promise<boolean> {
  const event = new KeyboardEvent("keydown", {
    bubbles: true,
    cancelable: true,
    key: value,
    ...init,
  });
  target.dispatchEvent(event);
  await nextTick();
  await nextTick();
  return event.defaultPrevented;
}

async function type(field: HTMLInputElement, text: string): Promise<void> {
  field.focus();
  field.value = text;
  field.setSelectionRange(text.length, text.length);
  field.dispatchEvent(new Event("input", { bubbles: true }));
  await nextTick();
}

async function paste(field: HTMLInputElement, text: string): Promise<boolean> {
  const event = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(event, "clipboardData", { value: { getData: () => text } });
  field.dispatchEvent(event);
  await nextTick();
  return event.defaultPrevented;
}

function emitted(handle: Handle, name: string): unknown[][] {
  return handle.recorded().flatMap((entry) => (entry.event === name ? [[...entry.payload]] : []));
}

test("renders tags, input semantics, ids, and data attributes", async () => {
  const handle = mountTags({
    ariaDescribedby: "topics-help",
    ariaErrormessage: "topics-error",
    ariaInvalid: true,
    defaultValue: ["vue", "vite"],
    id: "topics",
    max: 2,
    required: true,
  });
  await nextTick();
  const root = handle.root();
  const field = input(handle);
  const vue = handle.getByRole("group", { name: "vue" });

  assert.equal(root.getAttribute("data-vize-ui"), "tags-input");
  assert.equal(root.getAttribute("data-state"), "filled");
  assert.equal(root.getAttribute("data-count"), "2");
  assert.equal(root.getAttribute("data-full"), "true");
  assert.equal(root.getAttribute("data-invalid"), "true");
  assert.equal(field.id, "topics");
  assert.equal(field.placeholder, "Add topic");
  assert.equal(field.getAttribute("aria-describedby"), "topics-help");
  assert.equal(field.getAttribute("aria-errormessage"), "topics-error");
  assert.equal(field.getAttribute("aria-invalid"), "true");
  assert.equal(field.getAttribute("aria-required"), "true");
  assert.equal(field.required, false, "required is satisfied once a tag exists");
  assert.equal(vue.id, "topics-tag-0");
  assert.equal(vue.getAttribute("aria-roledescription"), "tag");
  assert.equal(vue.tabIndex, -1);
  assert.equal(vue.getAttribute("data-state"), "idle");
  assert.equal(vue.querySelector("[data-vize-ui='tags-input-item-text']")?.textContent, "vue");
  assert.ok(handle.getByRole("button", { name: "Remove vite" }));
  assert.equal(handle.getByRole("button", { name: "Remove vite" }).getAttribute("tabindex"), "-1");
  handle.unmount();
});

test("Enter and delimiter keys commit trimmed text while empty Enter passes through", async () => {
  const handle = mountTags({ delimiters: [",", ";"] });
  const field = input(handle);

  assert.equal(await key(field, "Enter"), false, "empty Enter keeps native form submission");
  await type(field, "  vue  ");
  assert.equal(await key(field, "Enter"), true);
  assert.deepEqual(tagTexts(handle), ["vue"]);
  assert.equal(field.value, "");

  await type(field, "vite");
  assert.equal(await key(field, ";"), true);
  await type(field, "pinia");
  assert.equal(await key(field, ","), true);
  assert.deepEqual(tagTexts(handle), ["vue", "vite", "pinia"]);
  assert.deepEqual(emitted(handle, "add"), [
    ["vue", 0, "enter"],
    ["vite", 1, "delimiter"],
    ["pinia", 2, "delimiter"],
  ]);
  assert.equal(handle.root().getAttribute("data-state"), "filled");
  handle.unmount();
});

test("typed delimiters commit complete segments and keep the trailing fragment", async () => {
  const handle = mountTags();
  const field = input(handle);

  await type(field, "vue,vite,pi");
  assert.deepEqual(tagTexts(handle), ["vue", "vite"]);
  assert.equal(field.value, "pi");
  handle.unmount();
});

test("pasting delimited or multi-line text splits it into tags", async () => {
  const handle = mountTags();
  const field = input(handle);

  field.focus();
  assert.equal(await paste(field, "plain"), false, "undelimited paste stays native");
  assert.equal(await paste(field, "vue, vite\npinia\tnuxt"), true);
  assert.deepEqual(tagTexts(handle), ["vue", "vite", "pinia", "nuxt"]);
  assert.deepEqual(
    emitted(handle, "add").map((payload) => payload[2]),
    ["paste", "paste", "paste", "paste"],
  );
  handle.unmount();

  const disabledPaste = mountTags({ addOnPaste: false });
  assert.equal(await paste(input(disabledPaste), "a,b"), false);
  assert.deepEqual(tagTexts(disabledPaste), []);
  disabledPaste.unmount();
});

test("duplicates, max, parse failures, and validators emit invalid events", async () => {
  const handle = mountTags({
    defaultValue: ["vue"],
    max: 3,
    parseTag: (text: string) => (text === "?" ? null : text.toLowerCase()),
    validate: (tag: string) => (tag.length > 5 ? "Too long" : tag !== "bad"),
  });
  const field = input(handle);

  await type(field, "VUE");
  await key(field, "Enter");
  assert.equal(field.value, "VUE", "rejected text stays editable");
  await type(field, "?");
  await key(field, "Enter");
  await type(field, "bad");
  await key(field, "Enter");
  await type(field, "toolong");
  await key(field, "Enter");
  await type(field, "a,b,");
  await type(field, "c");
  await key(field, "Enter");

  const invalid = emitted(handle, "invalid").map(
    ([event]) => event as TagsInputInvalidEvent<string>,
  );
  assert.deepEqual(
    invalid.map(({ reason, message, text, tag }) => [reason, message, text, tag]),
    [
      ["duplicate", null, "VUE", "vue"],
      ["parse", null, "?", null],
      ["invalid", null, "bad", "bad"],
      ["invalid", "Too long", "toolong", "toolong"],
      ["max", null, "c", "c"],
    ],
  );
  assert.deepEqual(tagTexts(handle), ["vue", "a", "b"]);
  assert.equal(handle.root().getAttribute("data-full"), "true");
  handle.unmount();
});

test("allowDuplicates and by keys control duplicate detection", async () => {
  const duplicates = mountTags({ allowDuplicates: true, defaultValue: ["vue"] });
  await type(input(duplicates), "vue");
  await key(input(duplicates), "Enter");
  assert.deepEqual(tagTexts(duplicates), ["vue", "vue"]);
  duplicates.unmount();

  const byComparator = mountTags({
    by: (left: string, right: string) => left.toLowerCase() === right.toLowerCase(),
    defaultValue: ["Vue"],
  });
  await type(input(byComparator), "vue");
  await key(input(byComparator), "Enter");
  assert.deepEqual(tagTexts(byComparator), ["Vue"]);
  assert.equal(emitted(byComparator, "invalid").length, 1);
  byComparator.unmount();

  const byKey = mountTags({
    by: "id",
    defaultValue: [{ id: 1, label: "Vue" }],
    parseTag: (text: string) => ({ id: text.length, label: text }),
    tagText: (tag: { label: string }) => tag.label,
  });
  await type(input(byKey), "X");
  await key(input(byKey), "Enter");
  await type(input(byKey), "XY");
  await key(input(byKey), "Enter");
  assert.deepEqual(tagTexts(byKey), ["Vue", "XY"]);
  byKey.unmount();
});

test("keyboard matrix moves focus between tags and removes with Backspace and Delete", async () => {
  const handle = mountTags({ defaultValue: ["a", "b", "c", "d"] });
  const field = input(handle);
  field.focus();
  field.setSelectionRange(0, 0);

  assert.equal(await key(field, "Backspace"), true);
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "d");
  assert.equal(
    handle.activeElement()?.getAttribute("data-active"),
    "true",
    "focused tag publishes data-active",
  );
  await key(handle.activeElement()!, "ArrowLeft");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "c");
  await key(handle.activeElement()!, "Home");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "a");
  await key(handle.activeElement()!, "ArrowLeft");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "a", "no wrap at start");
  await key(handle.activeElement()!, "ArrowRight");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "b");

  await key(handle.activeElement()!, "Backspace");
  assert.deepEqual(tagTexts(handle), ["a", "c", "d"]);
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "a");

  await key(handle.activeElement()!, "Delete");
  assert.deepEqual(tagTexts(handle), ["c", "d"]);
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "c");

  await key(handle.activeElement()!, "End");
  assert.ok(handle.activeElement() === field);

  await key(field, "ArrowLeft");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "d");
  await key(handle.activeElement()!, "ArrowRight");
  assert.ok(handle.activeElement() === field, "ArrowRight past the last tag returns to input");

  await key(field, "Backspace");
  await key(handle.activeElement()!, "Delete");
  assert.ok(handle.activeElement() === field, "deleting the last slot returns to input");
  assert.deepEqual(emitted(handle, "remove"), [
    ["b", 1, "backspace"],
    ["a", 0, "delete"],
    ["d", 1, "delete"],
  ]);
  handle.unmount();
});

test("Backspace and ArrowLeft keep caret editing when text precedes the caret", async () => {
  const handle = mountTags({ defaultValue: ["a"] });
  const field = input(handle);
  await type(field, "xy");

  assert.equal(await key(field, "Backspace"), false);
  assert.equal(await key(field, "ArrowLeft"), false);
  assert.ok(handle.activeElement() === field);
  handle.unmount();
});

test("rtl direction mirrors horizontal arrows", async () => {
  const handle = mountTags({ defaultValue: ["a", "b"], dir: "rtl" });
  const field = input(handle);
  field.focus();
  field.setSelectionRange(0, 0);

  assert.equal(handle.root().getAttribute("dir"), "rtl");
  assert.equal(await key(field, "ArrowLeft"), false);
  await key(field, "ArrowRight");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "b");
  await key(handle.activeElement()!, "ArrowRight");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "a");
  await key(handle.activeElement()!, "ArrowLeft");
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "b");
  handle.unmount();
});

test("the delete button removes its tag and keeps focus in the field", async () => {
  const handle = mountTags({ defaultValue: ["a", "b"] });

  await handle.click(handle.getByRole("button", { name: "Remove a" }));
  await nextTick();
  assert.deepEqual(tagTexts(handle), ["b"]);
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "b");
  await handle.click(handle.getByRole("button", { name: "Remove b" }));
  await nextTick();
  assert.deepEqual(tagTexts(handle), []);
  assert.ok(handle.activeElement() === input(handle));
  assert.deepEqual(emitted(handle, "remove"), [
    ["a", 0, "delete-button"],
    ["b", 0, "delete-button"],
  ]);
  assert.equal(handle.root().getAttribute("data-state"), "empty");
  handle.unmount();
});

test("editable tags edit inline with Enter, F2, and double-click", async () => {
  const handle = mountTags({ defaultValue: ["a", "b"], editable: true });
  const first = handle.getByRole("group", { name: "a" });
  first.focus();

  assert.equal(await key(first, "Enter"), true);
  const editor = handle.getByRole("textbox", { name: "Edit tag a" }) as HTMLInputElement;
  assert.equal(first.getAttribute("data-state"), "editing");
  assert.ok(handle.activeElement() === editor);
  assert.equal(editor.value, "a");
  editor.value = "b";
  await key(editor, "Enter");
  assert.ok(handle.getByRole("textbox", { name: "Edit tag a" }), "duplicate keeps editing");
  editor.value = "alpha";
  await key(editor, "Enter");
  await nextTick();
  assert.deepEqual(tagTexts(handle), ["alpha", "b"]);
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "alpha");
  assert.deepEqual(emitted(handle, "edit"), [["alpha", "a", 0]]);
  assert.equal(emitted(handle, "invalid").length, 1);

  const second = handle.getByRole("group", { name: "b" });
  second.focus();
  await key(second, "F2");
  const secondEditor = handle.getByRole("textbox", { name: "Edit tag b" }) as HTMLInputElement;
  secondEditor.value = "changed";
  await key(secondEditor, "Escape");
  await nextTick();
  assert.deepEqual(tagTexts(handle), ["alpha", "b"]);
  assert.equal(handle.activeElement()?.getAttribute("aria-label"), "b");

  second.dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
  await nextTick();
  await nextTick();
  const blurEditor = handle.getByRole("textbox", { name: "Edit tag b" }) as HTMLInputElement;
  blurEditor.value = "beta";
  blurEditor.dispatchEvent(new FocusEvent("blur"));
  await nextTick();
  assert.deepEqual(tagTexts(handle), ["alpha", "beta"]);
  handle.unmount();

  const readOnlyEdit = mountTags({ defaultValue: ["a"], editable: false });
  const tag = readOnlyEdit.getByRole("group", { name: "a" });
  tag.focus();
  assert.equal(await key(tag, "Enter"), false);
  assert.equal(readOnlyEdit.queryByRole("textbox", { name: "Edit tag a" }), null);
  readOnlyEdit.unmount();
});

test("disabled and readonly fields block every change", async () => {
  const disabled = mountTags({ defaultValue: ["a"], disabled: true, name: "topics" });
  const disabledField = input(disabled);
  const disabledTag = disabled.getByRole("group", { name: "a" });
  assert.equal(disabledField.disabled, true);
  assert.equal(disabledTag.hasAttribute("tabindex"), false);
  assert.equal(disabledTag.getAttribute("aria-disabled"), "true");
  assert.equal(disabled.root().getAttribute("data-state"), "disabled");
  assert.equal(
    (disabled.root().querySelector("input[type='hidden']") as HTMLInputElement).disabled,
    true,
  );
  await key(disabledTag, "Backspace");
  assert.deepEqual(tagTexts(disabled), ["a"]);
  disabled.unmount();

  const readonly = mountTags({ defaultValue: ["a"], editable: true, readonly: true });
  const readonlyField = input(readonly);
  const readonlyTag = readonly.getByRole("group", { name: "a" });
  assert.equal(readonlyField.readOnly, true);
  assert.equal(readonly.root().getAttribute("data-state"), "readonly");
  assert.equal(readonly.getByRole("button", { name: "Remove a" }).hasAttribute("disabled"), true);
  await type(readonlyField, "b,");
  readonlyTag.focus();
  await key(readonlyTag, "Delete");
  await key(readonlyTag, "Enter");
  assert.deepEqual(tagTexts(readonly), ["a"]);
  assert.equal(readonly.recorded().length, 0);
  readonly.unmount();
});

test("submits one form entry per tag, validates required, and restores on form reset", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", { id: "profile" }, [
        h(
          TagsInputRoot,
          {
            ariaLabel: "Topics",
            defaultValue: [1, 2],
            name: "topics",
            parseTag: (text: string) => {
              const parsed = Number(text);
              return Number.isInteger(parsed) ? parsed : null;
            },
            required: true,
            tagText: (tag: number) => `#${tag}`,
          },
          { default: renderTags },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root() as HTMLFormElement;
  const field = input(handle as Handle);

  assert.deepEqual(new FormData(form).getAll("topics"), ["#1", "#2"]);
  assert.equal(new FormData(form).has(""), false, "the visible input is not submitted");
  await type(field, "3");
  await key(field, "Enter");
  assert.deepEqual(new FormData(form).getAll("topics"), ["#1", "#2", "#3"]);

  field.setSelectionRange(0, 0);
  for (let count = 0; count < 3; count++) {
    if (handle.activeElement() === field) await key(field, "Backspace");
    await key(handle.activeElement()!, "Backspace");
  }
  await nextTick();
  assert.deepEqual(new FormData(form).getAll("topics"), []);
  assert.equal(field.required, true);
  assert.equal(form.checkValidity(), false);

  form.reset();
  await nextTick();
  await nextTick();
  assert.deepEqual(new FormData(form).getAll("topics"), ["#1", "#2"]);
  assert.equal(field.required, false);
  handle.unmount();
});

test("controlled tags wait for the parent to accept updates", async () => {
  const handle = mountTags({ modelValue: ["a"] });
  const field = input(handle);
  await type(field, "b");
  await key(field, "Enter");

  assert.deepEqual(tagTexts(handle), ["a"]);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[["a", "b"]]]);
  await handle.wrapper.setProps({ modelValue: ["a", "b"] });
  assert.deepEqual(tagTexts(handle), ["a", "b"]);
  handle.unmount();
});

test("addOnBlur commits on blur and IME composition never commits", async () => {
  const handle = mountTags({ addOnBlur: true });
  const field = input(handle);
  await type(field, "ime");
  assert.equal(await key(field, "Enter", { isComposing: true }), false);
  assert.equal(await key(field, ",", { isComposing: true }), false);
  assert.deepEqual(tagTexts(handle), []);
  field.dispatchEvent(new FocusEvent("blur"));
  await nextTick();
  assert.deepEqual(tagTexts(handle), ["ime"]);
  assert.deepEqual(emitted(handle, "add"), [["ime", 0, "blur"]]);
  handle.unmount();

  const noBlur = mountTags();
  await type(input(noBlur), "kept");
  input(noBlur).dispatchEvent(new FocusEvent("blur"));
  await nextTick();
  assert.deepEqual(tagTexts(noBlur), []);
  assert.equal(input(noBlur).value, "kept");
  noBlur.unmount();
});

test("pointerdown on empty root space focuses the input", async () => {
  const handle = mountTags({ defaultValue: ["a"] });
  const event = new PointerEvent("pointerdown", { bubbles: true, cancelable: true });
  handle.root().dispatchEvent(event);
  await nextTick();
  assert.equal(event.defaultPrevented, true);
  assert.ok(handle.activeElement() === input(handle));
  handle.unmount();
});

test("exposed root and item methods share state", async () => {
  let root: TagsInputRootExpose<string> | null = null;
  let firstItem: TagsInputItemExpose | null = null;
  const tags = ref<readonly string[]>(["a"]);
  const Probe = defineComponent({
    setup: () => () =>
      h(
        TagsInputRoot,
        {
          ariaLabel: "Topics",
          defaultValue: ["a"],
          editable: true,
          ref: (value: unknown) => {
            root = value as TagsInputRootExpose<string> | null;
          },
          "onUpdate:modelValue": (value: readonly string[]) => {
            tags.value = value;
          },
        },
        {
          default: (state: TagsInputSlotState<string>) => [
            ...state.tags.map((tag, index) =>
              h(TagsInputItem, {
                index,
                key: index,
                ref:
                  index === 0
                    ? (value: unknown) => {
                        firstItem = value as TagsInputItemExpose | null;
                      }
                    : undefined,
                value: tag,
              }),
            ),
            h(TagsInputInput),
          ],
        },
      ),
  });
  const handle = mountInteraction(Probe);
  if (root === null) assert.fail("TagsInputRoot must expose its API");
  const api: TagsInputRootExpose<string> = root;

  assert.equal(api.add("b"), true);
  assert.equal(api.add("b"), false);
  assert.equal(api.addTag("c"), true);
  await nextTick();
  assert.deepEqual(api.tags, ["a", "b", "c"]);
  assert.equal(api.remove(1), true);
  assert.equal(api.remove(9), false);
  api.setInputValue("draft");
  await nextTick();
  assert.equal(api.inputValue, "draft");
  api.focus();
  assert.equal((handle.activeElement() as HTMLInputElement).value, "draft");
  if (firstItem === null) assert.fail("TagsInputItem must expose its API");
  const item: TagsInputItemExpose = firstItem;
  item.focus();
  assert.equal(handle.activeElement()?.getAttribute("data-index"), "0");
  assert.equal(item.edit(), true);
  await nextTick();
  assert.equal(
    handle.root().querySelector("[data-state='editing']")?.getAttribute("data-index"),
    "0",
  );
  assert.equal(api.clear(), true);
  await nextTick();
  assert.deepEqual(api.tags, []);
  assert.equal(api.reset(), true);
  await nextTick();
  assert.deepEqual(api.tags, ["a"]);
  assert.equal(api.inputValue, "");
  assert.equal(api.state, "filled");
  assert.deepEqual(tags.value, ["a"]);
  handle.unmount();
});

test("items and parts require a matching TagsInput provider", () => {
  for (const component of [TagsInputItem, TagsInputInput]) {
    assert.throws(
      () => mountInteraction(component, { props: { index: 0, value: "orphan" } }),
      /VIZE_UI_CONTEXT_MISSING/,
    );
  }
  for (const component of [TagsInputItemText, TagsInputItemDelete]) {
    assert.throws(() => mountInteraction(component), /VIZE_UI_CONTEXT_MISSING/);
  }
});
