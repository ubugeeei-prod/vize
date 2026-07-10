import assert from "node:assert/strict";
import test from "node:test";

import { sortedArts } from "./art-order.js";
import type { ArtFileInfo } from "./types/index.js";

function art(title: string, order?: number): ArtFileInfo {
  return {
    path: `/repo/${title}.art.vue`,
    metadata: { title, tags: [], status: "ready", order },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}

void test("sortedArts orders by metadata order before title", () => {
  const arts = sortedArts([art("Tertiary", 30), art("Neutral"), art("Primary", 10), art("Alpha")]);

  assert.deepEqual(
    arts.map((item) => item.metadata.title),
    ["Primary", "Tertiary", "Alpha", "Neutral"],
  );
});
