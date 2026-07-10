import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";

import { resolveScanRoots } from "../utils.ts";
import { createMuseaWatchTargets, watchMuseaArtFiles } from "./watch.ts";

void test("musea dev watcher targets discovered art files instead of scan roots", () => {
  const root = "/workspace/apps/docs";
  const externalRoot = "/workspace/apps/design";
  const include = ["**/*.art.vue", "../design/**/*.art.vue"];
  const scanRoots = resolveScanRoots(root, include);
  const files = [
    path.join(root, "components", "Button.art.vue"),
    path.join(externalRoot, "Card.art.vue"),
  ];

  assert.deepEqual(scanRoots, [root, externalRoot]);
  const targets = createMuseaWatchTargets(files);
  assert.deepEqual(targets, files);
  assert.equal(targets.includes(root), false);
  assert.equal(targets.includes(externalRoot), false);
});

void test("musea dev watcher adds unique discovered files once", () => {
  const calls: string[][] = [];
  const file = "/workspace/apps/docs/components/Button.art.vue";

  watchMuseaArtFiles(
    {
      add(paths) {
        calls.push(Array.isArray(paths) ? paths : [paths]);
        return this;
      },
    },
    [file, file],
  );

  assert.deepEqual(calls, [[file]]);
});

void test("musea dev watcher skips empty file lists", () => {
  const calls: string[][] = [];

  watchMuseaArtFiles(
    {
      add(paths) {
        calls.push(Array.isArray(paths) ? paths : [paths]);
        return this;
      },
    },
    [],
  );

  assert.deepEqual(calls, []);
});
