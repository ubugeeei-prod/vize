import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { shallowRef } from "vue";

import {
  enabledTourSteps,
  findTourStep,
  isTourEditableTarget,
  padTourRect,
  resolveTourTarget,
  tourAlignFromPlacement,
  tourSideFromPlacement,
} from "./tour-state.ts";

test("resolves selector, element, ref, and getter targets without throwing", () => {
  const element = document.createElement("section");
  element.id = "tour-state-target";
  document.body.append(element);
  try {
    assert.equal(resolveTourTarget("#tour-state-target", document), element);
    assert.equal(resolveTourTarget("#absent", document), null);
    assert.equal(resolveTourTarget("::invalid(", document), null);
    assert.equal(resolveTourTarget("   ", document), null);
    assert.equal(resolveTourTarget("#tour-state-target", null), null);
    assert.equal(resolveTourTarget(element, null), element);
    assert.equal(resolveTourTarget(shallowRef(element), null), element);
    assert.equal(
      resolveTourTarget(() => element, null),
      element,
    );
    assert.equal(
      resolveTourTarget(() => null, document),
      null,
    );
    assert.equal(resolveTourTarget(undefined, document), null);
  } finally {
    element.remove();
  }
});

test("walks enabled steps in one direction without wrapping", () => {
  const steps = enabledTourSteps([
    { value: "a" },
    { value: "b", disabled: true },
    { value: "c" },
    { value: "d" },
  ]);
  assert.deepEqual(
    steps.map((step) => step.value),
    ["a", "c", "d"],
  );
  assert.equal(findTourStep(steps, 1, 1, () => true)?.value, "c");
  assert.equal(findTourStep(steps, 1, 1, (step) => step.value !== "c")?.value, "d");
  assert.equal(findTourStep(steps, 2, -1, (step) => step.value === "a")?.value, "a");
  assert.equal(
    findTourStep(steps, 3, 1, () => true),
    null,
  );
  assert.equal(
    findTourStep(steps, -1, -1, () => true),
    null,
  );
});

test("pads spotlight boxes and clamps negative sizes", () => {
  assert.deepEqual(padTourRect({ x: 10, y: 20, width: 30, height: 40 }, 5), {
    x: 5,
    y: 15,
    width: 40,
    height: 50,
  });
  assert.deepEqual(padTourRect({ x: 0, y: 0, width: 4, height: 4 }, -5), {
    x: 5,
    y: 5,
    width: 0,
    height: 0,
  });
  assert.deepEqual(padTourRect({ x: 1, y: 2, width: 3, height: 4 }, Number.NaN), {
    x: 1,
    y: 2,
    width: 3,
    height: 4,
  });
});

test("maps placements, including the centered fallback, to side and alignment tokens", () => {
  assert.equal(tourSideFromPlacement("center"), "center");
  assert.equal(tourAlignFromPlacement("center"), "center");
  assert.equal(tourSideFromPlacement("top-end"), "top");
  assert.equal(tourAlignFromPlacement("top-end"), "end");
  assert.equal(tourSideFromPlacement("left"), "left");
  assert.equal(tourAlignFromPlacement("right-start"), "start");
  assert.equal(tourAlignFromPlacement("bottom"), "center");
});

test("detects editable keyboard targets", () => {
  const editable = document.createElement("div");
  editable.contentEditable = "true";
  assert.equal(isTourEditableTarget(document.createElement("input")), true);
  assert.equal(isTourEditableTarget(document.createElement("textarea")), true);
  assert.equal(isTourEditableTarget(document.createElement("select")), true);
  assert.equal(isTourEditableTarget(document.createElement("button")), false);
  assert.equal(isTourEditableTarget(null), false);
  document.body.append(editable);
  try {
    assert.equal(isTourEditableTarget(editable), editable.isContentEditable);
  } finally {
    editable.remove();
  }
});
