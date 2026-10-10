import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Browser } from "playwright";
import {
  measureLocaleDropdown,
  verifyLiveLocaleLinks,
  verifyLocaleDropdown,
} from "./locale-render-assertions.ts";
import { verifyResponsiveLocaleDropdown } from "./locale-responsive-assertions.ts";
import { switchDocsTheme } from "./theme-render-assertions.ts";

/** Use dedicated real pages so lifecycle probes cannot alter the navigation oracle. */
export async function verifyLocaleRenderControls(browser: Browser, origin: string, output: string) {
  const destination = path.join(output, "locale-controls");
  await mkdir(destination, { recursive: true });
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const sha256 = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
  const sourceFiles = [
    "docs/theme/i18n/locale-switcher.js",
    "docs/theme/i18n/locale-selector.css",
    "docs/theme/i18n/navigation.js",
    "docs/theme/background.ts",
    "docs/scripts/locale-render-assertions.ts",
    "docs/scripts/locale-responsive-assertions.ts",
    "docs/scripts/locale-control-render.ts",
    "docs/scripts/verify-navigation-render.ts",
  ];
  const receipt = {
    sourceHead: null as string | null,
    workflow: {
      sha: process.env.GITHUB_SHA ?? null,
      run: process.env.GITHUB_RUN_ID ?? null,
      attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    },
    source: [] as { file: string; bytes: number; sha256: string }[],
    completed: false,
    failure: null as string | null,
    cases: [] as unknown[],
  };
  try {
    for (const file of sourceFiles) {
      const bytes = await readFile(path.join(root, file));
      receipt.source.push({ file, bytes: bytes.length, sha256: sha256(bytes) });
    }
    receipt.sourceHead = execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: root,
      encoding: "utf8",
    }).trim();
    for (const [device, viewport] of [
      ["desktop", { width: 1440, height: 960 }],
      ["mobile", { width: 390, height: 844 }],
    ] as const) {
      for (const locale of ["en", "ja"]) {
        for (const theme of ["light", "dark"]) {
          const name = `${locale}-${device}-${theme}`;
          const context = await browser.newContext({ viewport, reducedMotion: "reduce" });
          const errors: { url: string; error: string }[] = [];
          context.on("page", (page) =>
            page.on("pageerror", (error) => errors.push({ url: page.url(), error: String(error) })),
          );
          const page = await context.newPage();
          const report = {
            name,
            errors,
            html: null as unknown,
            metrics: null as unknown,
            screenshot: null as unknown,
            controls: null as unknown,
            liveLinks: null as unknown,
            responsive: null as unknown,
            finalSource: null as unknown,
          };
          receipt.cases.push(report);
          try {
            const route = `${locale === "en" ? "" : "/ja"}/getting-started/index.html`;
            const response = await page.goto(
              `${origin}${route}?locale-control=initial%20state#locale-control`,
              { waitUntil: "networkidle" },
            );
            assert(response, `${name}: rendered page returned no response`);
            assert.equal(response.status(), 200, name);
            const html = await response.body();
            const htmlFile = `${name}.html`;
            await writeFile(path.join(destination, htmlFile), html);
            report.html = {
              url: response.url(),
              file: htmlFile,
              bytes: html.length,
              sha256: sha256(html),
            };
            await page.locator("h1").first().waitFor();
            await page.evaluate(() => document.fonts.ready);
            await switchDocsTheme(page, theme, device);
            assert.equal(await page.locator("html").getAttribute("data-theme"), theme);
            await page.evaluate(() => scrollTo(0, 0));
            await page.locator(".docs-locale-select").press("ArrowDown");
            await page.keyboard.press("ArrowDown");
            await page.locator('.docs-locale-option[data-locale="fr"]').hover();
            report.metrics = await measureLocaleDropdown(page);
            const screenshot = `${name}.png`;
            const png = await page.screenshot({
              path: path.join(destination, screenshot),
              animations: "disabled",
            });
            report.screenshot = { file: screenshot, bytes: png.length, sha256: sha256(png) };
            await page.keyboard.press("Escape");
            if (theme === "light") {
              report.controls = await verifyLocaleDropdown(page);
              report.liveLinks = await verifyLiveLocaleLinks(page);
              if (locale === "en" && device === "desktop")
                report.responsive = await verifyResponsiveLocaleDropdown(page);
            }
            const finalSource = await page.content();
            const finalFile = `${name}-final.html`;
            await writeFile(path.join(destination, finalFile), finalSource);
            report.finalSource = {
              url: page.url(),
              file: finalFile,
              bytes: Buffer.byteLength(finalSource),
              sha256: sha256(finalSource),
            };
            assert.deepEqual(errors, [], `${name}: complete main/popup page-error stream`);
          } finally {
            await context.close();
          }
        }
      }
    }
    receipt.completed = true;
  } catch (error) {
    receipt.failure = error instanceof Error ? (error.stack ?? String(error)) : String(error);
    throw error;
  } finally {
    await writeFile(
      path.join(destination, "receipt.json"),
      JSON.stringify(receipt, null, 2) + "\n",
    );
  }
}
