import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { PNG } from "pngjs";
import { writePng } from "./comparison.ts";

const comparisonModule = new URL("./runner-comparison.ts", import.meta.url).href;

// A subprocess makes an uncaught stream error, an unhandled rejection, and a
// hanging comparison observable independently of the test runner's handlers.
function inChild(source: string): unknown {
  const child = spawnSync(
    process.execPath,
    ["--import", import.meta.resolve("tsx"), "--input-type=module", "--eval", source],
    { encoding: "utf8", timeout: 10000 },
  );
  assert.equal(child.error, undefined);
  assert.equal(child.signal, null);
  assert.equal(child.status, 0, child.stderr);
  assert.equal(child.stderr, "");
  return JSON.parse(child.stdout);
}

for (const side of ["baseline", "current"]) {
  for (const failure of ["missing", "directory", "corrupt"]) {
    void test(`comparison rejects a ${failure} ${side} without terminating or hanging`, async () => {
      const root = await mkdtemp(path.join(os.tmpdir(), "musea-png-source-"));
      try {
        const baseline = path.join(root, "baseline.png");
        const current = path.join(root, "current.png");
        const invalid = side === "baseline" ? baseline : current;
        await writePng(new PNG({ width: 1, height: 1 }), side === "baseline" ? current : baseline);
        if (failure === "directory") await mkdir(invalid);
        if (failure === "corrupt") await writeFile(invalid, "not a PNG");

        const result = inChild(`
          import { compareImages } from ${JSON.stringify(comparisonModule)};
          const paths = ${JSON.stringify([baseline, current, path.join(root, "diff.png")])};
          try {
            await compareImages(...paths, {});
            process.stdout.write(JSON.stringify({ rejected: false }));
          } catch (error) {
            process.stdout.write(JSON.stringify({
              rejected: true, code: error.code ?? null, message: error.message,
            }));
          }
        `) as { rejected: boolean; code: string | null; message: string };

        assert.equal(result.rejected, true);
        assert.equal(
          result.code,
          failure === "missing" ? "ENOENT" : failure === "directory" ? "EISDIR" : null,
        );
        assert.equal(typeof result.message, "string");
        assert.ok(result.message.length > 0);
        assert.deepEqual(Object.keys(result), ["rejected", "code", "message"]);
      } finally {
        await rm(root, { recursive: true, force: true });
      }
    });
  }
}

void test("an unreadable baseline returns the complete failed capture and closes its resources", async () => {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-png-capture-"));
  try {
    await mkdir(path.join(root, "baseline.png"));
    const source = path.join(root, "capture.png");
    await writePng(new PNG({ width: 1, height: 1 }), source);

    const output = inChild(`
      import fs from "node:fs/promises";
      import { captureAndCompare } from ${JSON.stringify(comparisonModule)};
      const closed = [];
      const page = {
        async goto() {}, async waitForSelector() {}, async waitForTimeout() {},
        async screenshot({ path }) { await fs.copyFile(${JSON.stringify(source)}, path); },
        async close() { closed.push("page"); },
      };
      const context = { async newPage() { return page; }, async close() { closed.push("context"); } };
      const runner = {
        getBrowser() { return { async newContext() { return context; } }; },
        getOptions() { return { snapshotDir: ${JSON.stringify(root)}, threshold: 0 }; },
        getCapture() { return { hideElements: [], maskElements: [] }; },
        getComparison() { return {}; },
        getHostedNavigation() {},
        getPreviewUrl() { return "http://localhost/preview"; },
        async getSnapshotName() { return "baseline.png"; },
      };
      const result = await captureAndCompare(runner, { path: "Button.art.vue" }, "Default", { width: 1, height: 1 }, "http://localhost");
      process.stdout.write(JSON.stringify({ result, closed }));
    `) as { result: { error: string }; closed: string[] };

    assert.match(output.result.error, /EISDIR/);
    assert.deepEqual(output, {
      result: {
        artPath: "Button.art.vue",
        variantName: "Default",
        viewport: { width: 1, height: 1 },
        passed: false,
        snapshotPath: path.join(root, "baseline.png"),
        error: output.result.error,
      },
      closed: ["page", "context"],
    });
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
