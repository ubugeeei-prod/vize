import assert from "node:assert/strict";
import { createServer } from "node:http";
import { mkdir, mkdtemp, readFile, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import test from "node:test";
import { PNG } from "pngjs";
import type { ArtFileInfo, BeforeCaptureContext } from "../types/index.ts";
import { MuseaVrtRunner } from "./runner.ts";
import type { CliOptions } from "../cli/index.ts";

const output = fileURLToPath(
  new URL("../../../../../output/playwright/musea-interactions/", import.meta.url),
);
const art: ArtFileInfo = {
  path: "src/Form.art.vue",
  metadata: { title: "Form", tags: [], status: "ready" },
  variants: ["Submitted", "Alternate"].map((name) => ({
    name,
    template: "<Form />",
    isDefault: false,
    skipVrt: false,
  })),
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
};
const viewport = { width: 160, height: 120, name: "small" };

void test(
  "authored capture interactions are awaited, isolated, and fail before screenshot or baseline updates",
  {
    skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
  },
  async () => {
    await mkdir(output, { recursive: true });
    const snapshotDir = await mkdtemp(path.join(output, "direct-"));
    const observations: unknown[] = [];
    const server = createServer((_request, response) => {
      response.setHeader("Content-Type", "text/html");
      response.end(
        `<style>html,body{margin:0}form{height:40px}#result{display:block;height:80px;background:#ff0000}input{width:80px}</style><main class="musea-variant"><form><label>Message<input name="message" /></label><button>Apply</button></form><output id="result">Initial</output></main><script>document.querySelector('form').onsubmit=(event)=>{event.preventDefault();const result=document.querySelector('#result');result.textContent=document.querySelector('input').value;result.style.background='#0000ff'};</script>`,
      );
    });
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    const base = `http://127.0.0.1:${address.port}`;
    const contexts: BeforeCaptureContext[] = [];
    const runner = new MuseaVrtRunner({
      projectRoot: output,
      snapshotDir,
      workers: 2,
      viewports: [viewport],
      threshold: 0,
      capture: {
        waitForNetwork: false,
        settleTime: 0,
        hideElements: ["form"],
        beforeCapture: async (page, context) => {
          const started = performance.now();
          contexts.push(context);
          assert.equal(await page.getByRole("textbox", { name: "Message" }).inputValue(), "");
          assert.equal(Object.isFrozen(context), true);
          assert.equal(Object.isFrozen(context.viewport), true);
          await page
            .getByRole("textbox", { name: "Message" })
            .fill(`投稿 🎨 ${context.variantName}`);
          await page.getByRole("button", { name: "Apply", exact: true }).click();
          assert.equal(
            await page.locator("#result").textContent(),
            `投稿 🎨 ${context.variantName}`,
          );
          // The screenshot must await completion, including asynchronous assertions.
          await new Promise((resolve) => setTimeout(resolve, 20));
          observations.push({
            phase: "interaction-timing",
            variantName: context.variantName,
            elapsedMs: performance.now() - started,
          });
        },
      },
    });
    try {
      await runner.init();
      const results = await runner.runAllTests([art], base);
      assert.equal(results.length, 2);
      assert.ok(results.every((result) => result.passed && result.isNew));
      assert.deepEqual(contexts.map((context) => context.variantName).toSorted(), [
        "Alternate",
        "Submitted",
      ]);
      for (const context of contexts) {
        assert.equal(context.artPath, art.path);
        assert.deepEqual(context.viewport, viewport);
        assert.notEqual(context.viewport, viewport);
      }
      for (const result of results) {
        const png = PNG.sync.read(await readFile(result.currentPath!));
        assert.equal(png.width, viewport.width);
        assert.equal(png.height, viewport.height);
        const at = (110 * png.width + 150) * 4;
        assert.deepEqual([...png.data.subarray(at, at + 4)], [0, 0, 255, 255]);
      }
    observations.push({ phase: "programmatic-interactions", results, contexts: [...contexts] });
      const repeat = await runner.runAllTests([art], base);
      assert.ok(repeat.every((result) => result.passed && result.diffPixels === 0));
      observations.push({ phase: "repeat-identical", results: repeat });

      const existing = results[0];
      const baseline = await readFile(existing.snapshotPath);
      const current = await readFile(existing.currentPath!);
      runner.getCapture().beforeCapture = async () => {
        throw new Error("expected assertion failure");
      };
      const failed = await runner.captureAndCompare(art, existing.variantName, viewport, base);
      assert.equal(failed.passed, false);
      assert.equal(failed.error, "beforeCapture failed: expected assertion failure");
      assert.equal(failed.currentPath, undefined);
      assert.deepEqual(await readFile(existing.snapshotPath), baseline);
      assert.deepEqual(await readFile(existing.currentPath!), current);
      const missing = await runner.captureAndCompare(art, "FailsBeforeBaseline", viewport, base);
      assert.equal(missing.error, "beforeCapture failed: expected assertion failure");
      await assert.rejects(stat(missing.snapshotPath), { code: "ENOENT" });
      assert.deepEqual(runner.getBrowser()!.contexts(), []);
      observations.push({ phase: "hook-failures-preserve-images", failed, missing });
    } finally {
      await runner.close();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(observations, null, 2),
      );
    }
  },
);

void test(
  "Vite configuration and CLI option projection retain and execute the authored interaction callback",
  {
    skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1",
  },
  async () => {
    // Configuration loads the compiler plugin; the native CI lane builds the
    // exact-source binding before executing this integration contract.
    const { loadMuseaVrtOptions } = await import("../cli/config.ts");
    const { createVrtOptions } = await import("../cli/commands.ts");
    await mkdir(output, { recursive: true });
    const snapshotDir = await mkdtemp(path.join(output, "configured-"));
    const config = path.join(output, "interaction.config.ts");
    const carrier = pathToFileURL(
      fileURLToPath(new URL("../plugin/options.ts", import.meta.url)),
    ).href;
    await writeFile(
      config,
      `import assert from 'node:assert/strict';
    import { attachMuseaOptions } from ${JSON.stringify(carrier)};
    export default { plugins: [attachMuseaOptions({name:'musea-fixture'}, {vrt: {
      capture: { waitForNetwork: false, waitForPreviewReady: false, settleTime: 0,
        beforeCapture: async (page, context) => {
          assert.equal(context.variantName, 'Submitted');
          await page.getByRole('button', {name: 'Prepare'}).click();
          assert.equal(await page.locator('#result').textContent(), 'Prepared by config');
        }
      }
    }})] };`,
    );
    const configured = await loadMuseaVrtOptions(config, output);
    assert.equal(typeof configured?.capture?.beforeCapture, "function");
    const projected = createVrtOptions({
      vrt: configured,
      projectRoot: output,
      output: snapshotDir,
      threshold: 0,
      thresholdProvided: false,
      colorThresholdProvided: false,
      workersProvided: false,
      browserArgs: [],
    } as unknown as CliOptions);
    assert.equal(projected.capture?.beforeCapture, configured!.capture!.beforeCapture);
    const server = createServer((_request, response) => {
      response.setHeader("Content-Type", "text/html");
      response.end(
        `<div class="musea-variant"><button onclick="document.querySelector('#result').textContent='Prepared by config'">Prepare</button><output id="result">Initial</output></div>`,
      );
    });
    await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
    const address = server.address();
    assert.ok(address && typeof address !== "string");
    const runner = new MuseaVrtRunner(projected);
    try {
      await runner.init();
      const result = await runner.captureAndCompare(
        art,
        "Submitted",
        viewport,
        `http://127.0.0.1:${address.port}`,
      );
      assert.equal(result.error, undefined);
      assert.equal(result.passed, true);
      assert.equal(result.isNew, true);
      assert.ok((await stat(result.currentPath!)).size > 0);
      await writeFile(
        path.join(output, "configured-observations.json"),
        JSON.stringify(
          {
            callbackRetained:
              projected.capture?.beforeCapture === configured!.capture!.beforeCapture,
            result,
          },
          null,
          2,
        ),
      );
    } finally {
      await runner.close();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
    }
  },
);
