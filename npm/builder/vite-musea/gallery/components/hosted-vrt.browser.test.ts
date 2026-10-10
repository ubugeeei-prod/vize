import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdir, readFile, writeFile, rm, readdir, copyFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { PNG } from "pngjs";
import { chromium } from "playwright";
import { loadHostedGallery } from "../../src/cli/hosted.ts";
import { buildHostedGallery } from "./hosted-tests.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output = path.join(repository, "artifacts/musea-hosted-vrt");
const cli = fileURLToPath(new URL("../../src/cli/index.ts", import.meta.url));
const optionsUrl = pathToFileURL(
  fileURLToPath(new URL("../../src/plugin/options.ts", import.meta.url)),
).href;

async function runCli(cwd: string, args: string[]) {
  return new Promise<{ status: number; stdout: string; stderr: string }>((resolve, reject) => {
    execFile(
      process.execPath,
      ["--import", import.meta.resolve("tsx"), cli, ...args],
      { cwd, timeout: 45000, encoding: "utf8" },
      (error, stdout, stderr) => {
        if (error && typeof error.code !== "number") return reject(error);
        resolve({ status: error?.code ?? 0, stdout, stderr });
      },
    );
  });
}

await test(
  "hosted CLI captures mounted previews, detects diffs, approves and cleans without source files",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1" },
  async () => {
    await mkdir(output, { recursive: true });
    await rm(path.join(output, "reports"), { recursive: true, force: true });
    const fixture = JSON.parse(
      await readFile(
        path.join(repository, "tests/_fixtures/differential/musea/hosted-vrt.json"),
        "utf8",
      ),
    );
    const host = await buildHostedGallery(output, fixture.setupDelayMs);
    await rm(host.art.path);
    await writeFile(
      path.join(output, "vite.config.mjs"),
      `
      import { attachMuseaOptions } from ${JSON.stringify(optionsUrl)};
      export default { plugins: [attachMuseaOptions({ name: "vite-plugin-musea" }, {
        vrt: { viewports: [${JSON.stringify(fixture.viewport)}],
          capture: { waitForNetwork: ${fixture.waitForNetwork}, settleTime: ${fixture.settleTime} }, workers: ${fixture.workers} }
      })] };
    `,
    );
    const args = [
      "--gallery-url",
      host.url,
      "--config",
      "vite.config.mjs",
      "--output",
      "reports",
      "--json",
      "--ci",
    ];
    const observations: unknown[] = [];
    async function capture(stage: string, expectedStatus: number) {
      const result = await runCli(output, args);
      assert.equal(result.status, expectedStatus, `${result.stdout}\n${result.stderr}`);
      const report = JSON.parse(
        await readFile(path.join(output, "reports/vrt-report.json"), "utf8"),
      );
      await copyFile(
        path.join(output, "reports/vrt-report.json"),
        path.join(output, `${stage}.json`),
      );
      observations.push({ stage, ...result, summary: report.summary });
      assert.equal(report.summary.total, 6);
      assert.equal(report.summary.errors, 0);
      return report;
    }
    try {
      const first = await capture("baseline", 0);
      assert.equal(first.summary.new, 6);
      assert.ok(first.summary.duration >= 1000, "capture waited for the 1200ms async setup");
      const browser = await chromium.launch();
      try {
        const page = await browser.newPage({ viewport: { width: 320, height: 200 } });
        const hosted = await loadHostedGallery(host.url);
        await page.goto(hosted.previewUrls[host.art.path]["Clean one"]);
        await page.getByRole("button", { name: "Accessible action" }).waitFor();
        const expected = PNG.sync.read(
          await page.screenshot({ path: path.join(output, "mounted.png"), animations: "disabled" }),
        );
        const baseline = first.results.find(
          (result: { variant: string }) => result.variant === "Clean one",
        );
        const actual = PNG.sync.read(await readFile(path.resolve(output, baseline.snapshotPath)));
        assert.deepEqual(
          actual.data,
          expected.data,
          "CLI baseline contains the fully mounted component",
        );
      } finally {
        await browser.close();
      }
      for (const result of first.results) {
        const png = PNG.sync.read(await readFile(path.resolve(output, result.snapshotPath)));
        assert.equal(png.width, 320);
        assert.equal(png.height, 200);
      }
      const repeated = await capture("repeat", 0);
      assert.equal(repeated.summary.passed, 6);
      assert.equal(repeated.summary.new, 0);
      const previews = path.join(host.dist, "__musea__/preview");
      for (const file of await readdir(previews)) {
        const html = path.join(previews, file);
        await writeFile(
          html,
          (await readFile(html, "utf8")).replace(
            "</head>",
            `<style>${fixture.changedStyle}</style></head>`,
          ),
        );
      }
      const changed = await capture("changed", 1);
      assert.equal(changed.summary.failed, 6);
      for (const result of changed.results) assert.ok(result.diffPath && result.diffPercentage > 1);
      const approved = await runCli(output, ["approve", ...args.filter((arg) => arg !== "--ci")]);
      assert.equal(approved.status, 0, `${approved.stdout}\n${approved.stderr}`);
      assert.ok(approved.stdout.includes("Approved 6 snapshot(s)"));
      const final = await capture("approved", 0);
      assert.equal(final.summary.passed, 6);
      const audited = await runCli(output, [...args.filter((arg) => arg !== "--ci"), "--a11y"]);
      assert.equal(audited.status, 0, `${audited.stdout}\n${audited.stderr}`);
      const a11y = JSON.parse(
        await readFile(path.join(output, "reports/a11y-report.json"), "utf8"),
      );
      assert.equal(a11y.summary.totalVariants, 6);
      assert.equal(a11y.summary.erroredVariants, 0);
      assert.equal(
        a11y.results.filter((result: { violations: { id: string }[] }) =>
          result.violations.some((violation) => violation.id === "button-name"),
        ).length,
        3,
      );
      await copyFile(path.join(output, "reports/a11y-report.json"), path.join(output, "a11y.json"));
      observations.push({ stage: "a11y", summary: a11y.summary });
      const orphan = path.join(output, "reports/snapshots/Orphan--Default--small.png");
      await writeFile(orphan, "orphan");
      const clean = await runCli(output, ["clean", ...args.filter((arg) => arg !== "--ci")]);
      assert.equal(clean.status, 0, `${clean.stdout}\n${clean.stderr}`);
      await assert.rejects(readFile(orphan), /ENOENT/);
      assert.ok(host.requests.some((url) => url.includes("/api/static.json")));
      assert.ok(host.requests.some((url) => url.includes("/preview/")));
      assert.ok(
        !host.requests.some((url) => url.startsWith("/__musea__/") || url.includes("@vite")),
      );
    } finally {
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ observations, requests: host.requests }, null, 2),
      );
      await host.close();
    }
  },
);
