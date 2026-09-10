import assert from "node:assert/strict";
import test from "node:test";

import type { ArtFileInfo, ViewportConfig } from "../types/index.ts";
import { MuseaVrtRunner, normalizeVrtWorkerCount } from "./runner.ts";
import type { VrtResult } from "./types.ts";

void test("worker count is normalized to a positive integer", () => {
  assert.equal(normalizeVrtWorkerCount(undefined), 1);
  assert.equal(normalizeVrtWorkerCount(0), 1);
  assert.equal(normalizeVrtWorkerCount(-2), 1);
  assert.equal(normalizeVrtWorkerCount(Number.NaN), 1);
  assert.equal(normalizeVrtWorkerCount(2.9), 2);
});

void test("runAllTests captures variants concurrently while preserving result order", async () => {
  const runner = new StubVrtRunner({ workers: 2, viewports: [{ width: 320, height: 240 }] });
  runner.markInitialized();

  const results = await runner.runAllTests(
    [art("Button.art.vue", ["primary", "secondary", "disabled"]), art("Badge.art.vue", ["info"])],
    "http://localhost:5173",
  );

  assert.equal(runner.maxActiveCaptures, 2);
  assert.deepEqual(
    results.map((result) => `${result.artPath}:${result.variantName}`),
    [
      "Button.art.vue:primary",
      "Button.art.vue:secondary",
      "Button.art.vue:disabled",
      "Badge.art.vue:info",
    ],
  );
});

class StubVrtRunner extends MuseaVrtRunner {
  activeCaptures = 0;
  maxActiveCaptures = 0;

  markInitialized(): void {
    (this as unknown as { browser: unknown }).browser = {};
  }

  override async captureAndCompare(
    artFile: ArtFileInfo,
    variantName: string,
    viewport: ViewportConfig,
  ): Promise<VrtResult> {
    this.activeCaptures++;
    this.maxActiveCaptures = Math.max(this.maxActiveCaptures, this.activeCaptures);
    await new Promise((resolve) => setTimeout(resolve, variantName === "primary" ? 30 : 1));
    this.activeCaptures--;

    return {
      artPath: artFile.path,
      variantName,
      viewport,
      passed: true,
      snapshotPath: `${artFile.path}-${variantName}.png`,
    };
  }
}

function art(path: string, variants: string[]): ArtFileInfo {
  return {
    path,
    metadata: {
      title: path,
      tags: [],
      status: "ready",
    },
    variants: variants.map((name) => ({
      name,
      template: `<div>${name}</div>`,
      isDefault: false,
      skipVrt: false,
    })),
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}
