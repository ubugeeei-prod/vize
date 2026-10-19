import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import {
  collectReviewedTranslations,
  isReviewedTranslation,
} from "../../docs/scripts/i18n/reviewed.ts";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { updatePerformance } from "../../tools/benchmarks/scripts/published-snapshot-sections.mjs";

const root = resolve(import.meta.dirname, "../..");
const path = "architecture/performance.md";
const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
const benchmarks = /<!-- benchmark:(\w+):start -->[\s\S]*?<!-- benchmark:\1:end -->/g;

void test("Japanese performance review survives both translation and benchmark regeneration", async () => {
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese, "the reviewed article is a generation input");
  assert.equal(
    isReviewedTranslation(
      japanese.replace("<!-- Reviewed translation;", "<!-- Generated translation;"),
    ),
    false,
    "a generated translation is not protected from regeneration",
  );
  const data = JSON.parse(
    readFileSync(resolve(root, "tools/benchmarks/results/tool-benchmark-latest.json"), "utf8"),
  );
  assert.ok(japanese.match(benchmarks)?.length, "the article has benchmark blocks to publish");
  assert.equal(
    updatePerformance(japanese, data, "ja").replace(benchmarks, ""),
    japanese.replace(benchmarks, ""),
    "benchmark publication changes only its marked blocks",
  );
  assert.ok(
    defaultNavigationRenderRoutes.includes("/ja/architecture/performance"),
    "the article receives the desktop/mobile render checks",
  );
});
