import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import SearchField from "./search-field.vue";
import { mountInteraction } from "./testing/mount.ts";

function dispatchSearchInput(input: HTMLInputElement, value: string): void {
  input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true, cancelable: true }));
}

function dispatchChange(input: HTMLInputElement): void {
  input.dispatchEvent(new Event("change", { bubbles: true, cancelable: true }));
}

function dispatchSearch(input: HTMLInputElement): void {
  input.dispatchEvent(new Event("search", { bubbles: true, cancelable: true }));
}

function dispatchComposition(input: HTMLInputElement, type: "compositionend" | "compositionstart") {
  input.dispatchEvent(new CompositionEvent(type, { bubbles: true, cancelable: true }));
}

test("renders a named native searchbox with root landmark and accessibility attributes", () => {
  const handle = mountInteraction(SearchField, {
    props: {
      id: "site-query",
      name: "query",
      ariaLabel: "Search site",
      ariaDescribedby: "query-help",
      ariaErrormessage: "query-error",
      ariaInvalid: "spelling",
      autocomplete: "off",
      placeholder: "Search docs",
      required: true,
      showClear: "always",
    },
    slots: { clear: ({ empty }: { empty: boolean }) => (empty ? "Empty" : "Clear") },
  });
  const root = handle.getByRole("search");
  const input = handle.getByRole("searchbox", { name: "Search site" }) as HTMLInputElement;

  assert.equal(root.getAttribute("data-vize-ui"), "search-field");
  assert.equal(root.getAttribute("data-empty"), "true");
  assert.equal(input.id, "site-query");
  assert.equal(input.name, "query");
  assert.equal(input.type, "search");
  assert.equal(input.required, true);
  assert.equal(input.getAttribute("autocomplete"), "off");
  assert.equal(input.getAttribute("enterkeyhint"), "search");
  assert.equal(input.getAttribute("inputmode"), "search");
  assert.equal(input.getAttribute("placeholder"), "Search docs");
  assert.equal(input.getAttribute("aria-describedby"), "query-help");
  assert.equal(input.getAttribute("aria-errormessage"), "query-error");
  assert.equal(input.getAttribute("aria-invalid"), "spelling");
  assert.equal(input.getAttribute("data-vize-ui"), "search-field-input");

  const clear = handle.getByRole("button", { name: "Clear search" }) as HTMLButtonElement;
  assert.equal(clear.id, "site-query-clear");
  assert.equal(clear.disabled, true);
  assert.equal(clear.textContent, "Empty");
  handle.unmount();
});

test("uncontrolled search field emits model before input, change, and search", async () => {
  const handle = mountInteraction(SearchField, {
    props: { ariaLabel: "Search" },
    record: ["update:modelValue", "input", "change", "search"],
  });
  const input = handle.getByRole("searchbox") as HTMLInputElement;

  dispatchSearchInput(input, "atelier");
  await nextTick();
  dispatchChange(input);
  dispatchSearch(input);
  await nextTick();

  assert.equal(input.value, "atelier");
  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["update:modelValue", "atelier"],
      ["input", "atelier"],
      ["change", "atelier"],
      ["search", "atelier"],
    ],
  );
  handle.unmount();
});

test("controlled search value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(SearchField, {
    props: { ariaLabel: "Search", modelValue: "vue" },
    record: ["update:modelValue", "input"],
  });
  const input = handle.getByRole("searchbox") as HTMLInputElement;

  dispatchSearchInput(input, "vize");
  await nextTick();

  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["update:modelValue", "vize"],
      ["input", "vize"],
    ],
  );
  assert.equal(input.value, "vue");

  await handle.wrapper.setProps({ modelValue: "vize" });
  assert.equal(input.value, "vize");
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(SearchField, {
          ariaLabel: "Search",
          defaultValue: "initial",
          name: "query",
        }),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const input = handle.getByRole("searchbox", { name: "Search" }) as HTMLInputElement;

  assert.equal(input.value, "initial");
  dispatchSearchInput(input, "changed");
  await nextTick();
  assert.equal(input.value, "changed");

  form.reset();
  await nextTick();
  assert.equal(input.value, "initial");
  handle.unmount();
});

test("clear button updates before clear event and returns focus to the searchbox", async () => {
  const handle = mountInteraction(SearchField, {
    props: { ariaLabel: "Search", defaultValue: "vize" },
    record: ["update:modelValue", "clear"],
  });
  const input = handle.getByRole("searchbox") as HTMLInputElement;
  const clear = handle.getByRole("button", { name: "Clear search" }) as HTMLButtonElement;

  clear.focus();
  await handle.click(clear);

  assert.equal(input.value, "");
  assert.ok(handle.activeElement() === input);
  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["update:modelValue", ""],
      ["clear", ""],
    ],
  );
  assert.ok(handle.recorded()[1]?.payload[1] instanceof MouseEvent);
  handle.unmount();
});

test("clear visibility and availability follow empty, disabled, and readonly state", async () => {
  const auto = mountInteraction(SearchField, { props: { ariaLabel: "Search" } });
  assert.equal(auto.queryByRole("button", { name: "Clear search" }), null);
  auto.unmount();

  const always = mountInteraction(SearchField, {
    props: { ariaLabel: "Search", showClear: "always" },
  });
  const alwaysClear = always.getByRole("button", { name: "Clear search" }) as HTMLButtonElement;
  assert.equal(alwaysClear.disabled, true);
  always.unmount();

  const readOnly = mountInteraction(SearchField, {
    props: { ariaLabel: "Search", defaultValue: "vize", readOnly: true },
  });
  const readOnlyClear = readOnly.getByRole("button", {
    name: "Clear search",
  }) as HTMLButtonElement;
  assert.equal(readOnlyClear.disabled, true);
  assert.ok((await readOnly.tab()) === readOnly.getByRole("searchbox"));
  readOnly.unmount();

  const disabled = mountInteraction(SearchField, {
    props: { ariaLabel: "Search", defaultValue: "vize", disabled: true },
  });
  const disabledClear = disabled.getByRole("button", {
    name: "Clear search",
  }) as HTMLButtonElement;
  assert.equal(disabledClear.disabled, true);
  assert.ok((await disabled.tab()) === null);
  disabled.unmount();
});

test("tracks IME composition without rewriting controlled native text", async () => {
  const handle = mountInteraction(SearchField, {
    props: { ariaLabel: "Search", modelValue: "" },
    record: ["update:modelValue", "input", "compositionStart", "compositionEnd"],
  });
  const input = handle.getByRole("searchbox") as HTMLInputElement;

  dispatchComposition(input, "compositionstart");
  dispatchSearchInput(input, "検");
  await nextTick();

  assert.equal(input.value, "検");
  assert.equal(input.getAttribute("data-composing"), "true");
  assert.equal(input.getAttribute("data-empty"), "false");
  assert.equal(handle.exposes<{ composing: boolean }>().composing, true);

  dispatchComposition(input, "compositionend");
  await nextTick();

  assert.equal(input.value, "");
  assert.equal(input.getAttribute("data-composing"), "false");
  assert.equal(input.getAttribute("data-empty"), "true");
  assert.deepEqual(
    handle.recorded().map((emitted) => [emitted.event, emitted.payload[0]]),
    [
      ["compositionStart", ""],
      ["update:modelValue", "検"],
      ["input", "検"],
      ["compositionEnd", "検"],
    ],
  );
  const recorded = handle.recorded();
  assert.ok(recorded[0]?.payload[1] instanceof CompositionEvent);
  assert.ok(recorded[3]?.payload[1] instanceof CompositionEvent);

  await handle.wrapper.setProps({ modelValue: "検索" });
  assert.equal(input.value, "検索");
  handle.unmount();
});

test("exposes value mutation, clear, selection, focus, and reset controls", async () => {
  const handle = mountInteraction(SearchField, {
    props: { ariaLabel: "Search", defaultValue: "initial" },
  });
  const input = handle.getByRole("searchbox") as HTMLInputElement;
  const exposed = handle.exposes<{
    clear: () => boolean;
    focus: (options?: FocusOptions) => void;
    reset: () => boolean;
    select: () => void;
    setValue: (value: string) => boolean;
  }>();

  assert.equal(exposed.setValue("vize"), true);
  await nextTick();
  assert.equal(input.value, "vize");

  input.blur();
  exposed.focus();
  assert.ok(handle.activeElement() === input);

  exposed.select();
  assert.equal(input.selectionStart, 0);
  assert.equal(input.selectionEnd, "vize".length);

  assert.equal(exposed.clear(), true);
  await nextTick();
  assert.equal(input.value, "");
  assert.ok(handle.activeElement() === input);

  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(input.value, "initial");
  handle.unmount();
});
