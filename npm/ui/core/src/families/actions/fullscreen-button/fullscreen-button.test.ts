import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { mountInteraction } from "../../../testing/mount.ts";
import { createControllerRig, recordedOperation, settle } from "./fullscreen-button-test-utils.ts";
import FullscreenButton from "./fullscreen-button.vue";
import type { FullscreenButtonController, FullscreenButtonOperation } from "./fullscreen-button.ts";

test("renders deterministic native button semantics and default label", () => {
  const handle = mountInteraction(FullscreenButton);
  const button = handle.getByRole("button", { name: "Enter fullscreen" }) as HTMLButtonElement;
  const label = button.querySelector('[data-vize-ui="fullscreen-button-label"]');

  assert.equal(button.tagName, "BUTTON");
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("data-vize-ui"), "fullscreen-button");
  assert.equal(button.getAttribute("part"), "root");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-disabled"), null);
  assert.equal(button.getAttribute("data-active"), null);
  assert.equal(button.getAttribute("data-pending"), null);
  assert.equal(button.getAttribute("aria-pressed"), "false");
  assert.equal(button.getAttribute("aria-disabled"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.getAttribute("class"), null);
  assert.equal(button.getAttribute("style"), null);
  assert.equal(button.getAttribute("data-target"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.getAttribute("part"), "label");
  assert.equal(label.textContent, "Enter fullscreen");
  handle.unmount();
});

test("non-native hosts preserve keyboard button activation", async () => {
  const rig = createControllerRig();
  const handle = mountInteraction(FullscreenButton, {
    props: {
      as: "span",
      controller: rig.controller,
    },
    record: ["fullscreen"],
  });
  const button = handle.getByRole("button", { name: "Enter fullscreen" });

  assert.equal(button.tagName, "SPAN");
  assert.equal(button.getAttribute("role"), "button");
  assert.equal(button.getAttribute("tabindex"), "0");
  const enter = await handle.press(button, "Enter");
  await settle();
  const space = await handle.press(button, " ");
  await settle();

  assert.equal(enter.activated, false);
  assert.equal(space.keydownPrevented, true);
  assert.equal(rig.requests.length, 1);
  assert.equal(rig.exits.length, 1);
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, recordedOperation([emit], 0).type]),
    [
      ["fullscreen", "enter"],
      ["fullscreen", "exit"],
    ],
  );
  handle.unmount();
});

test("runs the injected controller for enter and exit", async () => {
  const rig = createControllerRig();
  const handle = mountInteraction(FullscreenButton, {
    props: { controller: rig.controller },
    record: ["fullscreen", "error"],
  });
  const button = handle.getByRole("button", { name: "Enter fullscreen" });

  await handle.click(button);
  await settle();

  assert.equal(rig.requests.length, 1);
  assert.ok(rig.requests[0]?.event instanceof MouseEvent);
  assert.ok(rig.requests[0]?.target === document.documentElement);
  assert.equal(button.getAttribute("data-state"), "active");
  assert.equal(button.getAttribute("data-active"), "true");
  assert.equal(button.getAttribute("aria-pressed"), "true");
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.textContent, "Exit fullscreen");
  const enterOperation = recordedOperation(handle.recorded(), 0);
  assert.equal(enterOperation.type, "enter");
  assert.ok(enterOperation.target === document.documentElement);
  assert.ok(enterOperation.controller === rig.controller);
  assert.ok(handle.recorded()[0]?.payload[1] instanceof MouseEvent);

  await handle.click(button);
  await settle();

  assert.equal(rig.exits.length, 1);
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-active"), null);
  assert.equal(button.getAttribute("aria-pressed"), "false");
  assert.equal(button.textContent, "Enter fullscreen");
  const exitOperation = recordedOperation(handle.recorded(), 1);
  assert.equal(exitOperation.type, "exit");
  assert.ok(exitOperation.target === document.documentElement);
  assert.ok(exitOperation.controller === rig.controller);
  assert.ok(handle.recorded()[1]?.payload[1] instanceof MouseEvent);
  assert.equal(handle.wrapper.emitted("error"), undefined);
  handle.unmount();
});

test("disabled fullscreen buttons suppress actions and keep availability hooks", async () => {
  const rig = createControllerRig();
  const native = mountInteraction(FullscreenButton, {
    props: { controller: rig.controller, disabled: true },
    record: ["fullscreen", "error"],
  });
  const nativeButton = native.getByRole("button", {
    name: "Enter fullscreen",
  }) as HTMLButtonElement;

  assert.equal(nativeButton.disabled, true);
  assert.equal(nativeButton.getAttribute("data-state"), "idle");
  assert.equal(nativeButton.getAttribute("data-disabled"), "true");
  assert.equal(nativeButton.getAttribute("aria-disabled"), null);
  await native.click(nativeButton);
  assert.equal(rig.requests.length, 0);
  assert.equal(rig.exits.length, 0);
  assert.deepEqual(native.recorded(), []);
  assert.equal(await native.tab(), null);
  native.unmount();

  const custom = mountInteraction(FullscreenButton, {
    props: { as: "span", controller: rig.controller, disabled: true },
    record: ["fullscreen", "error"],
  });
  const customButton = custom.getByRole("button", { name: "Enter fullscreen" });

  assert.equal(customButton.tagName, "SPAN");
  assert.equal(customButton.getAttribute("tabindex"), "-1");
  assert.equal(customButton.getAttribute("aria-disabled"), "true");
  await custom.click(customButton);
  await custom.press(customButton, "Enter");
  assert.equal(rig.requests.length, 0);
  assert.equal(rig.exits.length, 0);
  assert.deepEqual(custom.recorded(), []);
  custom.unmount();
});

test("suppresses duplicate operations while entering and exiting are in flight", async () => {
  let resolveEnter: (() => void) | null = null;
  let resolveExit: (() => void) | null = null;
  const rig = createControllerRig({
    request: () =>
      new Promise<void>((resolve) => {
        resolveEnter = resolve;
      }),
    exit: () =>
      new Promise<void>((resolve) => {
        resolveExit = resolve;
      }),
  });
  const handle = mountInteraction(FullscreenButton, {
    props: { controller: rig.controller },
    record: ["fullscreen", "error"],
  });
  const button = handle.getByRole("button", { name: "Enter fullscreen" });

  void handle.click(button);
  await settle();
  await handle.click(button);
  await settle();

  assert.equal(rig.requests.length, 1);
  assert.equal(button.getAttribute("data-state"), "entering");
  assert.equal(button.getAttribute("data-pending"), "true");
  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  assert.equal(button.getAttribute("aria-pressed"), "false");
  assert.equal(button.textContent, "Changing fullscreen");
  assert.deepEqual(handle.recorded(), []);

  assert.ok(resolveEnter);
  resolveEnter();
  await settle();
  assert.equal(button.getAttribute("data-state"), "active");
  assert.equal(button.getAttribute("data-pending"), null);
  assert.equal(handle.recorded().length, 1);

  void handle.click(button);
  await settle();
  await handle.click(button);
  await settle();

  assert.equal(rig.exits.length, 1);
  assert.equal(button.getAttribute("data-state"), "exiting");
  assert.equal(button.getAttribute("data-pending"), "true");
  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  assert.equal(button.getAttribute("aria-pressed"), "true");

  assert.ok(resolveExit);
  resolveExit();
  await settle();
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-pending"), null);
  assert.equal(handle.recorded().length, 2);
  handle.unmount();
});

test("captures fullscreen failures without throwing out of activation", async () => {
  const failure = new Error("fullscreen denied");
  const target = document.createElement("section");
  const controller: FullscreenButtonController = {
    getFullscreenElement: () => null,
    requestFullscreen: () => {
      throw failure;
    },
    exitFullscreen: () => {},
  };
  const handle = mountInteraction(FullscreenButton, {
    props: {
      controller,
      errorLabel: "Could not enter fullscreen",
      target,
    },
    record: ["fullscreen", "error"],
  });
  const button = handle.getByRole("button", { name: "Enter fullscreen" });

  await handle.click(button);
  await settle();

  assert.equal(button.getAttribute("data-state"), "error");
  assert.equal(button.getAttribute("data-active"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.getAttribute("aria-pressed"), "false");
  assert.equal(button.textContent, "Could not enter fullscreen");
  assert.equal(handle.wrapper.emitted("fullscreen"), undefined);
  assert.deepEqual(
    handle
      .recorded()
      .map((emit) => [
        emit.event,
        emit.payload[0],
        (emit.payload[1] as FullscreenButtonOperation).type,
        (emit.payload[1] as FullscreenButtonOperation).target === target,
        emit.payload[2] instanceof MouseEvent,
      ]),
    [["error", failure, "enter", true, true]],
  );
  handle.unmount();
});

test("uses the submitted controller when props change while entering", async () => {
  let resolveFirst: (() => void) | null = null;
  const firstTarget = document.createElement("section");
  const secondTarget = document.createElement("article");
  const first = createControllerRig({
    request: () =>
      new Promise<void>((resolve) => {
        resolveFirst = resolve;
      }),
  });
  const second = createControllerRig();
  const handle = mountInteraction(FullscreenButton, {
    props: {
      controller: first.controller,
      target: firstTarget,
    },
    record: ["fullscreen", "error"],
  });
  const button = handle.getByRole("button", { name: "Enter fullscreen" });

  void handle.click(button);
  await settle();
  await handle.wrapper.setProps({ controller: second.controller, target: secondTarget });

  assert.ok(resolveFirst);
  resolveFirst();
  await settle();

  assert.equal(first.requests.length, 1);
  assert.equal(second.requests.length, 0);
  assert.deepEqual(
    handle.recorded().map((emit) => emit.event),
    ["fullscreen"],
  );
  const operation = recordedOperation(handle.recorded(), 0);
  assert.equal(operation.type, "enter");
  assert.ok(operation.target === firstTarget);
  assert.ok(operation.controller === first.controller);
  assert.equal(button.getAttribute("data-state"), "active");
  handle.unmount();
});
