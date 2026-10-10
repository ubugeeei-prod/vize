import assert from "node:assert/strict";
import { mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import { PNG } from "pngjs";
import type { Page } from "playwright";
import { build } from "vite";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import { fixture, variants } from "./literal-variant-browser.fixture.ts";
import { sha256 } from "./hosted-vrt-build.fixtures.ts";

export type LiteralTitle = keyof typeof variants;

/** Keep the frozen authored corpus/helper; only mount its real build under the secure host prefix. */
export async function buildLiteralServiceGallery() {
  const f = await fixture("hosted");
  const directory = path.join(f.root, "dist");
  await build({
    ...f.config,
    base: "/built/",
    logLevel: "warn",
    plugins: [vize(), musea({ include: ["src/**/*.art.vue"], basePath: "/gallery/" })],
    build: { outDir: directory, emptyOutDir: true, minify: true },
  });
  const raw = await readFile(path.join(directory, "gallery/api/static.json"));
  const manifest = JSON.parse(raw.toString()) as {
    snapshotIdentityVersion: number;
    snapshotIdentities: Record<string, string>;
    arts: Array<{ path: string; metadata: { title: LiteralTitle } }>;
    previews: Record<string, Record<string, string>>;
  };
  assert.equal(manifest.snapshotIdentityVersion, 1);
  assert.equal(manifest.arts.length, 2);
  await writeFile(path.join(f.output, "static.json"), raw);
  await rm(path.join(f.root, "src"), { recursive: true });
  for (const title of ["Controls", "Keys"] as const)
    await assert.rejects(stat(path.join(f.root, "src", `${title}.art.vue`)), { code: "ENOENT" });
  return { ...f, directory, manifest, raw };
}

interface Capture {
  success: boolean;
  summary: { total: number; new: number; passed: number; failed: number; errors: number };
  results: Array<{
    artPath: string;
    variantName: string;
    viewport: string;
    isNew?: boolean;
    passed: boolean;
    diffPercentage?: number;
    images: Record<string, string>;
  }>;
  reports: { json: string; html: string };
}

export async function retainedReports(local: string, title: LiteralTitle) {
  const reports: Record<string, Buffer> = {};
  for (const kind of ["json", "html"]) {
    try {
      reports[kind] = await readFile(path.join(local, `reports/vrt-${title}-report.${kind}`));
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  return reports;
}

/** Every byte comes from the real authenticated service and its own physical report paths. */
export async function captureLiteralService(input: {
  page: Page;
  session: { endpoint: string; token: string };
  origin: string;
  local: string;
  output: string;
  title: LiteralTitle;
  artPath: string;
  phase: "new" | "match";
  first?: Buffer[];
}) {
  const { page, session, title, phase } = input;
  const [response] = await Promise.all([
    page.waitForResponse(
      (item) => item.url() === `${session.endpoint}/capture` && item.request().method() === "POST",
    ),
    page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  ]);
  assert.deepEqual(response.request().postDataJSON(), { artPath: input.artPath, update: false });
  const raw = await response.body();
  const data = JSON.parse(raw.toString()) as Capture;
  const destination = path.join(input.output, `${title}-${phase}`);
  await mkdir(destination, { recursive: true });
  await writeFile(path.join(destination, "response.json"), raw);
  assert.equal(response.status(), 200, raw.toString());
  assert.equal(data.success, true);
  assert.deepEqual(
    [
      data.summary.total,
      data.summary.new,
      data.summary.passed,
      data.summary.failed,
      data.summary.errors,
    ],
    [
      variants[title].length,
      phase === "new" ? variants[title].length : 0,
      phase === "match" ? variants[title].length : 0,
      0,
      0,
    ],
  );
  assert.deepEqual(
    data.results.map((item) => item.variantName),
    variants[title].map(([name]) => name),
  );
  for (const item of data.results) {
    assert.equal(item.artPath, input.artPath);
    assert.equal(item.viewport, "authored");
    if (phase === "match") assert.equal(item.diffPercentage, 0);
  }
  const artifact = async (route: string) => {
    assert.match(route, /^\/artifacts\/[0-9a-f-]{36}$/);
    const actual = await fetch(`${session.endpoint}${route}`, {
      headers: { Origin: input.origin, Authorization: `Bearer ${session.token}` },
    });
    assert.equal(actual.status, 200);
    return Buffer.from(await actual.arrayBuffer());
  };
  const owned = await retainedReports(input.local, title);
  const report = JSON.parse(owned.json.toString());
  assert.deepEqual(report.reportOwner, { version: 1, artIdentity: `src/${title}.art.vue` });
  assert.deepEqual(
    report.results.map((item: { variantName: string }) => item.variantName),
    variants[title].map(([name]) => name),
  );
  const baselines: Buffer[] = [];
  const imageReceipts: unknown[] = [];
  for (const [index, item] of data.results.entries()) {
    for (const [kind, route] of Object.entries(item.images)) {
      const bytes = await artifact(route);
      assert.deepEqual(bytes, await readFile(report.results[index][`${kind}Path`]));
      const png = PNG.sync.read(bytes);
      assert.deepEqual([png.width, png.height], [320, 180]);
      await writeFile(path.join(destination, `${index}-${kind}.png`), bytes);
      imageReceipts.push({
        index,
        variant: item.variantName,
        kind,
        bytes: bytes.length,
        sha256: sha256(bytes),
      });
      if (kind === "snapshot") baselines[index] = bytes;
      if (phase === "match") assert.deepEqual(bytes, input.first![index]);
    }
    assert.ok(baselines[index]);
  }
  assert.equal(new Set(baselines.map(sha256)).size, variants[title].length);
  await page.locator(".vrt-results img").first().waitFor();
  await page.waitForFunction(() =>
    [...document.querySelectorAll<HTMLImageElement>(".vrt-results img")].every(
      (image) => image.complete && image.naturalWidth === 320 && image.src.startsWith("blob:"),
    ),
  );
  assert.deepEqual(
    await page.locator(".vrt-variant-name").allTextContents(),
    variants[title].map(([name]) => name),
  );
  const reportReceipts: unknown[] = [];
  for (const kind of ["json", "html"] as const) {
    const bytes = await artifact(data.reports[kind]);
    assert.deepEqual(bytes, owned[kind]);
    await writeFile(path.join(destination, `authenticated-report.${kind}`), bytes);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page
        .getByRole("button", { name: `Download ${kind.toUpperCase()} report`, exact: true })
        .click(),
    ]);
    const downloaded = path.join(destination, `downloaded-report.${kind}`);
    await download.saveAs(downloaded);
    assert.deepEqual(await readFile(downloaded), bytes);
    reportReceipts.push({ kind, bytes: bytes.length, sha256: sha256(bytes) });
  }
  return {
    baselines,
    receipt: { title, phase, data, images: imageReceipts, reports: reportReceipts },
  };
}
