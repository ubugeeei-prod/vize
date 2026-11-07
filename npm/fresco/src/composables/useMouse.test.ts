import assert from "node:assert/strict";
import { defineComponent } from "vue";
import { test } from "vitest";

import { lastMouseEvent } from "../app.js";
import { mountFresco } from "../testing/index.js";
import { useMouse } from "./useMouse.js";

test("delivers every mouse event when several arrive in one scheduler turn", () => {
  const kinds: string[] = [];
  const mounted = mountFresco(
    defineComponent({
      setup() {
        useMouse((event) => {
          if (event.kind) kinds.push(event.kind);
        });
        return () => null;
      },
    }),
  );

  lastMouseEvent.value = { type: "mouse", kind: "scroll-up", x: 1, y: 2 };
  lastMouseEvent.value = { type: "mouse", kind: "scroll-down", x: 1, y: 3 };

  assert.deepEqual(kinds, ["scroll-up", "scroll-down"]);
  mounted.unmount();
  lastMouseEvent.value = null;
});
