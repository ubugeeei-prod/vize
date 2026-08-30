import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import PrintButton from "./print-button.vue";
import type { PrintButtonAction, PrintButtonExpose, PrintButtonSlotState } from "./print-button.ts";

async function settle(): Promise<void> {
  await Promise.resolve();
  await nextTick();
}

test("renders deterministic native button semantics and default label", () => {
  const handle = mountInteraction(PrintButton);
  const button = handle.getByRole("button", { name: "Print" }) as HTMLButtonElement;
  const label = button.querySelector('[data-vize-ui="print-button-label"]');

  assert.equal(button.tagName, "BUTTON");
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("data-vize-ui"), "print-button");
  assert.equal(button.getAttribute("part"), "root");
  assert.equal(button.getAttribute("data-state"), "idle");
  assert.equal(button.getAttribute("data-disabled"), null);
  assert.equal(button.getAttribute("data-printing"), null);
  assert.equal(button.getAttribute("aria-disabled"), null);
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.getAttribute("class"), null);
  assert.equal(button.getAttribute("style"), null);
  assert.equal(button.getAttribute("data-action"), null);
  assert.ok(label instanceof HTMLSpanElement);
  assert.equal(label.getAttribute("part"), "label");
  assert.equal(label.textContent, "Print");
  handle.unmount();
});

test("uses the platform print function by default", async () => {
  let calls = 0;
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "print");
  Object.defineProperty(globalThis, "print", {
    configurable: true,
    value: () => {
      calls += 1;
    },
  });
  const handle = mountInteraction(PrintButton, {
    record: ["print", "error"],
  });

  try {
    await handle.click(handle.getByRole("button", { name: "Print" }));
    await settle();

    assert.equal(calls, 1);
    assert.deepEqual(
      handle.recorded().map((emit) => emit.event),
      ["print"],
    );
    assert.equal(handle.root().getAttribute("data-state"), "printed");
  } finally {
    handle.unmount();
    if (descriptor === undefined) {
      delete (globalThis as { print?: unknown }).print;
    } else {
      Object.defineProperty(globalThis, "print", descriptor);
    }
  }
});

test("runs the configured action and exposes printed state", async () => {
  const events: MouseEvent[] = [];
  const action: PrintButtonAction = async (event) => {
    events.push(event);
  };
  const handle = mountInteraction(PrintButton, {
    props: { action },
    record: ["print", "error"],
  });
  const button = handle.getByRole("button", { name: "Print" });

  await handle.click(button);
  await settle();

  assert.equal(events.length, 1);
  assert.ok(events[0] instanceof MouseEvent);
  assert.equal(button.getAttribute("data-state"), "printed");
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.textContent, "Printed");
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0] instanceof MouseEvent]),
    [["print", true]],
  );
  assert.equal(handle.wrapper.emitted("error"), undefined);
  handle.unmount();
});

test("captures action failures without throwing out of activation", async () => {
  const failure = new Error("print denied");
  const handle = mountInteraction(PrintButton, {
    props: {
      errorLabel: "Could not print",
      action: async () => {
        throw failure;
      },
    },
    record: ["print", "error"],
  });
  const button = handle.getByRole("button", { name: "Print" });

  await handle.click(button);
  await settle();

  assert.equal(button.getAttribute("data-state"), "error");
  assert.equal(button.getAttribute("aria-busy"), null);
  assert.equal(button.textContent, "Could not print");
  assert.equal(handle.wrapper.emitted("print"), undefined);
  assert.deepEqual(
    handle
      .recorded()
      .map((emit) => [emit.event, emit.payload[0], emit.payload[1] instanceof MouseEvent]),
    [["error", failure, true]],
  );
  handle.unmount();
});

test("disabled print buttons suppress action and keep platform semantics", async () => {
  let calls = 0;
  const action: PrintButtonAction = () => {
    calls += 1;
  };
  const native = mountInteraction(PrintButton, {
    props: { disabled: true, action },
    record: ["print", "error"],
  });
  const nativeButton = native.getByRole("button", { name: "Print" }) as HTMLButtonElement;

  assert.equal(nativeButton.disabled, true);
  assert.equal(nativeButton.getAttribute("data-state"), "idle");
  assert.equal(nativeButton.getAttribute("data-disabled"), "true");
  assert.equal(nativeButton.getAttribute("aria-disabled"), null);
  await native.click(nativeButton);
  assert.equal(calls, 0);
  assert.deepEqual(native.recorded(), []);
  assert.equal(await native.tab(), null);
  native.unmount();

  const custom = mountInteraction(PrintButton, {
    props: { as: "span", disabled: true, action },
    record: ["print", "error"],
  });
  const customButton = custom.getByRole("button", { name: "Print" });

  assert.equal(customButton.tagName, "SPAN");
  assert.equal(customButton.getAttribute("tabindex"), "-1");
  assert.equal(customButton.getAttribute("aria-disabled"), "true");
  await custom.click(customButton);
  await custom.press(customButton, "Enter");
  assert.equal(calls, 0);
  assert.deepEqual(custom.recorded(), []);
  custom.unmount();
});

test("supports custom labels and slot rendering", async () => {
  const handle = mountInteraction(PrintButton, {
    props: {
      idleLabel: "Print report",
      printingLabel: "Printing report",
      printedLabel: "Report sent to print",
      action: () => {},
    },
    slots: {
      default: (state: PrintButtonSlotState) =>
        h("span", { "data-slot-state": state.state }, `${state.label}:${state.printing}`),
    },
  });
  const button = handle.getByRole("button", { name: "Print report:false" });

  assert.equal(button.textContent, "Print report:false");
  assert.equal(button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"), "idle");
  await handle.click(button);
  await settle();
  assert.equal(button.getAttribute("data-state"), "printed");
  assert.equal(button.textContent, "Report sent to print:false");
  assert.equal(
    button.querySelector("[data-slot-state]")?.getAttribute("data-slot-state"),
    "printed",
  );
  handle.unmount();
});

test("suppresses duplicate actions while printing is in flight", async () => {
  let calls = 0;
  let resolvePrint: (() => void) | null = null;
  const action: PrintButtonAction = () => {
    calls += 1;
    return new Promise<void>((resolve) => {
      resolvePrint = resolve;
    });
  };
  const handle = mountInteraction(PrintButton, {
    props: { action },
    record: ["print", "error"],
  });
  const button = handle.getByRole("button", { name: "Print" });

  void handle.click(button);
  await settle();
  await handle.click(button);
  await settle();

  assert.equal(calls, 1);
  assert.equal(button.getAttribute("data-state"), "printing");
  assert.equal(button.getAttribute("data-printing"), "true");
  assert.equal(button.getAttribute("aria-busy"), "true");
  assert.equal(button.getAttribute("aria-disabled"), "true");
  assert.equal(button.textContent, "Printing");
  assert.deepEqual(handle.recorded(), []);

  assert.ok(resolvePrint);
  resolvePrint();
  await settle();
  assert.equal(button.getAttribute("data-state"), "printed");
  assert.equal(button.getAttribute("data-printing"), null);
  assert.equal(handle.recorded().length, 1);

  await handle.click(button);
  await settle();
  assert.equal(calls, 2);
  handle.unmount();
});

test("uses the submitted action when props change while printing", async () => {
  const calls: string[] = [];
  let resolvePrint: (() => void) | null = null;
  const firstAction: PrintButtonAction = () => {
    calls.push("first");
    return new Promise<void>((resolve) => {
      resolvePrint = resolve;
    });
  };
  const secondAction: PrintButtonAction = () => {
    calls.push("second");
  };
  const handle = mountInteraction(PrintButton, {
    props: { action: firstAction },
    record: ["print", "error"],
  });
  const button = handle.getByRole("button", { name: "Print" });

  void handle.click(button);
  await settle();
  await handle.wrapper.setProps({ action: secondAction });

  assert.ok(resolvePrint);
  resolvePrint();
  await settle();

  assert.deepEqual(calls, ["first"]);
  assert.deepEqual(
    handle.recorded().map((emit) => emit.event),
    ["print"],
  );
  assert.equal(button.getAttribute("data-state"), "printed");
  handle.unmount();
});

test("non-native hosts preserve keyboard button activation", async () => {
  let calls = 0;
  const handle = mountInteraction(PrintButton, {
    props: {
      as: "span",
      action: () => {
        calls += 1;
      },
    },
    record: ["print"],
  });
  const button = handle.getByRole("button", { name: "Print" });

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

test("exposes live state and focus", async () => {
  const handle = mountInteraction(PrintButton, {
    props: {
      ariaLabel: "Print invoice",
      printedLabel: "Invoice printed",
      action: () => {},
    },
  });
  const exposed = handle.exposes<PrintButtonExpose>();
  const button = handle.getByRole("button", { name: "Print invoice" });

  assert.ok(exposed.element === button);
  assert.equal(exposed.disabled, false);
  assert.equal(exposed.printing, false);
  assert.equal(exposed.unavailable, false);
  assert.equal(exposed.state, "idle");
  assert.equal(exposed.label, "Print");
  exposed.focus();
  assert.ok(handle.activeElement() === button);

  await handle.click(button);
  await settle();
  assert.equal(exposed.state, "printed");
  assert.equal(exposed.printing, false);
  assert.equal(exposed.label, "Invoice printed");
  handle.unmount();
});
