import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { mountInteraction } from "../../../testing/mount.ts";
import ShareButton from "./share-button.vue";
import { createShareFile, settle } from "./share-button-test-utils.ts";
import type { ShareButtonAction, ShareButtonPayload } from "./share-button.ts";

test("renders deterministic native button semantics and default label", () => {
  const handle = mountInteraction(ShareButton);
  const button = handle.getByRole("button", { name: "Share" }) as HTMLButtonElement;
  const label = button.querySelector('[data-vize-ui="share-button-label"]');

  assert.equal(button.tagName, "BUTTON");
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("data-vize-ui"), "share-button");
  assert.equal(button.getAttribute("part"), "root");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-disabled"), null);
  assert.equal(button.getAttribute("data-sharing"), null);
  assert.equal(button.getAttribute("aria-disabled"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.getAttribute("class"), null);
  assert.equal(button.getAttribute("style"), null);
  assert.equal(button.getAttribute("data-title"), null);
  assert.equal(button.getAttribute("data-text"), null);
  assert.equal(button.getAttribute("data-url"), null);
  assert.equal(button.getAttribute("data-files"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.getAttribute("part"), "label");
  assert.equal(label.textContent, "Share");
  handle.unmount();
});

test("uses navigator.share by default", async () => {
  const calls: ShareButtonPayload[] = [];
  const file = createShareFile();
  const files = [file];
  const descriptor = Object.getOwnPropertyDescriptor(globalThis.navigator, "share");
  Object.defineProperty(globalThis.navigator, "share", {
    configurable: true,
    value: async (payload: ShareButtonPayload) => {
      calls.push(payload);
    },
  });
  const handle = mountInteraction(ShareButton, {
    props: {
      files,
      text: "Read the docs",
      title: "Vize docs",
      url: "https://vize.dev/docs",
    },
    record: ["share", "error"],
  });

  try {
    await handle.click(handle.getByRole("button", { name: "Share" }));
    await settle();

    assert.equal(calls.length, 1);
    assert.deepEqual(calls[0], {
      files: [file],
      text: "Read the docs",
      title: "Vize docs",
      url: "https://vize.dev/docs",
    });
    assert.notEqual(calls[0]?.files, files);
    assert.deepEqual(
      handle.recorded().map((emit) => emit.event),
      ["share"],
    );
    assert.equal(handle.root().getAttribute("data-state"), "shared");
  } finally {
    handle.unmount();
    if (descriptor === undefined) {
      delete (globalThis.navigator as { share?: unknown }).share;
    } else {
      Object.defineProperty(globalThis.navigator, "share", descriptor);
    }
  }
});

test("emits a stable error when navigator.share is unavailable", async () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis.navigator, "share");
  Object.defineProperty(globalThis.navigator, "share", {
    configurable: true,
    value: undefined,
  });
  const handle = mountInteraction(ShareButton, {
    props: { title: "Unavailable share" },
    record: ["share", "error"],
  });

  try {
    await handle.click(handle.getByRole("button", { name: "Share" }));
    await settle();

    assert.equal(handle.root().getAttribute("data-state"), "error");
    assert.equal(handle.wrapper.emitted("share"), undefined);
    const recorded = handle.recorded();
    assert.equal(recorded.length, 1);
    const error = recorded[0]?.payload[0];
    assert.ok(error instanceof Error);
    assert.equal(error.message, "VIZE_UI_SHARE_BUTTON_ACTION_UNAVAILABLE");
    assert.deepEqual(recorded[0]?.payload[1], { title: "Unavailable share" });
    assert.ok(recorded[0]?.payload[2] instanceof MouseEvent);
  } finally {
    handle.unmount();
    if (descriptor === undefined) {
      delete (globalThis.navigator as { share?: unknown }).share;
    } else {
      Object.defineProperty(globalThis.navigator, "share", descriptor);
    }
  }
});

test("runs the configured action and exposes shared state", async () => {
  const file = createShareFile("report.txt");
  const calls: { readonly payload: ShareButtonPayload; readonly event: MouseEvent }[] = [];
  const action: ShareButtonAction = async (payload, event) => {
    calls.push({ event, payload });
  };
  const handle = mountInteraction(ShareButton, {
    props: {
      action,
      files: [file],
      title: "Quarterly report",
      url: "https://vize.dev/report",
    },
    record: ["share", "error"],
  });
  const button = handle.getByRole("button", { name: "Share" });

  await handle.click(button);
  await settle();

  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0]?.payload, {
    files: [file],
    title: "Quarterly report",
    url: "https://vize.dev/report",
  });
  assert.equal("text" in (calls[0]?.payload ?? {}), false);
  assert.ok(calls[0]?.event instanceof MouseEvent);
  assert.equal(button.getAttribute("data-state"), "shared");
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.textContent, "Shared");
  assert.deepEqual(
    handle
      .recorded()
      .map((emit) => [emit.event, emit.payload[0], emit.payload[1] instanceof MouseEvent]),
    [["share", calls[0]?.payload, true]],
  );
  assert.equal(handle.wrapper.emitted("error"), undefined);
  handle.unmount();
});

test("captures action failures without throwing out of activation", async () => {
  const failure = new Error("share denied");
  const handle = mountInteraction(ShareButton, {
    props: {
      action: async () => {
        throw failure;
      },
      errorLabel: "Could not share",
      text: "Confidential",
    },
    record: ["share", "error"],
  });
  const button = handle.getByRole("button", { name: "Share" });

  await handle.click(button);
  await settle();

  assert.equal(button.getAttribute("data-state"), "error");
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.textContent, "Could not share");
  assert.equal(handle.wrapper.emitted("share"), undefined);
  assert.deepEqual(
    handle
      .recorded()
      .map((emit) => [
        emit.event,
        emit.payload[0],
        emit.payload[1],
        emit.payload[2] instanceof MouseEvent,
      ]),
    [["error", failure, { text: "Confidential" }, true]],
  );
  handle.unmount();
});

test("disabled share buttons suppress actions and keep availability hooks", async () => {
  let calls = 0;
  const action: ShareButtonAction = () => {
    calls += 1;
  };
  const native = mountInteraction(ShareButton, {
    props: { action, disabled: true, title: "native" },
    record: ["share", "error"],
  });
  const nativeButton = native.getByRole("button", { name: "Share" }) as HTMLButtonElement;

  assert.equal(nativeButton.disabled, true);
  assert.equal(nativeButton.getAttribute("data-state"), "idle");
  assert.equal(nativeButton.getAttribute("data-disabled"), "true");
  assert.equal(nativeButton.getAttribute("aria-disabled"), null);
  await native.click(nativeButton);
  assert.equal(calls, 0);
  assert.deepEqual(native.recorded(), []);
  assert.equal(await native.tab(), null);
  native.unmount();

  const custom = mountInteraction(ShareButton, {
    props: { action, as: "span", disabled: true, title: "custom" },
    record: ["share", "error"],
  });
  const customButton = custom.getByRole("button", { name: "Share" });

  assert.equal(customButton.tagName, "SPAN");
  assert.equal(customButton.getAttribute("tabindex"), "-1");
  assert.equal(customButton.getAttribute("aria-disabled"), "true");
  await custom.click(customButton);
  await custom.press(customButton, "Enter");
  assert.equal(calls, 0);
  assert.deepEqual(custom.recorded(), []);
  custom.unmount();
});

test("suppresses duplicate actions while sharing is in flight", async () => {
  let calls = 0;
  let resolveShare: (() => void) | null = null;
  const action: ShareButtonAction = () => {
    calls += 1;
    return new Promise<void>((resolve) => {
      resolveShare = resolve;
    });
  };
  const handle = mountInteraction(ShareButton, {
    props: { action, title: "One share" },
    record: ["share", "error"],
  });
  const button = handle.getByRole("button", { name: "Share" });

  void handle.click(button);
  await settle();
  await handle.click(button);
  await settle();

  assert.equal(calls, 1);
  assert.equal(button.getAttribute("data-state"), "sharing");
  assert.equal(button.getAttribute("data-sharing"), "true");
  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  assert.equal(button.textContent, "Sharing");
  assert.deepEqual(handle.recorded(), []);

  assert.ok(resolveShare);
  resolveShare();
  await settle();
  assert.equal(button.getAttribute("data-state"), "shared");
  assert.equal(button.getAttribute("data-sharing"), null);
  assert.equal(handle.recorded().length, 1);

  await handle.click(button);
  await settle();
  assert.equal(calls, 2);
  handle.unmount();
});

test("uses the submitted action and payload when props change", async () => {
  const calls: string[] = [];
  let resolveFirst: (() => void) | null = null;
  const firstAction: ShareButtonAction = (payload) => {
    calls.push(`first:${payload.title ?? ""}:${payload.url ?? ""}`);
    return new Promise<void>((resolve) => {
      resolveFirst = resolve;
    });
  };
  const secondAction: ShareButtonAction = (payload) => {
    calls.push(`second:${payload.title ?? ""}:${payload.url ?? ""}`);
  };
  const handle = mountInteraction(ShareButton, {
    props: {
      action: firstAction,
      title: "Initial",
      url: "https://vize.dev/initial",
    },
    record: ["share", "error"],
  });
  const button = handle.getByRole("button", { name: "Share" });

  void handle.click(button);
  await settle();
  await handle.wrapper.setProps({
    action: secondAction,
    title: "Changed",
    url: "https://vize.dev/changed",
  });

  assert.ok(resolveFirst);
  resolveFirst();
  await settle();

  assert.deepEqual(calls, ["first:Initial:https://vize.dev/initial"]);
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [["share", { title: "Initial", url: "https://vize.dev/initial" }]],
  );
  assert.equal(button.getAttribute("data-state"), "shared");
  handle.unmount();
});

test("non-native hosts preserve keyboard button activation", async () => {
  let calls = 0;
  const handle = mountInteraction(ShareButton, {
    props: {
      action: () => {
        calls += 1;
      },
      as: "span",
      title: "Keyboard share",
    },
    record: ["share"],
  });
  const button = handle.getByRole("button", { name: "Share" });

  assert.equal(button.tagName, "SPAN");
  assert.equal(button.getAttribute("role"), "button");
  assert.equal(button.getAttribute("tabindex"), "0");
  const enter = await handle.press(button, "Enter");
  await settle();
  const space = await handle.press(button, " ");
  await settle();

  assert.equal(enter.activated, false);
  assert.equal(space.keydownPrevented, true);
  assert.equal(calls, 2);
  assert.equal(handle.recorded().length, 2);
  handle.unmount();
});
