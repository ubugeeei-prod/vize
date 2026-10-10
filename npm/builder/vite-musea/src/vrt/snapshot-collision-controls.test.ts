import assert from "node:assert/strict";
import test from "node:test";
import { assertUniqueSnapshotNames } from "./snapshot-collisions.ts";
import type { ArtFileInfo } from "../types/index.ts";

const art = (file: string): ArtFileInfo => ({
  path: file,
  metadata: { title: file, tags: [], status: "ready" },
  variants: [],
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
});
const job = (file: string, variantName = "Default", name = "small", width = 200) => ({
  art: art(file),
  variantName,
  viewport: { width, height: 100, name },
});

void test("ordinary Art, variant and viewport names retain their legacy independent outputs", () => {
  assert.doesNotThrow(() => assertUniqueSnapshotNames([]));
  assert.doesNotThrow(() =>
    assertUniqueSnapshotNames([
      job("left/Button.art.vue"),
      job("right/Badge.art.vue"),
      job("left/Button.art.vue", "Secondary"),
      job("left/Button.art.vue", "Default", "wide", 400),
    ]),
  );
});

void test("duplicate viewport names and duplicated capture jobs are rejected before storage", () => {
  assert.throws(
    () =>
      assertUniqueSnapshotNames([
        job("Button.art.vue"),
        job("Button.art.vue", "Default", "small", 400),
      ]),
    /Snapshot name collision.*200x100.*400x100/,
  );
  assert.throws(
    () => assertUniqueSnapshotNames([job("Button.art.vue"), job("Button.art.vue")]),
    /Snapshot name collision/,
  );
});
