import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { chromium } from "playwright";
import { build, createServer, preview } from "vite";
import { staticPreviewId } from "./static-data.ts";
import {
  capture,
  fixture,
  previews,
  unusedPort,
  variants,
} from "./literal-variant-browser.fixture.ts";

const sectionIds = {
  Controls: ["variant-default", "variant-custom-theme"],
  Keys: ["variant-proto", "variant-constructor", "variant-hasownproperty"],
};
const native = { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1" };

void test(
  "native static export preserves every literal variant own URL and rendered preview",
  native,
  async () => {
    const f = await fixture("static");
    const errors: string[] = [];
    const consoleErrors: string[] = [];
    const observations: unknown[] = [];
    const browser = await chromium.launch();
    const page = await browser.newPage();
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("console", (message) => {
      if (message.type() === "error") consoleErrors.push(message.text());
    });
    const outDir = path.join(f.root, "dist");
    let server: Awaited<ReturnType<typeof preview>> | undefined;
    try {
      await build({ ...f.config, build: { outDir, emptyOutDir: true } });
      server = await preview({
        root: f.root,
        configFile: false,
        build: { outDir },
        preview: { host: "127.0.0.1", port: await unusedPort(), strictPort: true },
      });
      const origin = server.resolvedUrls!.local[0];
      const url = new URL("/__musea__/", origin).href;
      const raw = await readFile(path.join(outDir, "__musea__/api/static.json"), "utf8");
      await writeFile(path.join(f.output, "static.json"), raw);
      const response = await fetch(new URL("api/static.json", url));
      assert.equal(response.status, 200);
      assert.equal(await response.text(), raw);
      const payload = JSON.parse(raw);
      const manifest: unknown[] = [];
      for (const title of ["Controls", "Keys"] as const) {
        const artPath = path.join(f.root, "src", `${title}.art.vue`);
        const urls = payload.previews[artPath];
        manifest.push({
          title,
          keys: Object.keys(urls),
          own: variants[title].map(([name]) => ({
            name,
            own: Object.hasOwn(urls, name),
            valueType: typeof urls[name],
          })),
        });
        // Each physically emitted native module must render even if the gallery index loses its URL.
        for (const [name, button] of variants[title]) {
          const relative = `/__musea__/preview/${staticPreviewId(artPath, name)}.html`;
          const physical = await readFile(path.join(outDir, relative.slice(1)), "utf8");
          await page.goto(new URL(relative, origin).href);
          await page.getByRole("button", { name: button, exact: true }).waitFor();
          assert.equal(await page.locator("museacomponent").count(), 0);
          observations.push({
            title,
            name,
            url: page.url(),
            physicalSha256: createHash("sha256").update(physical).digest("hex"),
            html: await page.content(),
          });
        }
      }
      await page.goto(url);
      observations.push({ phase: "controls-gallery", previews: await previews(page, "Controls") });
      assert.deepEqual(errors, []);
      assert.deepEqual(consoleErrors, []);
      const controlGlobals = await page.evaluate(() => {
        const data = Reflect.get(window, "__MUSEA_STATIC_PREVIEWS__");
        return Object.keys(data).map((artPath) => ({
          artPath,
          keys: Object.keys(data[artPath]),
          prototypeUnchanged: Object.getPrototypeOf(data[artPath]) === Object.prototype,
          hasOwnToString: Object.hasOwn(data[artPath], "toString"),
          protoOwn: Object.hasOwn(data[artPath], "__proto__"),
          protoType: typeof data[artPath]["__proto__"],
        }));
      });
      let galleryFailure: string | undefined;
      try {
        observations.push({ phase: "keys-gallery", previews: await previews(page, "Keys") });
      } catch (error) {
        galleryFailure = String(error);
      }
      observations.push({ manifest, controlGlobals, galleryFailure });
      await page.screenshot({ path: path.join(f.output, "gallery.png"), fullPage: true });
      await writeFile(path.join(f.output, "gallery.html"), await page.content());
      for (const title of ["Controls", "Keys"] as const) {
        const urls = payload.previews[path.join(f.root, "src", `${title}.art.vue`)];
        assert.deepEqual(
          Object.keys(urls),
          variants[title].map(([name]) => name),
        );
      }
      assert.equal(galleryFailure, undefined);
      for (const item of observations as Array<{
        phase?: string;
        previews?: Awaited<ReturnType<typeof previews>>;
      }>) {
        const title =
          item.phase === "controls-gallery"
            ? "Controls"
            : item.phase === "keys-gallery"
              ? "Keys"
              : undefined;
        if (!title) continue;
        assert.ok(item.previews);
        assert.deepEqual(
          item.previews.map((frame) => frame.sectionName),
          variants[title].map(([name]) => name),
        );
        assert.deepEqual(
          item.previews.map((frame) => frame.sectionId),
          sectionIds[title],
        );
        assert.deepEqual(
          item.previews.map((frame) => frame.ariaControls),
          sectionIds[title],
        );
      }
      await page.locator(".variant-toc-item").last().click();
      await page.locator(".variant-toc-item").first().click();
      await page.locator('.variant-toc-item[aria-current="true"]').first().waitFor();
      assert.equal(
        await page.locator(".variant-toc-item").first().getAttribute("aria-current"),
        "true",
      );
      observations.push({
        phase: "literal-nav-selected",
        name: await page.locator(".variant-toc-name").first().textContent(),
        sectionId: await page.locator(".variant-section").first().getAttribute("id"),
        ariaControls: await page.locator(".variant-toc-item").first().getAttribute("aria-controls"),
      });
      assert.ok(controlGlobals.every((item) => item.prototypeUnchanged && !item.hasOwnToString));
      const keysGlobals = controlGlobals.find((item) => item.artPath.endsWith("/Keys.art.vue"));
      assert.equal(keysGlobals?.protoOwn, true);
      assert.equal(keysGlobals?.protoType, "string");
      assert.deepEqual(errors, []);
      assert.deepEqual(consoleErrors, []);
    } finally {
      await writeFile(
        path.join(f.output, "observations.json"),
        JSON.stringify({ sources: f.sources, observations, errors, consoleErrors }, null, 2),
      );
      await browser.close();
      if (server)
        await new Promise<void>((resolve, reject) =>
          server!.httpServer.close((error) => (error ? reject(error) : resolve())),
        );
      await f.retain();
    }
  },
);

void test(
  "real native VRT groups literal names as own data and retains separate repeated PNG results",
  native,
  async () => {
    const f = await fixture("dev");
    const server = await createServer({
      ...f.config,
      server: { host: "127.0.0.1", port: await unusedPort(), strictPort: true },
    });
    const browser = await chromium.launch();
    const page = await browser.newPage();
    const errors: string[] = [];
    const consoleErrors: string[] = [];
    const observations: unknown[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("console", (message) => {
      if (message.type() === "error") consoleErrors.push(message.text());
    });
    try {
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const url = `http://127.0.0.1:${address.port}/__musea__/`;
      for (const title of ["Controls", "Keys"] as const) {
        const previous: Array<string> = [];
        for (const phase of ["initial", "repeat"]) {
          await page.goto(url);
          const rendered = await previews(page, title);
          assert.ok(rendered.every((item) => item.unresolved === 0));
          await page.getByRole("button", { name: "VRT", exact: true }).click();
          const data = await capture(page, f.output, `${title}-${phase}`);
          assert.equal(data.summary.total, variants[title].length);
          assert.equal(data.summary.failed, 0);
          assert.equal(
            data.summary[phase === "initial" ? "new" : "passed"],
            variants[title].length,
          );
          assert.deepEqual(
            data.results.map((result) => result.variantName),
            variants[title].map(([name]) => name),
          );
          const files = await Promise.all(
            data.results.map((result) => readFile(result.snapshotPath)),
          );
          const hashes = files.map((bytes) => createHash("sha256").update(bytes).digest("hex"));
          assert.equal(
            new Set(data.results.map((result) => result.snapshotPath)).size,
            variants[title].length,
          );
          assert.equal(new Set(hashes).size, variants[title].length);
          if (phase === "initial") previous.push(...hashes);
          else {
            assert.deepEqual(hashes, previous);
            assert.ok(data.results.every((result) => result.diffPercentage === 0));
          }
          let renderFailure: string | undefined;
          try {
            await page
              .locator(".vrt-variant")
              .nth(variants[title].length - 1)
              .waitFor({ timeout: 8_000 });
          } catch (error) {
            renderFailure = String(error);
          }
          const groups = await page.locator(".vrt-variant-name").allTextContents();
          observations.push({ title, phase, rendered, data, hashes, groups, renderFailure });
          await page.screenshot({
            path: path.join(f.output, `${title}-${phase}.png`),
            fullPage: true,
          });
          await writeFile(path.join(f.output, `${title}-${phase}.html`), await page.content());
          if (title === "Controls") {
            assert.equal(renderFailure, undefined);
            assert.deepEqual(
              groups,
              variants[title].map(([name]) => name),
            );
          }
        }
      }
      // Preserve both real successful API captures and actual source UI exceptions before failing.
      assert.deepEqual(errors, []);
      assert.deepEqual(consoleErrors, []);
      for (const item of observations as Array<{
        groups: string[];
        title: keyof typeof variants;
        rendered: Array<{ sectionName: string; sectionId: string; ariaControls: string }>;
      }>) {
        assert.deepEqual(
          item.groups,
          variants[item.title].map(([name]) => name),
        );
        assert.deepEqual(
          item.rendered.map((frame) => frame.sectionName),
          variants[item.title].map(([name]) => name),
        );
        assert.deepEqual(
          item.rendered.map((frame) => frame.sectionId),
          sectionIds[item.title],
        );
        assert.deepEqual(
          item.rendered.map((frame) => frame.ariaControls),
          sectionIds[item.title],
        );
      }
    } finally {
      await writeFile(
        path.join(f.output, "observations.json"),
        JSON.stringify({ sources: f.sources, observations, errors, consoleErrors }, null, 2),
      );
      await browser.close();
      await server.close();
      await f.retain();
    }
  },
);
