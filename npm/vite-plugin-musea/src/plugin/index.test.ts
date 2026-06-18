import assert from "node:assert/strict";
import test from "node:test";
import { shouldApplyMuseaPlugin } from "./apply.js";

void test("musea plugin is inactive in Vite test mode", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "serve", mode: "test" }), false);
});

void test("musea plugin remains active during Vite serve outside test mode", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "serve", mode: "development" }), true);
});

void test("musea plugin remains active during production builds", () => {
  assert.equal(shouldApplyMuseaPlugin({ command: "build", mode: "production" }), true);
});
