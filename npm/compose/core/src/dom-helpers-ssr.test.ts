import assert from "node:assert/strict";
import { test } from "node:test";
import { createSSRApp, defineComponent, h, ref, useTemplateRef } from "vue";
import { renderToString } from "vue/server-renderer";

import { useAnimate } from "./animate.ts";
import { breakpointsTailwind, useBreakpoints } from "./breakpoints.ts";
import { useColorMode, useDark } from "./color-mode.ts";
import { useDraggable } from "./draggable.ts";
import { useElementByPoint } from "./element-by-point.ts";
import { useElementHover } from "./element-hover.ts";
import { useElementRef } from "./element-ref.ts";
import { useInfiniteScroll } from "./infinite-scroll.ts";
import { onClickOutside } from "./on-click-outside.ts";
import { onKeyStroke } from "./on-key-stroke.ts";
import { onLongPress } from "./on-long-press.ts";
import { onStartTyping } from "./on-start-typing.ts";
import { useParentElement } from "./parent-element.ts";
import { useScrollLock } from "./scroll-lock.ts";
import { useTextareaAutosize } from "./textarea-autosize.ts";
import { useTransition } from "./transition.ts";
import { useVirtualList } from "./virtual-list.ts";

const rows = Array.from({ length: 100 }, (_, index) => index);

const Probe = defineComponent({
  setup() {
    const box = useTemplateRef<HTMLElement>("box");
    const drag = useDraggable(box, { initialValue: { x: 4, y: 8 } });
    const hovered = useElementHover(box);
    const infinite = useInfiniteScroll(box, () => undefined);
    const virtual = useVirtualList(rows, { itemSize: 20, initialItemCount: 3 });
    const autosize = useTextareaAutosize(box, { input: ref("text") });
    const locked = useScrollLock(box, { initialValue: true });
    const parent = useParentElement(box);
    const point = useElementByPoint({ x: 0, y: 0 });
    const { element } = useElementRef();
    const animate = useAnimate(box, [{ opacity: 0 }, { opacity: 1 }], 200);
    const tween = useTransition(ref(42));
    const bp = useBreakpoints(breakpointsTailwind, { ssrWidth: 1024 });
    const color = useColorMode();
    const dark = useDark();
    onClickOutside(box, () => undefined);
    onKeyStroke("k", () => undefined);
    onLongPress(box, () => undefined);
    onStartTyping(() => undefined);

    return () =>
      h(
        "div",
        { ref: "box" },
        JSON.stringify({
          drag: [drag.x.value, drag.y.value, drag.isDragging.value, drag.style.value],
          hovered: hovered.value,
          loading: infinite.isLoading.value,
          virtual: virtual.list.value.map((row) => row.index),
          input: autosize.input.value,
          locked: locked.value,
          parent: parent.value,
          point: [point.isSupported.value, point.element.value],
          element: element.value,
          animate: [animate.isSupported.value, animate.playState.value],
          tween: tween.value,
          breakpoints: [bp.active.value, bp.current.value],
          color: [color.mode.value, color.state.value, dark.value],
        }),
      );
  },
});

void test("DOM helpers render deterministic server values without browser globals", async () => {
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
    drag: [4, 8, false, "left:4px;top:8px;"],
    hovered: false,
    loading: false,
    virtual: [0, 1, 2],
    input: "text",
    locked: true,
    parent: null,
    point: [false, null],
    element: null,
    animate: [false, "idle"],
    tween: 42,
    breakpoints: ["lg", ["sm", "md", "lg"]],
    color: ["auto", "light", false],
  });
});
