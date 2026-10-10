import assert from "node:assert/strict";
import test from "node:test";
import { buildVariantUrl } from "./utils.ts";
import { MuseaVrtRunner } from "./runner.ts";
import { parseArgs } from "../cli/index.ts";
import { createVrtOptions } from "../cli/commands.ts";

void test("configured routes retain legacy query encoding and normalize origin and route slashes", () => {
  const suffix = "preview?art=src%2FA%20B.art.vue&variant=Default%20%2F%20wide";
  assert.equal(
    buildVariantUrl("http://localhost:5173", "src/A B.art.vue", "Default / wide"),
    `http://localhost:5173/__musea__/${suffix}`,
  );
  assert.equal(
    buildVariantUrl(
      "http://localhost:5173/",
      "src/A B.art.vue",
      "Default / wide",
      "/site/components/",
    ),
    `http://localhost:5173/site/components/${suffix}`,
  );
  assert.equal(
    buildVariantUrl("http://localhost:5173", "src/A B.art.vue", "Default / wide", "/"),
    `http://localhost:5173/${suffix}`,
  );
  const runner = new MuseaVrtRunner({ previewBasePath: "/site/components" });
  assert.equal(
    runner.getPreviewUrl("http://localhost:5173", "src/A B.art.vue", "Default / wide"),
    `http://localhost:5173/site/components/${suffix}`,
  );
});

void test("exact hosted previews never fall back to a development URL", () => {
  const runner = new MuseaVrtRunner({
    previewUrls: { "A.art.vue": { Default: "https://gallery.invalid/actual.html" } },
  });
  assert.equal(
    runner.getPreviewUrl("http://localhost:5173", "A.art.vue", "Default"),
    "https://gallery.invalid/actual.html",
  );
  assert.throws(
    () => runner.getPreviewUrl("http://localhost:5173", "A.art.vue", "toString"),
    /Hosted preview not found/,
  );
  assert.throws(
    () => runner.getPreviewUrl("http://localhost:5173", "A.art.vue", "Missing"),
    /Hosted preview not found/,
  );
});

void test("CLI selects hosted URL and waits for readiness while allowing a deliberate capture override", () => {
  const options = parseArgs(["--gallery-url", "https://gallery.invalid/components/"]);
  assert.equal(options.galleryUrl, "https://gallery.invalid/components/");
  assert.equal(createVrtOptions(options).capture?.waitForPreviewReady, true);
  options.vrt = { capture: { waitForPreviewReady: false } };
  assert.equal(createVrtOptions(options).capture?.waitForPreviewReady, false);
  assert.throws(() => parseArgs(["--gallery-url"]), /requires a URL/);
  assert.throws(() => parseArgs(["--gallery-url", "--json"]), /requires a URL/);
});
