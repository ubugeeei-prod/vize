import assert from "node:assert/strict";
import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { chromium } from "playwright";
import { hostNativeStaticGallery, repository, sha256 } from "./inline-art-static.fixtures.ts";
import { buildNativeBindingGallery } from "./variant-bindings-native.fixtures.ts";

void test(
  "native built HTTP previews distinguish normalized variant names and preserve ordinary controls",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 120000 },
  async (t) => {
    const output = path.join(repository, "artifacts/musea-native-bindings");
    await mkdir(path.join(output, "responses"), { recursive: true });
    const built = await buildNativeBindingGallery(output);
    const cleanup: (() => Promise<unknown>)[] = [
      () => rm(built.root, { recursive: true, force: true }),
    ];
    t.after(async () => {
      for (const close of cleanup.reverse()) await close();
    });
    const host = await hostNativeStaticGallery(built.directory);
    cleanup.push(() => host.close());
    const browser = await chromium.launch();
    cleanup.push(() => browser.close());
    const page = await browser.newPage();
    const errors: string[] = [];
    const records: { url: string; status: number; sha256?: string; error?: string }[] = [];
    const pending: Promise<void>[] = [];
    const observations: unknown[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("response", (response) => {
      const record: (typeof records)[number] = { url: response.url(), status: response.status() };
      records.push(record);
      pending.push(
        (async () => {
          try {
            const bytes = await response.body();
            record.sha256 = sha256(bytes);
            await writeFile(path.join(output, "responses", record.sha256), bytes);
          } catch (error) {
            record.error = String(error);
          }
        })(),
      );
    });
    try {
      assert.equal(built.manifest.arts.length, 2);
      for (const kind of ["collision", "ordinary"]) {
        const art = built.manifest.arts.find((art) => art.path.endsWith(`/${kind}/Host.art.vue`));
        assert.ok(art);
        assert.deepEqual(
          art.variants.map((variant) => variant.name),
          ["State enabled", kind === "collision" ? "State-enabled" : "State disabled"],
        );
        assert.equal(art.variants[0].isDefault, true);
        for (const [index, variant] of art.variants.entries()) {
          const url = new URL(built.manifest.previews[art.path][variant.name], host.origin);
          assert.ok(url.pathname.startsWith("/built/gallery/preview/"));
          assert.ok(url.pathname.endsWith(".html"));
          await page.goto(url.href);
          await page.locator("button.native-host").waitFor();
          const rendered = await page.evaluate(() => {
            const button = document.querySelector<HTMLButtonElement>("button.native-host")!;
            return {
              variant: document.querySelector("[data-variant]")!.getAttribute("data-variant"),
              text: button.textContent,
              color: getComputedStyle(button).color,
              scoped: [...button.attributes].some((attribute) =>
                attribute.name.startsWith("data-v-"),
              ),
              unresolved: document.querySelectorAll("host,museacomponent").length,
              html: document.documentElement.outerHTML,
            };
          });
          assert.equal(rendered.variant, variant.name);
          assert.equal(rendered.text, index === 0 ? "Space variant" : "Dash variant");
          assert.equal(rendered.color, "rgb(12, 34, 56)");
          assert.equal(rendered.scoped, true);
          assert.equal(rendered.unresolved, 0);
          observations.push({ kind, url: url.href, rendered });
          await page.screenshot({
            path: path.join(output, `${kind}-${index}.png`),
            fullPage: true,
          });
        }
      }
      await Promise.all(pending);
      assert.deepEqual(errors, []);
      assert.ok(
        records.every((record) => record.status < 400 && record.error === undefined),
        JSON.stringify(records),
      );
      assert.ok(
        records.every(
          (record) => !record.url.includes("/@vite/") && !record.url.includes("virtual:"),
        ),
      );
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(
          {
            purpose: "Exact source-native built HTTP contracts, not installed public acceptance",
            observations,
            errors,
            records,
            server: host.responses,
          },
          null,
          2,
        ),
      );
    } catch (error) {
      await Promise.all(pending);
      await writeFile(
        path.join(output, "failure.json"),
        JSON.stringify(
          {
            error: String(error),
            html: await page.content(),
            errors,
            records,
            server: host.responses,
          },
          null,
          2,
        ),
      );
      await page.screenshot({ path: path.join(output, "failure.png"), fullPage: true });
      throw error;
    }
  },
);
