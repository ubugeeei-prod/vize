import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import { topPortalLayer, usePortalStack } from "./portal-stack.ts";
import Portal from "./portal.vue";

function nestedProbe(showInner: ReturnType<typeof ref<boolean>>) {
  return defineComponent({
    name: "NestedPortalProbe",
    setup() {
      return () =>
        h(
          Portal,
          { disabled: true },
          {
            default: () => [
              "Outer",
              showInner.value ? h(Portal, { disabled: true }, { default: () => "Inner" }) : null,
            ],
          },
        );
    },
  });
}

test("publishes incrementing depth for nested portals", async () => {
  const showInner = ref(true);
  const handle = mountInteraction(nestedProbe(showInner));
  try {
    await nextTick();
    const layers = [...handle.root().querySelectorAll("[data-vize-portal-depth]")];
    assert.deepEqual(
      layers.map((layer) => layer.getAttribute("data-vize-portal-depth")),
      ["0", "1"],
    );
  } finally {
    handle.unmount();
  }
});

test("tracks nested layers shallow-to-deep in the shared stack", async () => {
  const stack = usePortalStack();
  assert.equal(stack.value.length, 0);
  const showInner = ref(true);
  const handle = mountInteraction(nestedProbe(showInner));
  try {
    await nextTick();
    assert.deepEqual(
      stack.value.map((entry) => entry.depth),
      [0, 1],
    );
    const top = topPortalLayer();
    assert.equal(top?.depth, 1);
    assert.equal(top?.element.getAttribute("data-vize-portal-depth"), "1");
  } finally {
    handle.unmount();
  }
});

test("releases layers from the stack on unmount", async () => {
  const stack = usePortalStack();
  const showInner = ref(true);
  const handle = mountInteraction(nestedProbe(showInner));
  try {
    await nextTick();
    assert.equal(stack.value.length, 2);
    showInner.value = false;
    await nextTick();
    assert.deepEqual(
      stack.value.map((entry) => entry.depth),
      [0],
    );
    assert.equal(topPortalLayer()?.depth, 0);
  } finally {
    handle.unmount();
  }
  assert.equal(stack.value.length, 0);
  assert.equal(topPortalLayer(), null);
});
