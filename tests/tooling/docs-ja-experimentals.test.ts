import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

void test("translation regeneration keeps the reviewed Japanese experimentals pages", async () => {
  const japanese = resolve(import.meta.dirname, "../../docs/content/ja");
  const paths = ["guide/experimentals.md", "guide/experimentals-vue-rfcs.md"];
  const retained = await collectReviewedTranslations(paths, japanese);
  for (const path of paths)
    assert.equal(retained.get(path), readFileSync(resolve(japanese, path), "utf8"), path);
});
