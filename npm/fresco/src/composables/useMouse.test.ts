import assert from "node:assert/strict";
import { computed, defineComponent, ref } from "vue";
import { test } from "vitest";

import { lastMouseEvent } from "../app.js";
import { mountFresco } from "../testing/index.js";
import { useMouse } from "./useMouse.js";
import type { UseMouseOptions, UseMouseReturn } from "./useMouse.js";

function mountMouse(handler: Parameters<typeof useMouse>[0], options: UseMouseOptions = {}) {
  let controls: UseMouseReturn | undefined;
  const mounted = mountFresco(
    defineComponent({
      setup() {
        controls = useMouse(handler, options);
        return () => null;
      },
    }),
  );
  if (!controls) throw new Error("useMouse did not initialize");
  return { controls, mounted };
}

test("delivers every mouse event when several arrive in one scheduler turn", () => {
  const kinds: string[] = [];
  const { mounted } = mountMouse((event) => {
    if (event.kind) kinds.push(event.kind);
  });

  lastMouseEvent.value = { type: "mouse", kind: "scroll-up", x: 1, y: 2 };
  lastMouseEvent.value = { type: "mouse", kind: "scroll-down", x: 1, y: 3 };

  assert.deepEqual(kinds, ["scroll-up", "scroll-down"]);
  mounted.unmount();
  lastMouseEvent.value = null;
});

test("keeps getter-only activation separate from imperative enablement", () => {
  const externallyActive = ref(true);
  const kinds: string[] = [];
  const { controls, mounted } = mountMouse(
    (event) => {
      if (event.kind) kinds.push(event.kind);
    },
    { isActive: computed(() => externallyActive.value) },
  );

  controls.disable();
  lastMouseEvent.value = { type: "mouse", kind: "scroll-up", x: 1, y: 2 };
  assert.deepEqual(kinds, []);

  controls.enable();
  lastMouseEvent.value = { type: "mouse", kind: "scroll-down", x: 1, y: 3 };
  assert.deepEqual(kinds, ["scroll-down"]);

  externallyActive.value = false;
  controls.enable();
  lastMouseEvent.value = { type: "mouse", kind: "scroll-up", x: 1, y: 4 };
  assert.deepEqual(kinds, ["scroll-down"], "enable does not override the external source");

  externallyActive.value = true;
  lastMouseEvent.value = { type: "mouse", kind: "scroll-up", x: 1, y: 5 };
  assert.deepEqual(kinds, ["scroll-down", "scroll-up"]);
  mounted.unmount();
  lastMouseEvent.value = null;
});

test("can stop an unscoped mouse subscription", () => {
  const kinds: string[] = [];
  const controls = useMouse((event) => {
    if (event.kind) kinds.push(event.kind);
  });

  lastMouseEvent.value = { type: "mouse", kind: "scroll-up", x: 1, y: 2 };
  controls.stop();
  lastMouseEvent.value = { type: "mouse", kind: "scroll-down", x: 1, y: 3 };

  assert.deepEqual(kinds, ["scroll-up"]);
  lastMouseEvent.value = null;
});
