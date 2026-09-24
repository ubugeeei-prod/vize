import assert from "node:assert/strict";

import { test, vi } from "vite-plus/test";
import { h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import {
  locateTextOffset,
  measureFieldCaret,
  readFieldCaret,
  readFieldText,
  replaceEditableText,
} from "./mention-caret.ts";
import MentionContent from "./mention-content.vue";
import MentionEditable from "./mention-editable.vue";
import MentionEmpty from "./mention-empty.vue";
import MentionInput from "./mention-input.vue";
import MentionItem from "./mention-item.vue";
import MentionRoot from "./mention-root.vue";
import type { MentionTrigger } from "./mention-core.ts";
import type { MentionLoadContext, MentionRootExpose, MentionSlotState } from "./mention-types.ts";
import {
  activeOption,
  field,
  keydown,
  mentionOptions,
  options,
  people,
  settle,
  typeInto,
} from "./mention-test-utils.ts";
import type { Person } from "./mention-test-utils.ts";

function mountMention(
  rootProps: Record<string, unknown> = {},
  inputProps: Record<string, unknown> = {},
) {
  return mountInteraction(MentionRoot, mentionOptions(rootProps, inputProps));
}

test("renders a textarea with listbox autocomplete semantics and no popup", async () => {
  const handle = mountMention();
  await settle();
  const textarea = field(handle);
  assert.ok(textarea instanceof HTMLTextAreaElement);
  assert.equal(handle.root().getAttribute("data-vize-ui"), "mention");
  assert.equal(textarea.id, "composer-field");
  assert.equal(textarea.getAttribute("aria-autocomplete"), "list");
  assert.equal(textarea.getAttribute("aria-haspopup"), "listbox");
  assert.equal(textarea.getAttribute("aria-controls"), null);
  assert.equal(handle.root().querySelector("[role='listbox']"), null);
  handle.unmount();
});

test("typing a trigger opens the listbox, filters by query, and highlights the first item", async () => {
  const handle = mountMention();
  const textarea = field(handle);
  textarea.focus();
  await typeInto(textarea, "hi @a");
  const listbox = handle.root().querySelector("[role='listbox']");
  assert.equal(listbox?.id, "composer-listbox");
  assert.equal(listbox?.getAttribute("aria-label"), "People");
  assert.equal(textarea.getAttribute("aria-controls"), "composer-listbox");
  assert.equal(handle.root().getAttribute("data-trigger"), "@");
  assert.deepEqual(options(handle), ["Ada Lovelace", "Alan Turing", "Grace Hopper"]);
  assert.equal(activeOption(handle), "Ada Lovelace");
  await typeInto(textarea, "hi @al");
  assert.deepEqual(options(handle), ["Alan Turing"]);
  await typeInto(textarea, "hi @a");
  assert.deepEqual(handle.wrapper.emitted("query-change")?.at(-1), ["a", { char: "@" }]);
  assert.deepEqual(handle.wrapper.emitted("update:query")?.at(-1), ["a"]);

  await typeInto(textarea, "hi @gr");
  assert.deepEqual(options(handle), ["Grace Hopper"]);
  await typeInto(textarea, "hi @zz");
  assert.deepEqual(options(handle), []);
  const empty = handle.root().querySelector("[data-vize-ui='mention-empty']");
  assert.equal(empty?.hasAttribute("hidden"), false);
  assert.equal(empty?.textContent, "No people");
  handle.unmount();
});

test("mid-word triggers never open and moving the caret out of the token closes", async () => {
  const handle = mountMention();
  const textarea = field(handle);
  await typeInto(textarea, "mail@a");
  assert.equal(handle.root().querySelector("[role='listbox']"), null);

  await typeInto(textarea, "hi @al there", 6);
  assert.deepEqual(options(handle), ["Alan Turing"]);
  textarea.setSelectionRange(12, 12);
  textarea.dispatchEvent(new KeyboardEvent("keyup", { bubbles: true, key: "End" }));
  await settle();
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.deepEqual(handle.wrapper.emitted("query-change")?.at(-1), [null, null]);
  handle.unmount();
});

test("keyboard matrix: arrows, Home/End, Enter inserts, Tab inserts, Escape dismisses", async () => {
  const handle = mountMention({ loop: true });
  const textarea = field(handle);
  textarea.focus();
  await typeInto(textarea, "@");
  assert.equal(activeOption(handle), "Ada Lovelace");
  assert.equal(keydown(textarea, "ArrowDown").defaultPrevented, true);
  await settle();
  assert.equal(activeOption(handle), "Alan Turing");
  keydown(textarea, "End");
  await settle();
  assert.equal(activeOption(handle), "Grace Hopper");
  keydown(textarea, "ArrowDown");
  await settle();
  assert.equal(activeOption(handle), "Ada Lovelace", "loop wraps");
  keydown(textarea, "ArrowUp");
  await settle();
  assert.equal(activeOption(handle), "Grace Hopper");
  keydown(textarea, "Home");
  await settle();
  assert.equal(activeOption(handle), "Ada Lovelace");

  const enter = keydown(textarea, "Enter");
  await settle();
  assert.equal(enter.defaultPrevented, true);
  assert.equal(textarea.value, "@ada ");
  assert.equal(textarea.selectionStart, 5);
  assert.deepEqual(handle.wrapper.emitted("select"), [[people[0], { char: "@" }]]);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.at(-1), ["@ada "]);
  assert.equal(handle.root().getAttribute("data-state"), "closed");

  await typeInto(textarea, "@ada @g");
  const tab = keydown(textarea, "Tab");
  await settle();
  assert.equal(tab.defaultPrevented, true);
  assert.equal(textarea.value, "@ada @grace ");

  await typeInto(textarea, "@ada @grace @a");
  const escape = keydown(textarea, "Escape");
  await settle();
  assert.equal(escape.defaultPrevented, true);
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  await typeInto(textarea, "@ada @grace @al");
  assert.equal(handle.root().getAttribute("data-state"), "closed", "dismissal lasts for the token");
  await typeInto(textarea, "@ada @grace @al @");
  assert.equal(handle.root().getAttribute("data-state"), "open", "a new token reopens");
  handle.unmount();
});

test("keys pass through while closed and Enter without a highlight is not consumed", async () => {
  const handle = mountMention();
  const textarea = field(handle);
  await typeInto(textarea, "plain text");
  assert.equal(keydown(textarea, "ArrowDown").defaultPrevented, false);
  assert.equal(keydown(textarea, "Enter").defaultPrevented, false);
  await typeInto(textarea, "@zz");
  assert.equal(keydown(textarea, "Enter").defaultPrevented, false);
  handle.unmount();
});

test("clicking an item inserts it and pointer movement highlights", async () => {
  const handle = mountMention();
  const textarea = field(handle);
  textarea.focus();
  await typeInto(textarea, "ping @ now", 6);
  const grace = [...handle.root().querySelectorAll("[role='option']")].find(
    (option) => option.textContent === "Grace Hopper",
  );
  if (!(grace instanceof HTMLElement)) assert.fail("option expected");
  grace.dispatchEvent(new PointerEvent("pointermove", { bubbles: true, pointerType: "mouse" }));
  await settle();
  assert.equal(activeOption(handle), "Grace Hopper");
  const down = new PointerEvent("pointerdown", { bubbles: true, cancelable: true });
  grace.dispatchEvent(down);
  assert.equal(down.defaultPrevented, true, "options keep focus in the field");
  await handle.click(grace);
  await settle();
  assert.equal(textarea.value, "ping @grace now", "the trailing space merges with the next one");
  handle.unmount();
});

test("insertion transforms and multiple triggers receive the item and trigger", async () => {
  const tags = ["bug", "docs"] as const;
  const handle = mountInteraction(MentionRoot, {
    props: {
      insert: (item: string, trigger: MentionTrigger) =>
        trigger.char === "#" ? `[#${item}] ` : `<@${item}> `,
      items: ["ada", "bug", "docs"],
      triggers: [{ char: "@" }, { char: "#" }],
    },
    record: ["select"],
    slots: {
      default: (state: MentionSlotState<string>) => [
        h(MentionInput, { ariaLabel: "Message", as: "input" }),
        h(MentionContent, { portalDisabled: true }, () =>
          state.filteredItems.map((item) =>
            h(MentionItem<string>, { key: item, value: item }, () => item),
          ),
        ),
      ],
    },
  });
  const input = field(handle);
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.getAttribute("role"), "combobox");
  input.focus();
  await typeInto(input, "fix #do");
  assert.equal(input.getAttribute("aria-expanded"), "true");
  keydown(input, "Enter");
  await settle();
  assert.equal(input.value, "fix [#docs] ");
  await typeInto(input, "fix [#docs] @a");
  keydown(input, "Enter");
  await settle();
  assert.equal(input.value, "fix [#docs] <@ada> ");
  assert.deepEqual(
    handle.wrapper
      .emitted("select")
      ?.map(([item, trigger]) => [item, (trigger as MentionTrigger).char]),
    [
      [tags[1], "#"],
      ["ada", "@"],
    ],
  );
  handle.unmount();
});

test("contenteditable fields detect tokens from the selection and insert text in place", async () => {
  const handle = mountInteraction(MentionRoot, {
    props: {
      itemText: (person: Person) => person.handle,
      items: people,
    },
    record: ["select", "update:modelValue"],
    slots: {
      default: (state: MentionSlotState<Person>) => [
        h(MentionEditable, { ariaLabel: "Notes" }),
        h(MentionContent, { portalDisabled: true }, () =>
          state.filteredItems.map((person) =>
            h(MentionItem<Person>, { key: person.handle, value: person }, () => person.name),
          ),
        ),
      ],
    },
  });
  const editor = handle.root().querySelector("[data-vize-ui='mention-editable']");
  if (!(editor instanceof HTMLElement)) assert.fail("editor expected");
  assert.equal(editor.getAttribute("role"), "textbox");
  assert.equal(editor.getAttribute("contenteditable"), "true");
  assert.equal(editor.getAttribute("aria-multiline"), "true");

  editor.textContent = "hello @gr";
  const text = editor.firstChild;
  if (text === null) assert.fail("text node expected");
  const selection = document.getSelection();
  const range = document.createRange();
  range.setStart(text, 9);
  range.collapse(true);
  selection?.removeAllRanges();
  selection?.addRange(range);
  editor.dispatchEvent(new InputEvent("input", { bubbles: true }));
  await settle();
  assert.deepEqual(
    [...handle.root().querySelectorAll("[role='option']")].map((option) => option.textContent),
    ["Grace Hopper"],
  );
  const activeId = editor.getAttribute("aria-activedescendant");
  assert.equal(
    activeId === null ? null : document.getElementById(activeId)?.textContent,
    "Grace Hopper",
  );
  keydown(editor, "Enter");
  await settle();
  assert.equal(editor.textContent, "hello @grace ");
  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.at(-1), ["hello @grace "]);
  assert.equal(handle.exposes<MentionRootExpose<Person>>().match, null);
  handle.unmount();
});

test("loadItems loads per query, publishes loading, and aborts superseded requests", async () => {
  const requests: {
    query: string;
    char: string;
    signal: AbortSignal;
    resolve: (items: readonly Person[]) => void;
  }[] = [];
  const handle = mountMention({
    debounce: 0,
    items: undefined,
    loadItems: (query: string, trigger: MentionTrigger, context: MentionLoadContext) =>
      new Promise<readonly Person[]>((resolve) => {
        requests.push({ char: trigger.char, query, resolve, signal: context.signal });
      }),
  });
  const textarea = field(handle);
  textarea.focus();
  await typeInto(textarea, "@a");
  assert.deepEqual(
    requests.map((request) => [request.query, request.char]),
    [["a", "@"]],
  );
  assert.equal(handle.root().getAttribute("data-loading"), "true");
  assert.equal(
    handle.root().querySelector("[data-vize-ui='mention-empty']")?.hasAttribute("hidden"),
    true,
  );

  await typeInto(textarea, "@al");
  assert.equal(requests[0]?.signal.aborted, true);
  requests[0]?.resolve(people);
  requests[1]?.resolve(people.slice(1, 2));
  await settle();
  assert.deepEqual(options(handle), ["Alan Turing"], "loaded items are not filtered again");
  assert.equal(handle.root().getAttribute("data-loading"), null);

  await typeInto(textarea, "@al ");
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  handle.unmount();
});

test("debounced query changes abort pending loads before stale results can render", async () => {
  vi.useFakeTimers();
  try {
    const requests: {
      query: string;
      signal: AbortSignal;
      resolve: (items: readonly Person[]) => void;
    }[] = [];
    const handle = mountMention({
      debounce: 50,
      items: undefined,
      loadItems: (query: string, _trigger: MentionTrigger, context: MentionLoadContext) =>
        new Promise<readonly Person[]>((resolve) => {
          requests.push({ query, resolve, signal: context.signal });
        }),
    });
    const textarea = field(handle);
    await typeInto(textarea, "@a");
    vi.advanceTimersByTime(50);
    await settle();
    assert.equal(requests[0]?.query, "a");

    await typeInto(textarea, "@al");
    assert.equal(requests[0]?.signal.aborted, true);
    requests[0]?.resolve(people);
    await settle();
    assert.deepEqual(options(handle), [], "obsolete results stay hidden while debouncing");

    vi.advanceTimersByTime(50);
    await settle();
    assert.equal(requests[1]?.query, "al");
    requests[1]?.resolve(people.slice(1, 2));
    await settle();
    assert.deepEqual(options(handle), ["Alan Turing"]);

    await typeInto(textarea, "@alx");
    assert.deepEqual(options(handle), [], "the prior query's results clear before the debounce");
    assert.equal(handle.root().getAttribute("data-loading"), "true");
    handle.unmount();
  } finally {
    vi.useRealTimers();
  }
});

test("controlled text, filter injection, disabled state, and exposed methods", async () => {
  const handle = mountMention({
    filter: (person: Person, query: string) => person.name.toLowerCase().includes(query),
    modelValue: "hello",
  });
  const textarea = field(handle);
  assert.equal(textarea.value, "hello");
  textarea.focus();
  await typeInto(textarea, "@hop");
  assert.deepEqual(options(handle), ["Grace Hopper"], "custom filter searches names");
  const exposed = handle.exposes<MentionRootExpose<Person>>();
  assert.equal(exposed.open, true);
  exposed.dismiss();
  await settle();
  assert.equal(exposed.open, false);
  assert.deepEqual(handle.wrapper.emitted("update:modelValue")?.at(-1), ["@hop"]);
  handle.unmount();

  const disabled = mountMention({ disabled: true });
  const disabledField = field(disabled);
  assert.equal(disabledField.disabled, true);
  await typeInto(disabledField, "@a");
  assert.equal(disabled.root().getAttribute("data-state"), "closed");
  disabled.unmount();
});

test("parts require a Mention provider", () => {
  assert.throws(() => mountInteraction(MentionInput), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(MentionEmpty), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(MentionItem, { props: { value: "x" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});

test("caret helpers measure text fields and editable content without leaking mirrors", () => {
  const textarea = document.createElement("textarea");
  textarea.value = "hello @ada";
  document.body.append(textarea);
  const before = document.body.childElementCount;
  const rect = measureFieldCaret(textarea, 6);
  assert.equal(document.body.childElementCount, before, "the mirror is removed");
  assert.ok(Number.isFinite(rect.x) && Number.isFinite(rect.y));
  assert.equal(rect.width, 1);
  textarea.setSelectionRange(3, 3);
  assert.equal(readFieldCaret(textarea), 3);
  textarea.setSelectionRange(1, 4);
  assert.equal(readFieldCaret(textarea), null, "range selections have no caret");
  textarea.remove();

  const editor = document.createElement("div");
  editor.append("ab", document.createElement("br"), "cd");
  document.body.append(editor);
  const position = locateTextOffset(editor, 3);
  assert.equal(position.node.textContent, "cd");
  assert.equal(position.offset, 1);
  assert.equal(locateTextOffset(editor, 99).offset, 2, "offsets clamp to the last text node");
  assert.equal(readFieldText(editor), "abcd");
  assert.ok(Number.isFinite(measureFieldCaret(editor, 1).y));
  replaceEditableText(editor, 1, 3, "XY");
  assert.equal(editor.textContent, "aXYd");
  editor.remove();
});
