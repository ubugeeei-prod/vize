import assert from "node:assert/strict";
import { cp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import { sha256, writeVrtConfig, type VrtFixture } from "./vrt_fixtures.ts";
import { inventory, type Evidence } from "./vrt_artifacts.ts";
import type { staticVrtHost } from "./vrt_host.ts";

/** Authored Arts are compiled by the installed plugins, including every original audit trait. */
export async function observeHostedAudit(
  page: Page,
  f: VrtFixture,
  e: Evidence,
  host: Awaited<ReturnType<typeof staticVrtHost>>,
  beforeRefusal: () => void,
) {
  const input = await readFile(path.join(process.cwd(), "vrt-inputs/hosted-audits.json"));
  const contract = JSON.parse(input.toString()) as {
    setupDelayMs: number;
    variants: string[];
    successfulCounts: string[];
    missingVendorCounts: string[];
  };
  assert.deepEqual(contract, {
    setupDelayMs: 300,
    variants: ["Clean one", "Broken one", "Clean two", "Broken two", "Clean three", "Broken three"],
    successfulCounts: ["6", "3", "3", "0"],
    missingVendorCounts: ["6", "0", "6", "0"],
  });
  await e.save("hosted-audit/contract.json", input);
  const authored = `<art title="Hosted">\n${contract.variants
    .map(
      (name, i) =>
        `  <variant name="${name}"${i === 0 ? " default" : ""}><main><button type="button">${name.startsWith("Clean") ? "Accessible action" : ""}</button></main></variant>`,
    )
    .join("\n")}\n</art>\n`;
  const artPath = path.join(f.root, "src/Host.art.vue");
  await mkdir(path.dirname(artPath));
  await writeFile(artPath, authored, { flag: "wx" });
  const setup = `export default async function() {
  await new Promise(resolve => setTimeout(resolve, ${contract.setupDelayMs}));
  Reflect.set(window, '__installedHostedSetupAfterLoad', document.readyState === 'complete');
}\n`;
  await writeFile(path.join(f.root, "vrt-delayed.setup.ts"), setup, { flag: "wx" });
  writeVrtConfig(f, true);
  await e.save("hosted-audit/inputs/Host.art.vue", authored);
  await e.save("hosted-audit/inputs/vrt-delayed.setup.ts", setup);
  await e.save("hosted-audit/inputs/vite.config.mjs", await readFile(f.config));
  const { build } = await import("vite");
  const directory = path.join(f.root, "dist");
  await build({
    root: f.root,
    configFile: f.config,
    logLevel: "warn",
    build: { outDir: directory, emptyOutDir: true },
  });
  const raw = await readFile(path.join(directory, "gallery/vrt/api/static.json"));
  const manifest = JSON.parse(raw.toString()) as {
    arts: Array<{ path: string; variants: Array<{ name: string }> }>;
    previews: Record<string, Record<string, string>>;
  };
  await e.save("hosted-audit/static.json", raw);
  assert.equal(manifest.arts.length, 1);
  assert.equal(manifest.arts[0].path, artPath);
  assert.deepEqual(
    manifest.arts[0].variants.map((item) => item.name),
    contract.variants,
  );
  assert.ok(Object.hasOwn(manifest.previews, artPath));
  assert.deepEqual(Object.keys(manifest.previews[artPath]), contract.variants);
  await cp(directory, path.join(e.output, "hosted-audit/dist"), { recursive: true });
  e.records.push({
    phase: "hosted-audit-build",
    contractSha256: sha256(input),
    authoredSha256: sha256(authored),
    manifestSha256: sha256(raw),
    dist: await inventory(directory),
  });
  await rm(path.join(f.root, "src"), { recursive: true });
  await rm(path.join(f.root, "vrt-delayed.setup.ts"));
  await assert.rejects(readFile(artPath), { code: "ENOENT" });
  const response = await fetch(new URL("api/static.json", host.url));
  assert.equal(response.status, 200);
  assert.deepEqual(Buffer.from(await response.arrayBuffer()), raw);
  for (const [i, name] of contract.variants.entries()) {
    const preview = manifest.previews[artPath][name];
    assert.ok(preview.startsWith("/built/gallery/vrt/preview/"));
    await page.goto(new URL(preview, host.url).href);
    await page.waitForFunction(
      () => Reflect.get(window, "__installedHostedSetupAfterLoad") === true,
    );
    assert.equal(await page.locator("museacomponent").count(), 0);
    assert.equal(await page.locator("main button").count(), 1);
    assert.equal(
      await page.locator("main button").innerText(),
      name.startsWith("Clean") ? "Accessible action" : "",
    );
    const document = await page.content();
    await e.save(`hosted-audit/preview-${i}.html`, document);
    e.records.push({
      phase: "hosted-audit-native-preview",
      name,
      url: page.url(),
      documentSha256: sha256(document),
      setupAfterLoad: true,
    });
  }
  await page.goto(new URL("tests", host.url).href);
  const runAll = () => page.getByRole("button", { name: "Run All A11y Tests", exact: true });
  for (let repeat = 0; repeat < 2; repeat++) {
    await runAll().click();
    await runAll().waitFor({ timeout: 15_000 });
    const counts = await page.locator(".summary-stats .stat-value").allTextContents();
    assert.deepEqual(counts.slice(0, 4), contract.successfulCounts);
    assert.equal(
      await page
        .locator(".test-item.failed .violation-tag")
        .filter({ hasText: "button-name" })
        .count(),
      3,
    );
    assert.equal(await page.locator(".test-item.passed .count.passes").count(), 3);
    const document = await page.content();
    await e.save(`hosted-audit/repeat-${repeat}.html`, document);
    e.records.push({
      phase: `hosted-audit-repeat-${repeat}`,
      counts,
      tests: await page.locator(".test-list").innerText(),
      documentSha256: sha256(document),
    });
  }
  await page.locator(".test-item").first().click();
  const frame = page.frameLocator(".variant-card iframe").first();
  await frame.getByRole("button", { name: "Accessible action", exact: true }).waitFor();
  assert.equal(
    await frame
      .locator("body")
      .evaluate(() => Reflect.get(window, "__installedHostedSetupAfterLoad")),
    true,
  );
  assert.equal(await frame.locator("museacomponent").count(), 0);
  await page.locator(".tab-btn").filter({ hasText: "A11y" }).click();
  await page.getByRole("button", { name: /^Run (Test|Again)$/ }).click();
  await page.locator(".a11y-panel .a11y-success").waitFor({ timeout: 10_000 });
  await e.save("hosted-audit/individual.html", await page.content());
  await page.getByRole("button", { name: "VRT", exact: true }).click();
  await page.getByText("Capture this hosted gallery with Node and Playwright:").waitFor();
  assert.ok((await page.locator(".vrt-command").innerText()).includes(host.url));
  assert.equal(await page.getByRole("button", { name: "Run VRT", exact: true }).count(), 0);
  await e.save("hosted-audit/vrt-notice.html", await page.content());
  await page.screenshot({ path: path.join(e.output, "hosted-audit/complete.png"), fullPage: true });
  beforeRefusal();
  await page.goto(new URL("tests", host.url).href);
  host.refuseAxe();
  await runAll().click();
  await runAll().waitFor({ timeout: 15_000 });
  const counts = await page.locator(".summary-stats .stat-value").allTextContents();
  assert.deepEqual(counts.slice(0, 4), contract.missingVendorCounts);
  assert.equal(await page.locator(".test-item.failed .test-error").count(), 6);
  const document = await page.content();
  await e.save("hosted-audit/vendor-refusal.html", document);
  e.records.push({
    phase: "hosted-audit-vendor-refusal",
    counts,
    tests: await page.locator(".test-list").innerText(),
    documentSha256: sha256(document),
  });
  assert.ok(
    host.requests.some((item) => {
      const request = item as { pathname: string; status: number };
      return request.pathname.endsWith("/vendor/axe-core.min.js") && request.status === 404;
    }),
  );
}
