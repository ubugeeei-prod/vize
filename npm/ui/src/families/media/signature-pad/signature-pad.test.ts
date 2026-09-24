import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import type {
  SignaturePadCanvasExpose,
  SignaturePadRootExpose,
  SignaturePadSlotState,
  SignatureValue,
} from "./signature-pad.ts";
import SignaturePadCanvas from "./signature-pad-canvas.vue";
import SignaturePadClear from "./signature-pad-clear.vue";
import SignaturePadGuide from "./signature-pad-guide.vue";
import SignaturePadRedo from "./signature-pad-redo.vue";
import SignaturePadRoot from "./signature-pad-root.vue";
import SignaturePadUndo from "./signature-pad-undo.vue";
import { mountInteraction } from "../../../testing/mount.ts";

function mountPad(props: Record<string, unknown> = {}) {
  return mountInteraction(SignaturePadRoot, {
    props: { id: "sig", width: 200, height: 100, ...props },
    record: ["update:modelValue", "change", "strokeStart", "strokeEnd"],
    slots: {
      default: (state: SignaturePadSlotState) => [
        h("output", { "data-empty": String(state.empty) }, state.state),
        h(SignaturePadGuide, null, () => "Sign above"),
        h(SignaturePadCanvas, { ariaDescribedby: "sig-help" }),
        h(SignaturePadUndo, null, () => "Undo"),
        h(SignaturePadRedo, null, () => "Redo"),
        h(SignaturePadClear, null, () => "Clear"),
      ],
    },
  });
}

function canvasOf(root: HTMLElement): SVGSVGElement {
  const canvas = root.querySelector('[data-vize-ui="signature-pad-canvas"]');
  assert.ok(canvas instanceof SVGSVGElement);
  return canvas;
}

function stubRect(canvas: Element, width = 400, height = 200): void {
  Object.defineProperty(canvas, "getBoundingClientRect", {
    configurable: true,
    value: () => ({ left: 10, top: 20, width, height, right: 10 + width, bottom: 20 + height }),
  });
}

function pointer(type: string, init: PointerEventInit & { timeStamp?: number } = {}): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: true,
    cancelable: true,
    pointerId: 1,
    pointerType: "mouse",
    button: 0,
    pressure: 0.5,
    ...init,
  });
  if (init.timeStamp !== undefined) {
    Object.defineProperty(event, "timeStamp", { value: init.timeStamp });
  }
  return event;
}

async function drawStroke(
  canvas: SVGSVGElement,
  points: readonly (readonly [number, number])[],
  init: PointerEventInit = {},
): Promise<void> {
  const [first, ...rest] = points;
  assert.ok(first);
  canvas.dispatchEvent(
    pointer("pointerdown", { clientX: first[0], clientY: first[1], timeStamp: 100, ...init }),
  );
  rest.forEach(([x, y], index) =>
    canvas.dispatchEvent(
      pointer("pointermove", { clientX: x, clientY: y, timeStamp: 110 + index * 10, ...init }),
    ),
  );
  canvas.dispatchEvent(pointer("pointerup", init));
  await nextTick();
}

test("renders an accessible SVG surface with empty state hooks", () => {
  const handle = mountPad();
  const root = handle.root();
  const canvas = canvasOf(root);

  assert.equal(root.id, "sig");
  assert.equal(root.getAttribute("data-vize-ui"), "signature-pad-root");
  assert.equal(root.getAttribute("data-state"), "empty");
  assert.equal(canvas.getAttribute("role"), "img");
  assert.equal(canvas.getAttribute("aria-label"), "Signature");
  assert.equal(canvas.getAttribute("aria-describedby"), "sig-help");
  assert.equal(canvas.getAttribute("viewBox"), "0 0 200 100");
  assert.equal(canvas.style.touchAction, "none");
  assert.equal(
    root.querySelector('[data-vize-ui="signature-pad-guide"]')?.getAttribute("aria-hidden"),
    "true",
  );
  for (const name of ["Undo", "Redo", "Clear"]) {
    const button = handle.getByRole("button", { name }) as HTMLButtonElement;
    assert.equal(button.disabled, true);
    assert.equal(button.getAttribute("aria-controls"), "sig");
  }
  assert.equal(root.querySelector("input"), null, "no hidden input without a name");
  handle.unmount();
});

test("pointer strokes map to viewBox coordinates and commit on pointerup", async () => {
  const handle = mountPad();
  const root = handle.root();
  const canvas = canvasOf(root);
  stubRect(canvas);

  canvas.dispatchEvent(pointer("pointerdown", { clientX: 10, clientY: 20, timeStamp: 100 }));
  canvas.dispatchEvent(pointer("pointermove", { clientX: 210, clientY: 120, timeStamp: 120 }));
  await nextTick();
  assert.equal(root.getAttribute("data-drawing"), "true");
  assert.ok(root.querySelector('[data-vize-ui="signature-pad-live-stroke"]'));
  canvas.dispatchEvent(pointer("pointerup"));
  await nextTick();

  const value = handle.wrapper.emitted("update:modelValue")?.[0]?.[0] as SignatureValue;
  assert.equal(value.length, 1);
  const [start, end] = value[0]?.points ?? [];
  assert.deepEqual([start?.x, start?.y, start?.time], [0, 0, 0]);
  assert.deepEqual([end?.x, end?.y, end?.time], [100, 50, 20]);
  assert.equal(start?.pressure, 0.5, "mouse pressure is simulated from 0.5");
  assert.equal(root.getAttribute("data-state"), "filled");
  assert.equal(root.hasAttribute("data-drawing"), false);
  assert.equal(root.querySelectorAll('[data-vize-ui="signature-pad-stroke"]').length, 1);
  assert.equal(root.querySelector('[data-vize-ui="signature-pad-live-stroke"]'), null);
  assert.deepEqual(handle.wrapper.emitted("strokeStart")?.[0]?.[0], start);
  assert.deepEqual(handle.wrapper.emitted("strokeEnd")?.[0]?.[0], value[0]);
  assert.deepEqual(handle.wrapper.emitted("change")?.[0]?.slice(1), [[], "stroke"]);
  handle.unmount();
});

test("pen pressure, coalesced samples, and pressure modes shape the points", async () => {
  const handle = mountPad();
  const canvas = canvasOf(handle.root());
  const down = pointer("pointerdown", {
    pointerType: "pen",
    pressure: 0.9,
    clientX: 0,
    clientY: 0,
  });
  canvas.dispatchEvent(down);
  const move = pointer("pointermove", {
    pointerType: "pen",
    pressure: 0.2,
    clientX: 4,
    clientY: 0,
  });
  Object.defineProperty(move, "getCoalescedEvents", {
    value: () => [
      pointer("pointermove", { pointerType: "pen", pressure: 0.4, clientX: 2, clientY: 0 }),
      pointer("pointermove", { pointerType: "pen", pressure: 0.2, clientX: 4, clientY: 0 }),
    ],
  });
  canvas.dispatchEvent(move);
  canvas.dispatchEvent(pointer("pointerup", { pointerType: "pen" }));
  await nextTick();
  const value = handle.exposes<SignaturePadRootExpose>().value;
  assert.deepEqual(
    value[0]?.points.map((point) => [point.x, point.pressure]),
    [
      [0, 0.9],
      [2, 0.4],
      [4, 0.2],
    ],
  );
  handle.unmount();

  const forced = mountPad({ pressure: "pointer" });
  await drawStroke(
    canvasOf(forced.root()),
    [
      [0, 0],
      [5, 0],
    ],
    { pressure: 0.25 },
  );
  assert.deepEqual(
    forced.exposes<SignaturePadRootExpose>().value[0]?.points.map((point) => point.pressure),
    [0.25, 0.25],
  );
  forced.unmount();

  const simulated = mountPad({ pressure: "simulate" });
  await drawStroke(
    canvasOf(simulated.root()),
    [
      [0, 0],
      [1, 0],
    ],
    { pointerType: "pen", pressure: 1 },
  );
  assert.equal(simulated.exposes<SignaturePadRootExpose>().value[0]?.points[0]?.pressure, 0.5);
  simulated.unmount();
});

test("ignores secondary buttons, foreign pointers, and cancelled strokes", async () => {
  const handle = mountPad();
  const canvas = canvasOf(handle.root());
  canvas.dispatchEvent(pointer("pointerdown", { button: 2 }));
  assert.equal(handle.root().hasAttribute("data-drawing"), false);

  canvas.dispatchEvent(pointer("pointerdown", { clientX: 1, clientY: 1 }));
  canvas.dispatchEvent(pointer("pointerdown", { pointerId: 2, pointerType: "touch" }));
  canvas.dispatchEvent(pointer("pointermove", { pointerId: 2, clientX: 50, clientY: 50 }));
  canvas.dispatchEvent(pointer("pointerup", { pointerId: 2 }));
  await nextTick();
  assert.equal(handle.root().getAttribute("data-drawing"), "true");
  canvas.dispatchEvent(pointer("pointercancel"));
  await nextTick();
  assert.equal(handle.root().hasAttribute("data-drawing"), false);
  assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
  handle.unmount();
});

test("undo, redo, and clear walk history and honor preventDefault", async () => {
  const handle = mountPad();
  const canvas = canvasOf(handle.root());
  await drawStroke(canvas, [
    [0, 0],
    [10, 10],
  ]);
  await drawStroke(canvas, [
    [20, 20],
    [30, 30],
  ]);
  const exposed = handle.exposes<SignaturePadRootExpose>();
  assert.equal(exposed.value.length, 2);
  const undo = handle.getByRole("button", { name: "Undo" }) as HTMLButtonElement;
  const redo = handle.getByRole("button", { name: "Redo" }) as HTMLButtonElement;
  const clear = handle.getByRole("button", { name: "Clear" }) as HTMLButtonElement;
  assert.equal(undo.disabled, false);
  assert.equal(redo.disabled, true);

  await handle.click(undo);
  assert.equal(exposed.value.length, 1);
  assert.equal(redo.disabled, false);
  await handle.click(redo);
  assert.equal(exposed.value.length, 2);
  await handle.click(clear);
  assert.equal(exposed.value.length, 0);
  assert.equal(handle.root().getAttribute("data-state"), "empty");
  await handle.click(undo);
  assert.equal(exposed.value.length, 2, "clear is undoable");
  await drawStroke(canvas, [[5, 5]]);
  assert.equal(redo.disabled, true, "a new stroke drops the redo branch");
  assert.deepEqual(
    handle.wrapper.emitted("change")?.map((payload) => payload[2]),
    ["stroke", "stroke", "undo", "redo", "clear", "undo", "stroke"],
  );

  clear.addEventListener("click", (event) => event.preventDefault());
  const blocked = mountInteraction(SignaturePadRoot, {
    props: { defaultValue: exposed.value },
    slots: {
      default: () =>
        h(
          SignaturePadClear,
          { onClick: (event: MouseEvent) => event.preventDefault() },
          () => "Clear",
        ),
    },
  });
  blocked
    .root()
    .querySelector("button")
    ?.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
  await nextTick();
  assert.equal(blocked.exposes<SignaturePadRootExpose>().value.length, 3);
  blocked.unmount();
  handle.unmount();
});

test("history honors the configured limit", async () => {
  const handle = mountPad({ historyLimit: 1 });
  const canvas = canvasOf(handle.root());
  await drawStroke(canvas, [[0, 0]]);
  await drawStroke(canvas, [[5, 5]]);
  const exposed = handle.exposes<SignaturePadRootExpose>();
  assert.equal(exposed.undo(), true);
  assert.equal(exposed.undo(), false);
  assert.equal(exposed.value.length, 1);
  handle.unmount();

  const none = mountPad({ historyLimit: 0 });
  await drawStroke(canvasOf(none.root()), [[0, 0]]);
  assert.equal(none.exposes<SignaturePadRootExpose>().canUndo, false);
  none.unmount();
});

test("disabled and read-only pads never draw or edit", async () => {
  for (const props of [{ disabled: true }, { readOnly: true }]) {
    const handle = mountPad({
      ...props,
      defaultValue: [{ points: [{ x: 1, y: 1, pressure: 0.5, time: 0 }] }],
    });
    const root = handle.root();
    await drawStroke(canvasOf(root), [
      [0, 0],
      [10, 10],
    ]);
    assert.equal(handle.wrapper.emitted("update:modelValue"), undefined);
    assert.equal(
      (handle.getByRole("button", { name: "Clear" }) as HTMLButtonElement).disabled,
      true,
    );
    const exposed = handle.exposes<SignaturePadRootExpose>();
    assert.equal(exposed.clear(), false);
    assert.equal(exposed.undo(), false);
    assert.equal(exposed.redo(), false);
    assert.equal(root.getAttribute("data-state"), "filled");
    handle.unmount();
  }
  const disabled = mountPad({ disabled: true });
  assert.equal(disabled.root().getAttribute("data-disabled"), "true");
  assert.equal(canvasOf(disabled.root()).getAttribute("data-disabled"), "true");
  disabled.unmount();
  const readOnly = mountPad({ readOnly: true });
  assert.equal(readOnly.root().getAttribute("data-readonly"), "true");
  readOnly.unmount();
});

test("controlled strokes win until the parent accepts the request", async () => {
  const handle = mountPad({ modelValue: [] });
  await drawStroke(canvasOf(handle.root()), [
    [0, 0],
    [5, 5],
  ]);
  assert.equal(handle.wrapper.emitted("update:modelValue")?.length, 1);
  assert.equal(handle.root().getAttribute("data-state"), "empty");
  const accepted = handle.wrapper.emitted("update:modelValue")?.[0]?.[0];
  await handle.wrapper.setProps({ modelValue: accepted });
  assert.equal(handle.root().getAttribute("data-state"), "filled");
  handle.unmount();
});

test("submits the serialized value through a hidden input", async () => {
  const handle = mountPad({ name: "signature", form: "contract" });
  const input = handle
    .root()
    .querySelector<HTMLInputElement>('[data-vize-ui="signature-pad-input"]');
  assert.ok(input);
  assert.equal(input.type, "hidden");
  assert.equal(input.name, "signature");
  assert.equal(input.getAttribute("form"), "contract");
  assert.equal(input.value, "");
  await drawStroke(canvasOf(handle.root()), [[10, 20]]);
  assert.match(input.value, /^\[\{"points":\[{"x":10,"y":20,"pressure":0\.5,"time":0}\]\}\]$/);
  await handle.wrapper.setProps({ valueFormat: "svg" });
  assert.match(input.value, /^<svg /);
  await handle.wrapper.setProps({ disabled: true });
  assert.equal(input.disabled, true);
  handle.unmount();
});

test("exposes typed state, history controls, and exports", async () => {
  const handle = mountPad();
  const exposed = handle.exposes<SignaturePadRootExpose>();
  assert.equal(exposed.empty, true);
  assert.equal(exposed.state, "empty");
  assert.ok(exposed.element === handle.root());
  const value = [{ points: [{ x: 5, y: 5, pressure: 1, time: 0 }] }];
  assert.equal(exposed.setValue(value), true);
  assert.equal(exposed.setValue(exposed.value), false);
  assert.equal(exposed.canUndo, true);
  assert.match(exposed.toSvg({ color: "blue" }), /viewBox="0 0 200 100"[\s\S]*fill="blue"/);
  assert.match(await exposed.toDataUrl({ type: "image/svg+xml" }), /^data:image\/svg\+xml/);
  assert.equal(handle.wrapper.emitted("change")?.[0]?.[2], "api");
  const canvas = handle.wrapper.findComponent(SignaturePadCanvas)
    .vm as unknown as SignaturePadCanvasExpose;
  assert.equal(canvas.drawing, false);
  assert.ok(canvas.element instanceof SVGSVGElement);
  handle.unmount();
});

test("compound parts require a matching root provider", () => {
  for (const part of [
    SignaturePadCanvas,
    SignaturePadClear,
    SignaturePadUndo,
    SignaturePadRedo,
    SignaturePadGuide,
  ]) {
    assert.throws(
      () => mountInteraction(part),
      /VIZE_UI_CONTEXT_MISSING: SignaturePad requires a matching provider/,
    );
  }
});
