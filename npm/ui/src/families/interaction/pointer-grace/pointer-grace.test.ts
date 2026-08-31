import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope } from "vue";

import {
  createPointerGrace,
  gracePolygon,
  pointInGraceArea,
  usePointerGrace,
} from "./pointer-grace.ts";

const target = { height: 80, width: 120, x: 200, y: 40 };
const origin = { x: 40, y: 80 };

test("builds a polygon from the origin and extreme target corners", () => {
  const polygon = gracePolygon(origin, target);
  assert.equal(polygon.length, 3);
  assert.deepEqual(polygon[0], origin);
  assert.equal(
    pointInGraceArea({ x: 210, y: 50 }, origin, target),
    true,
    "points inside the target remain in the grace area",
  );
});

test("clears a pending timer when the pointer re-enters the target", async () => {
  let ended = 0;
  const grace = createPointerGrace({ delay: 20, onGraceEnd: () => ended++ });
  grace.setOrigin(origin);
  grace.setTarget(target);
  grace.handleMove({ x: 0, y: 0 });
  assert.equal(grace.isPending.value, true);
  grace.handleMove({ x: 210, y: 50 });
  assert.equal(grace.isPending.value, false);
  await new Promise((resolve) => setTimeout(resolve, 30));
  assert.equal(ended, 0);
  grace.dispose();
});

test("clears a pending timer when the pointer stays in the safe triangle", async () => {
  let ended = 0;
  const grace = createPointerGrace({ delay: 20, onGraceEnd: () => ended++ });
  grace.setOrigin(origin);
  grace.setTarget(target);
  const midpoint = {
    x: (origin.x + target.x) / 2,
    y: (origin.y + target.y + target.height / 2) / 2,
  };
  grace.handleMove({ x: 0, y: 0 });
  assert.equal(grace.isPending.value, true);
  grace.handleMove(midpoint);
  assert.equal(grace.contains(midpoint), true);
  await new Promise((resolve) => setTimeout(resolve, 30));
  assert.equal(ended, 0);
  grace.dispose();
});

test("ends grace after the pointer leaves the polygon", async () => {
  let ended = 0;
  const grace = createPointerGrace({ delay: 15, onGraceEnd: () => ended++ });
  grace.setOrigin(origin);
  grace.setTarget(target);
  grace.handleMove({ x: 0, y: 0 });
  await new Promise((resolve) => setTimeout(resolve, 30));
  assert.equal(ended, 1);
  grace.dispose();
});

test("rejects composable use outside an effect scope", () => {
  assert.throws(() => usePointerGrace(), /VIZE_UI_POINTER_GRACE_SETUP/);
});

test("disposes with the current effect scope", () => {
  const scope = effectScope();
  const controller = scope.run(() => usePointerGrace({ delay: 50 }));
  controller?.setTarget(target);
  controller?.handleMove({ x: 0, y: 0 });
  scope.stop();
  assert.throws(() => controller?.handleMove({ x: 1, y: 1 }), /VIZE_UI_POINTER_GRACE_DISPOSED/);
});
