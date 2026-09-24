import assert from "node:assert/strict";
import { test } from "node:test";
import { createSSRApp, defineComponent, h, useTemplateRef } from "vue";
import { renderToString } from "vue/server-renderer";

import { useActiveElement } from "./active-element.ts";
import { useElementBounding } from "./element-bounding.ts";
import { useFocus, useFocusWithin } from "./focus.ts";
import { useElementVisibility, useIntersectionObserver } from "./intersection-observer.ts";
import { useKeyPressed, useMagicKeys } from "./magic-keys.ts";
import { useMouse } from "./mouse.ts";
import { useMouseInElement } from "./mouse-in-element.ts";
import { useMutationObserver } from "./mutation-observer.ts";
import { usePinch } from "./pinch.ts";
import { usePointer } from "./pointer.ts";
import { usePointerLock } from "./pointer-lock.ts";
import { useElementSize, useResizeObserver } from "./resize-observer.ts";
import { useScroll, useWindowScroll } from "./scroll.ts";
import { usePointerSwipe, useSwipe } from "./swipe.ts";
import { useTextSelection } from "./text-selection.ts";
import { useWindowSize } from "./window-size.ts";

const Probe = defineComponent({
  setup() {
    const box = useTemplateRef<HTMLElement>("box");
    const size = useElementSize(box, { initialSize: { width: 1, height: 2 } });
    const observer = useResizeObserver(box, () => undefined);
    const bounds = useElementBounding(box);
    const visible = useElementVisibility(box);
    const intersection = useIntersectionObserver(box, () => undefined);
    const mutation = useMutationObserver(box, () => undefined, { childList: true });
    const windowSize = useWindowSize({ initialWidth: 1280, initialHeight: 720 });
    const scroll = useScroll(box);
    const windowScroll = useWindowScroll();
    const mouse = useMouse();
    const inElement = useMouseInElement(box);
    const pointer = usePointer();
    const lock = usePointerLock(box);
    const swipe = useSwipe(box);
    const pointerSwipe = usePointerSwipe(box);
    const pinch = usePinch(box);
    const keys = useMagicKeys();
    const escape = useKeyPressed("escape");
    const focus = useFocus(box, { initialValue: true });
    const within = useFocusWithin(box);
    const active = useActiveElement();
    const selection = useTextSelection();

    return () =>
      h(
        "div",
        { ref: "box" },
        JSON.stringify({
          size: [size.width.value, size.height.value],
          supported: [
            observer.isSupported.value,
            intersection.isSupported.value,
            mutation.isSupported.value,
            windowSize.isSupported.value,
            lock.isSupported.value,
          ],
          bounds: [bounds.x.value, bounds.width.value],
          visible: visible.isVisible.value,
          window: [windowSize.width.value, windowSize.height.value],
          scroll: [scroll.x.value, scroll.arrivedState.top, windowScroll.arrivedState.bottom],
          mouse: [mouse.x.value, mouse.sourceType.value, inElement.isOutside.value],
          pointer: pointer.state.pointerType,
          swipe: [swipe.direction.value, pointerSwipe.isSwiping.value, pinch.scale.value],
          keys: [keys.current.size, keys.isPressed("ctrl+k").value, escape.value],
          focus: [focus.focused.value, within.focused.value, active.value],
          selection: selection.text.value,
        }),
      );
  },
});

void test("browser sensors render deterministic server values without browser globals", async () => {
  assert.equal(typeof globalThis.window, "undefined");
  const first = await renderToString(createSSRApp(Probe));
  const second = await renderToString(createSSRApp(Probe));

  assert.equal(first, second);
  const payload: unknown = JSON.parse(
    first
      .replace(/^<div>/, "")
      .replace(/<\/div>$/, "")
      .replaceAll("&quot;", '"'),
  );
  assert.deepEqual(payload, {
    size: [1, 2],
    supported: [false, false, false, false, false],
    bounds: [0, 0],
    visible: false,
    window: [1280, 720],
    scroll: [0, true, false],
    mouse: [0, null, true],
    pointer: null,
    swipe: ["none", false, 1],
    keys: [0, false, false],
    focus: [false, false, null],
    selection: "",
  });
});
