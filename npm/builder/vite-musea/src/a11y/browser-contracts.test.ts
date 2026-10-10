import assert from "node:assert/strict";
import { createServer } from "node:http";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { chromium } from "playwright";
import { PNG } from "pngjs";
import { MuseaA11yRunner } from "./index.ts";
import { generatePreviewHtml } from "../preview/html.ts";
import { MuseaVrtRunner } from "../vrt/runner.ts";
import type { ArtFileInfo } from "../types/index.ts";

const browserTest = { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" };
const art: ArtFileInfo = {
  path: "Spinner.art.vue",
  metadata: { title: "Spinner", tags: [], status: "ready" },
  variants: [{ name: "Default", template: "<div />", isDefault: true, skipVrt: false }],
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
};

void test(
  "tall preview content scrolls as a document without a body accessibility violation",
  browserTest,
  async () => {
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage({ viewport: { width: 800, height: 600 } });
      const html = generatePreviewHtml(art, art.variants[0], "/__musea__").replace(
        /<script\b[^>]*>[\s\S]*?<\/script>/g,
        "",
      );
      await page.setContent(html);
      await page.addStyleTag({ content: "html { overflow-y: scroll }" });
      await page.evaluate(() => {
        const content = document.createElement("main");
        content.className = "musea-variant";
        content.innerHTML = Array.from({ length: 100 }, (_, index) => `<p>Line ${index}</p>`).join(
          "",
        );
        document.body.replaceChildren(content);
      });
      const runner = new MuseaA11yRunner({ includeRules: ["scrollable-region-focusable"] });
      const result = await runner.auditPage(page, art.path, "Tall");
      assert.ok(
        !result.violations.some((v) => v.id === "scrollable-region-focusable"),
        JSON.stringify(result),
      );
      const sizing = await page.evaluate(() => ({
        body: document.body.clientHeight,
        content: document.body.scrollHeight,
        viewport: window.innerHeight,
        overflow: getComputedStyle(document.body).overflowY,
      }));
      assert.equal(sizing.overflow, "visible");
      assert.ok(sizing.body >= sizing.content && sizing.body > sizing.viewport);
      // Negative control: the formerly authored wrapper must reproduce the issue.
      await page.addStyleTag({ content: "html, body { height: 100% } body { overflow: auto }" });
      const broken = await runner.auditPage(page, art.path, "Tall");
      assert.ok(
        broken.violations.some(
          (v) =>
            v.id === "scrollable-region-focusable" &&
            v.targets?.some((node) => node.target.includes("body")),
        ),
      );
    } finally {
      await browser.close();
    }
  },
);

void test(
  "real axe contrast results retain measured colors, ratio, selector, and HTML",
  browserTest,
  async () => {
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage();
      await page.setContent(
        '<html lang="en"><head><title>Contrast</title></head><body style="background:#fff"><main><span id="label" style="color:#bbb">Label</span></main></body></html>',
      );
      const runner = new MuseaA11yRunner();
      const result = await runner.auditPage(page, art.path, "Contrast");
      const violation = result.violations.find((v) => v.id === "color-contrast");
      assert.ok(violation);
      const node = violation.targets?.find((node) => node.target.includes("#label"));
      assert.ok(node?.html.includes('id="label"'));
      assert.ok(node.failureSummary?.includes("contrast"));
      const data = node.any.find((check) => check.id === "color-contrast")?.data as Record<
        string,
        unknown
      >;
      assert.equal(data.fgColor, "#bbbbbb");
      assert.equal(data.bgColor, "#ffffff");
      assert.ok(typeof data.contrastRatio === "number" && data.contrastRatio < 2);
      assert.deepEqual(
        JSON.parse(runner.generateJsonReport([result])).results[0].violations,
        JSON.parse(JSON.stringify(result.violations)),
      );
    } finally {
      await browser.close();
    }
  },
);

void test(
  "animated variants capture identically and reduced motion reaches the browser",
  browserTest,
  async () => {
    const workspace = mkdtempSync(path.join(os.tmpdir(), "musea-motion-"));
    const html = readFileSync(new URL("./fixtures/animated-preview.html", import.meta.url), "utf8");
    const server = createServer((_request, response) => {
      response.setHeader("Content-Type", "text/html");
      response.end(html);
    });
    const runner = new MuseaVrtRunner({
      snapshotDir: workspace,
      projectRoot: "/",
      capture: { reducedMotion: "reduce", settleTime: 0 },
    });
    try {
      await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
      const address = server.address();
      assert.ok(address && typeof address !== "string");
      await runner.init();
      const url = `http://127.0.0.1:${address.port}`;
      const viewport = { width: 320, height: 240 };
      const first = await runner.captureAndCompare(art, "Default", viewport, url);
      assert.equal(first.passed, true, first.error);
      assert.equal(first.isNew, true);
      for (let iteration = 0; iteration < 3; iteration++) {
        const result = await runner.captureAndCompare(art, "Default", viewport, url);
        assert.equal(result.passed, true, result.error);
        assert.equal(result.diffPixels, 0);
      }
      const screenshot = PNG.sync.read(readFileSync(first.currentPath!));
      assert.deepEqual([...screenshot.data.subarray(0, 4)], [0, 255, 0, 255]);
      assert.ok(
        screenshot.data.some((byte, index) => index % 4 === 0 && byte > 0),
        "the spinner remains visible",
      );
    } finally {
      await runner.close();
      await new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      );
      rmSync(workspace, { recursive: true, force: true });
    }
  },
);
