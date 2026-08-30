import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import CopyButton from "./copy-button.vue";
import type { CopyButtonExpose, CopyButtonSlotState, CopyButtonWriter } from "./copy-button.ts";

async function settle(): Promise<void> {
  await Promise.resolve();
  await nextTick();
}

test("renders deterministic native button semantics and default label", () => {
  const handle = mountInteraction(CopyButton, {
    props: { value: "token" },
  });
  const button = handle.getByRole("button", { name: "Copy" }) as HTMLButtonElement;
  const label = button.querySelector('[data-vize-ui="copy-button-label"]');

  assert.equal(button.tagName, "BUTTON");
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("data-vize-ui"), "copy-button");
  assert.equal(button.getAttribute("part"), "root");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-disabled"), null);
  assert.equal(button.getAttribute("data-writing"), null);
  assert.equal(button.getAttribute("aria-disabled"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.getAttribute("class"), null);
  assert.equal(button.getAttribute("style"), null);
  assert.equal(button.getAttribute("data-value"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.getAttribute("part"), "label");
  assert.equal(label.textContent, "Copy");
  handle.unmount();
});

test("uses navigator.clipboard.writeText by default", async () => {
  const calls: string[] = [];
  const clipboard = {
    writeText: async (nextValue: string) => {
      calls.push(nextValue);
    },
  };
  const descriptor = Object.getOwnPropertyDescriptor(globalThis.navigator, "clipboard");
  Object.defineProperty(globalThis.navigator, "clipboard", {
    configurable: true,
    value: clipboard,
  });
  const handle = mountInteraction(CopyButton, {
    props: { value: "from navigator" },
    record: ["copy", "error"],
  });

  try {
    await handle.click(handle.getByRole("button", { name: "Copy" }));
    await settle();

    assert.deepEqual(calls, ["from navigator"]);
    assert.deepEqual(
      handle.recorded().map((emit) => emit.event),
      ["copy"],
    );
    assert.equal(handle.root().getAttribute("data-state"), "copied");
  } finally {
    handle.unmount();
    if (descriptor === undefined) {
      delete (globalThis.navigator as { clipboard?: unknown }).clipboard;
    } else {
      Object.defineProperty(globalThis.navigator, "clipboard", descriptor);
    }
  }
});

test("copies the configured value and exposes copied state", async () => {
  const writes: string[] = [];
  const writer: CopyButtonWriter = async (nextValue) => {
    writes.push(nextValue);
  };
  const handle = mountInteraction(CopyButton, {
    props: {
      value: "https://vize.dev/docs",
      writer,
    },
    record: ["copy", "error"],
  });
  const button = handle.getByRole("button", { name: "Copy" });

  await handle.click(button);
  await settle();

  assert.deepEqual(writes, ["https://vize.dev/docs"]);
  assert.equal(button.getAttribute("data-state"), "copied");
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.textContent, "Copied");
  assert.deepEqual(
    handle
      .recorded()
      .map((emit) => [emit.event, emit.payload[0], emit.payload[1] instanceof MouseEvent]),
    [["copy", "https://vize.dev/docs", true]],
  );
  assert.equal(handle.wrapper.emitted("error"), undefined);
  handle.unmount();
});

test("captures writer failures without throwing out of activation", async () => {
  const failure = new Error("denied");
  const handle = mountInteraction(CopyButton, {
    props: {
      errorLabel: "Could not copy",
      value: "secret",
      writer: async () => {
        throw failure;
      },
    },
    record: ["copy", "error"],
  });
  const button = handle.getByRole("button", { name: "Copy" });

  await handle.click(button);
  await settle();

  assert.equal(button.getAttribute("data-state"), "error");
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.textContent, "Could not copy");
  assert.equal(handle.wrapper.emitted("copy"), undefined);
  assert.deepEqual(
    handle
      .recorded()
      .map((emit) => [
        emit.event,
        emit.payload[0],
        emit.payload[1],
        emit.payload[2] instanceof MouseEvent,
      ]),
    [["error", failure, "secret", true]],
  );
  handle.unmount();
});

test("disabled copy buttons suppress writes and keep platform semantics", async () => {
  const writes: string[] = [];
  const writer: CopyButtonWriter = (nextValue) => {
    writes.push(nextValue);
  };
  const native = mountInteraction(CopyButton, {
    props: { disabled: true, value: "native", writer },
    record: ["copy", "error"],
  });
  const nativeButton = native.getByRole("button", { name: "Copy" }) as HTMLButtonElement;

  assert.equal(nativeButton.disabled, true);
  assert.equal(nativeButton.getAttribute("data-state"), "idle");
  assert.equal(nativeButton.getAttribute("data-disabled"), "true");
  assert.equal(nativeButton.getAttribute("aria-disabled"), null);
  await native.click(nativeButton);
  assert.deepEqual(writes, []);
  assert.deepEqual(native.recorded(), []);
  assert.equal(await native.tab(), null);
  native.unmount();

  const custom = mountInteraction(CopyButton, {
    props: { as: "span", disabled: true, value: "custom", writer },
    record: ["copy", "error"],
  });
  const customButton = custom.getByRole("button", { name: "Copy" });

  assert.equal(customButton.tagName, "SPAN");
  assert.equal(customButton.getAttribute("tabindex"), "-1");
  assert.equal(customButton.getAttribute("aria-disabled"), "true");
  await custom.click(customButton);
  await custom.press(customButton, "Enter");
  assert.deepEqual(writes, []);
  assert.deepEqual(custom.recorded(), []);
  custom.unmount();
});

test("supports custom labels and slot rendering", async () => {
  const handle = mountInteraction(CopyButton, {
    props: {
      copiedLabel: "Copied token",
      idleLabel: "Copy token",
      value: "token",
      writer: () => {},
    },
    slots: {
      default: (state: CopyButtonSlotState) =>
        h("span", { "data-slot-state": state.state }, `${state.label}:${state.value}`),
    },
  });
  const button = handle.getByRole("button", { name: "Copy token:token" });

  assert.equal(button.textContent, "Copy token:token");
  assert.equal(button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"), "idle");
  await handle.click(button);
  await settle();
  assert.equal(button.getAttribute("data-state"), "copied");
  assert.equal(button.textContent, "Copied token:token");
  assert.equal(
    button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"),
    "copied",
  );
  handle.unmount();
});

test("suppresses duplicate writes while a copy is in flight", async () => {
  const writes: string[] = [];
  let resolveWrite: (() => void) | null = null;
  const writer: CopyButtonWriter = (nextValue) => {
    writes.push(nextValue);
    return new Promise<void>((resolve) => {
      resolveWrite = resolve;
    });
  };
  const handle = mountInteraction(CopyButton, {
    props: { value: "once", writer },
    record: ["copy", "error"],
  });
  const button = handle.getByRole("button", { name: "Copy" });

  void handle.click(button);
  await settle();
  await handle.click(button);
  await settle();

  assert.deepEqual(writes, ["once"]);
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-writing"), "true");
  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  assert.deepEqual(handle.recorded(), []);

  assert.ok(resolveWrite);
  resolveWrite();
  await settle();
  assert.equal(button.getAttribute("data-state"), "copied");
  assert.equal(button.getAttribute("data-writing"), null);
  assert.equal(handle.recorded().length, 1);

  await handle.click(button);
  await settle();
  assert.deepEqual(writes, ["once", "once"]);
  handle.unmount();
});

test("emits the submitted value when props change while writing", async () => {
  const writes: string[] = [];
  let resolveWrite: (() => void) | null = null;
  const writer: CopyButtonWriter = (nextValue) => {
    writes.push(nextValue);
    return new Promise<void>((resolve) => {
      resolveWrite = resolve;
    });
  };
  const handle = mountInteraction(CopyButton, {
    props: { value: "initial", writer },
    record: ["copy", "error"],
  });
  const button = handle.getByRole("button", { name: "Copy" });

  void handle.click(button);
  await settle();
  await handle.wrapper.setProps({ value: "changed" });
  assert.equal(button.getAttribute("data-state"), "idle");

  assert.ok(resolveWrite);
  resolveWrite();
  await settle();

  assert.deepEqual(writes, ["initial"]);
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [["copy", "initial"]],
  );
  assert.equal(button.getAttribute("data-state"), "copied");
  handle.unmount();
});

test("non-native hosts preserve keyboard button activation", async () => {
  const writes: string[] = [];
  const handle = mountInteraction(CopyButton, {
    props: {
      as: "span",
      value: "keyboard",
      writer: (nextValue: string) => {
        writes.push(nextValue);
      },
    },
    record: ["copy"],
  });
  const button = handle.getByRole("button", { name: "Copy" });

  assert.equal(button.tagName, "SPAN");
  assert.equal(button.getAttribute("role"), "button");
  assert.equal(button.getAttribute("tabindex"), "0");
  const enter = await handle.press(button, "Enter");
  await settle();
  const space = await handle.press(button, " ");
  await settle();

  assert.equal(enter.activated, false);
  assert.equal(space.keydownPrevented, true);
  assert.deepEqual(writes, ["keyboard", "keyboard"]);
  assert.equal(handle.recorded().length, 2);
  handle.unmount();
});

test("exposes live state and focus without broad clipboard abstractions", async () => {
  const handle = mountInteraction(CopyButton, {
    props: {
      ariaLabel: "Copy invite link",
      copiedLabel: "Invite link copied",
      value: "invite",
      writer: () => {},
    },
  });
  const exposed = handle.exposes<CopyButtonExpose>();
  const button = handle.getByRole("button", { name: "Copy invite link" });

  assert.ok(exposed.element === button);
  assert.equal(exposed.disabled, false);
  assert.equal(exposed.writing, false);
  assert.equal(exposed.unavailable, false);
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.value, "invite");
  assert.equal(exposed.label, "Copy");
  exposed.focus();
  assert.ok(handle.activeElement() === button);

  await handle.click(button);
  await settle();
  assert.equal(exposed.state, "copied");
  assert.equal(exposed.label, "Invite link copied");

  await handle.wrapper.setProps({ value: "next invite" });
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.value, "next invite");
  handle.unmount();
});
