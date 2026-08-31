import assert from "node:assert/strict";

import { effectScope, ref } from "vue";
import { test } from "vite-plus/test";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { createSpatialNavigation, useSpatialNavigation } from "./spatial-navigation.ts";
import type { SpatialNavigationChange } from "./spatial-navigation.ts";
import { keyboard, mountSpatial, rect } from "./spatial-navigation-test-utils.ts";

test("moves physically in four directions and skips disabled candidates", () => {
  const changes: SpatialNavigationChange<string>[] = [];
  const harness = mountSpatial({ onNavigate: (change) => changes.push(change) });
  harness.registry.setActiveKey("alpha");
  assert.equal(harness.controller.findTarget("right"), "bravo");
  assert.equal(harness.controller.navigate("right"), "bravo");
  assert.equal(document.activeElement, harness.elements.get("bravo"));
  assert.equal(harness.controller.navigate("down"), "delta");
  assert.equal(harness.controller.navigate("left"), "charlie");
  assert.equal(harness.controller.navigate("up"), "alpha");
  assert.equal(harness.controller.navigate("right"), "bravo");
  assert.equal(harness.controller.navigate("right"), null);
  assert.deepEqual(
    changes.map(({ key, previousKey, direction }) => ({ key, previousKey, direction })),
    [
      { key: "bravo", previousKey: "alpha", direction: "right" },
      { key: "delta", previousKey: "bravo", direction: "down" },
      { key: "charlie", previousKey: "delta", direction: "left" },
      { key: "alpha", previousKey: "charlie", direction: "up" },
      { key: "bravo", previousKey: "alpha", direction: "right" },
    ],
  );
  assert.ok(changes.every(Object.isFrozen));
  harness.unmount();
});

test("normal and grid algorithms follow distinct CSS spatial ranking priorities", () => {
  const algorithm = ref<"grid" | "normal">("normal");
  const harness = mountSpatial({ algorithm });
  harness.registry.setActiveKey("alpha");
  harness.rects.set("bravo", rect(101, 110, 10, 10));
  harness.rects.set("charlie", rect(0, 200));
  harness.rects.set("delta", rect(500, 500));
  assert.equal(harness.controller.findTarget("down"), "bravo");
  algorithm.value = "grid";
  assert.equal(harness.controller.findTarget("down"), "charlie");
  harness.unmount();
});

test("registration order is the deterministic tie break", () => {
  const harness = mountSpatial();
  harness.registry.setActiveKey("alpha");
  harness.rects.set("bravo", rect(120, 0));
  harness.rects.set("charlie", rect(120, 0));
  assert.equal(harness.controller.findTarget("right"), "bravo");
  harness.unmount();
});

test("looping wraps across the opposite aligned spatial edge", () => {
  const loop = ref(false);
  const harness = mountSpatial({ loop });
  harness.registry.setActiveKey("alpha");
  assert.equal(harness.controller.navigate("left"), null);
  loop.value = true;
  assert.equal(harness.controller.navigate("left"), "bravo");
  assert.equal(harness.controller.navigate("up"), "delta");
  harness.unmount();
});

test("keyboard boundary ownership is configurable and retains native events", () => {
  const boundaryBehavior = ref<"contain" | "exit">("contain");
  const boundaries: unknown[] = [];
  const changes: SpatialNavigationChange<string>[] = [];
  const harness = mountSpatial({
    boundaryBehavior,
    onBoundary: (boundary) => boundaries.push(boundary),
    onNavigate: (change) => changes.push(change),
  });
  harness.registry.setActiveKey("alpha");
  const success = keyboard("ArrowRight", harness.container);
  assert.equal(success.defaultPrevented, true);
  assert.equal(changes[0]?.originalEvent, success);
  const contained = keyboard("ArrowRight", harness.container);
  assert.equal(contained.defaultPrevented, true);
  assert.ok(Object.isFrozen(boundaries[0]));
  boundaryBehavior.value = "exit";
  const exited = keyboard("ArrowRight", harness.container);
  assert.equal(exited.defaultPrevented, false);
  assert.equal(boundaries.length, 2);
  harness.unmount();
});

test("logical focus supports virtual geometry and custom reveal behavior", () => {
  const registry = createCollectionRegistry<string, string>();
  registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha" });
  registry.register({ key: "bravo", value: "Bravo", textValue: "Bravo" });
  registry.setActiveKey("alpha");
  const revealed: string[] = [];
  const controller = createSpatialNavigation({
    registry,
    focusBehavior: "logical",
    getRect: ({ key }) => (key === "alpha" ? rect(0, 0) : rect(120, 0)),
    scrollIntoView: ({ key }) => revealed.push(key),
  });
  assert.equal(controller.navigate("right"), "bravo");
  assert.deepEqual(revealed, ["bravo"]);
  controller.dispose();
  registry.dispose();
});

test("reactive disablement blocks input without discarding active state", () => {
  const disabled = ref(false);
  const harness = mountSpatial({ isDisabled: disabled });
  harness.registry.setActiveKey("alpha");
  disabled.value = true;
  assert.equal(harness.controller.findTarget("right"), null);
  assert.equal(harness.controller.navigate("right"), null);
  assert.equal(keyboard("ArrowRight", harness.container).defaultPrevented, false);
  assert.equal(harness.registry.activeKey.value, "alpha");
  harness.unmount();
});

test("rejects malformed options, geometry, origins, and commands", () => {
  const registry = createCollectionRegistry<string, string>();
  registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha" });
  assert.throws(() => createSpatialNavigation(null as never), /options must be an object/);
  assert.throws(
    () => createSpatialNavigation({ registry, algorithm: "nearest" } as never),
    /VIZE_UI_SPATIAL_NAVIGATION_OPTION.*algorithm/,
  );
  const controller = createSpatialNavigation({
    registry,
    getRect: () => ({ ...rect(0, 0), left: Number.NaN }),
  });
  assert.throws(() => controller.findTarget("right"), /VIZE_UI_SPATIAL_NAVIGATION_RECT/);
  assert.throws(() => controller.findTarget("diagonal" as never), /direction is invalid/);
  assert.throws(
    () => controller.findTarget("right", "missing"),
    /VIZE_UI_SPATIAL_NAVIGATION_ORIGIN/,
  );
  controller.dispose();
  registry.dispose();
});

test("disposal is idempotent, scope-owned, and leaves registry ownership alone", () => {
  const registry = createCollectionRegistry<string, string>();
  registry.register({ key: "alpha", value: "Alpha", textValue: "Alpha", element: null });
  const controller = createSpatialNavigation({ registry, getRect: () => rect(0, 0) });
  controller.dispose();
  controller.dispose();
  assert.throws(() => controller.findTarget("right"), /VIZE_UI_SPATIAL_NAVIGATION_DISPOSED/);
  assert.doesNotThrow(() => registry.setActiveKey("alpha"));
  assert.throws(() => useSpatialNavigation({ registry }), /VIZE_UI_SPATIAL_NAVIGATION_SETUP/);

  const scope = effectScope();
  let scoped!: ReturnType<typeof useSpatialNavigation<string, string>>;
  scope.run(() => {
    scoped = useSpatialNavigation({ registry, getRect: () => rect(0, 0) });
  });
  scope.stop();
  assert.throws(() => scoped.navigate("right"), /VIZE_UI_SPATIAL_NAVIGATION_DISPOSED/);
  registry.dispose();
});
