import assert from "node:assert/strict";
import test from "node:test";
import museaDefault, {
  MuseaA11yRunner,
  MuseaVrtRunner,
  UnsupportedDiffStyleError,
  musea,
} from "./index.js";
import { readMuseaOptions } from "./plugin/options.js";

void test("root entry preserves the synchronous plugin factory contract", () => {
  assert.equal(museaDefault, musea);
  const plugins = musea({ include: ["src/**/*.art.vue"] });
  assert.deepEqual(
    plugins.map((plugin) => plugin.name),
    ["vite-plugin-musea", "vite-plugin-musea:style-post-transform"],
  );
  assert.deepEqual(readMuseaOptions(plugins[0]), { include: ["src/**/*.art.vue"] });
  assert.equal(plugins[0]?.enforce, "pre");
  assert.equal(plugins[0]?.api?.vizePrecompileInputOwner.version, 1);
});

void test("root VRT facade preserves synchronous configuration access", () => {
  const runner = new MuseaVrtRunner({
    workers: 0,
    previewBasePath: "/gallery/",
    capture: { waitForPreviewReady: true },
  });
  assert.equal(runner.getOptions().workers, 1);
  assert.equal(runner.getCapture().waitForPreviewReady, true);
  assert.equal(
    runner.getPreviewUrl("https://example.test/", "src/Button.art.vue", "Primary"),
    "https://example.test/gallery/preview?art=src%2FButton.art.vue&variant=Primary",
  );
  assert.throws(
    () => new MuseaVrtRunner({ comparison: { diffStyle: "other" as "overlay" } }),
    UnsupportedDiffStyleError,
  );
});

void test("root accessibility facade retains reports without loading a browser", () => {
  const runner = new MuseaA11yRunner();
  const results = [
    {
      artPath: "src/Button.art.vue",
      variantName: "Primary",
      violations: [],
      passes: 2,
      incomplete: 0,
    },
  ];
  assert.deepEqual(runner.getSummary(results), {
    totalComponents: 1,
    totalVariants: 1,
    erroredVariants: 0,
    totalViolations: 0,
    criticalCount: 0,
    seriousCount: 0,
    moderateCount: 0,
    minorCount: 0,
  });
  assert.match(runner.generateHtmlReport(results), /No accessibility violations found/);
});
