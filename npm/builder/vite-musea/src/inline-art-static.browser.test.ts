import assert from "node:assert/strict";
import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { chromium } from "playwright";
import { stringifyQuery } from "vue-router";
import {
  frames,
  rememberDocuments,
  documentIdentity,
} from "../../../../tools/support/release/public_acceptance/gallery.ts";
import {
  buildNativeStaticGallery,
  hostNativeStaticGallery,
  repository,
  defaults,
  changed,
  sha256,
} from "./inline-art-static.fixtures.ts";

void test(
  "built HTTP gallery renders native inline Self and colon variants with globals and stable Documents",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 120000 },
  async (t) => {
    const output = path.join(repository, "artifacts/musea-native-static");
    await mkdir(path.join(output, "responses"), { recursive: true });
    const built = await buildNativeStaticGallery(output);
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
      await page.goto(`${host.origin}/built/gallery/`);
      await page.locator(".art-item").first().waitFor();
      const href = await page
        .locator(".art-item")
        .filter({ hasText: "Button" })
        .first()
        .getAttribute("href");
      assert.ok(href);
      const initial = new URL(href, page.url());
      initial.search = stringifyQuery({ museaGlobals: JSON.stringify(defaults), retained: "keep" });
      initial.hash = "#variant-default";
      await page.goto(initial.href); // Actual HTTP deep-link request, not client-only navigation.
      const before = await frames(page, defaults);
      assert.equal(before.length, 6);
      const rendered = await page.evaluate(() =>
        [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map((iframe) => ({
          variant: iframe
            .contentDocument!.querySelector("[data-variant]")!
            .getAttribute("data-variant"),
          buttons: [
            ...iframe.contentDocument!.querySelectorAll<HTMLButtonElement>("button.btn"),
          ].map((button) => ({
            text: button.textContent,
            disabled: button.disabled,
            borderStyle: iframe.contentWindow!.getComputedStyle(button).borderStyle,
            scoped: [...button.attributes].some((attribute) =>
              attribute.name.startsWith("data-v-"),
            ),
          })),
          unresolved: iframe.contentDocument!.querySelectorAll("museacomponent").length,
        })),
      );
      assert.deepEqual(
        rendered.map((variant) => variant.buttons.length),
        [1, 1, 1, 1, 3, 4],
      );
      assert.ok(
        rendered.every(
          (variant) =>
            variant.unresolved === 0 &&
            variant.buttons.every((button) => button.scoped && button.borderStyle === "solid"),
        ),
      );
      assert.equal(rendered[3].buttons[0].disabled, true);
      assert.ok(
        before.every(
          (frame) =>
            frame.url.startsWith(`${host.origin}/built/gallery/preview/`) &&
            new URL(frame.url).pathname.endsWith(".html"),
        ),
      );
      observations.push({ phase: "built-native-defaults", frames: before, rendered });
      await rememberDocuments(page);
      await page.getByRole("combobox", { name: "Brand", exact: true }).selectOption("1");
      await page
        .getByRole("group", { name: "Component theme", exact: true })
        .getByRole("button", { name: "Dark", exact: true })
        .click();
      await page.getByRole("combobox", { name: "Locale", exact: true }).selectOption("1");
      const after = await frames(page, changed);
      assert.equal((await documentIdentity(page)).unchanged, true);
      assert.deepEqual(
        after.map((frame) => frame.documentToken),
        before.map((frame) => frame.documentToken),
      );
      for (const frame of after) assert.deepEqual(frame.firstGlobals, defaults);
      assert.equal(new URL(page.url()).hash, "#variant-default");
      assert.equal(new URL(page.url()).searchParams.get("retained"), "keep");
      assert.deepEqual(JSON.parse(new URL(page.url()).searchParams.get("museaGlobals")!), changed);
      observations.push({ phase: "built-native-globals-no-remount", frames: after });
      await page.reload();
      const reloaded = await frames(page, changed);
      for (const frame of reloaded) assert.deepEqual(frame.firstGlobals, changed);
      observations.push({ phase: "built-native-deep-link-reload", frames: reloaded });
      await page.screenshot({ path: path.join(output, "gallery.png"), fullPage: true });
      const art = built.manifest.arts.find(
        (art) => art.metadata.title === "Native static variants",
      );
      assert.ok(art);
      assert.deepEqual(
        art.variants.map((variant) => variant.name),
        ["Default", "State: enabled"],
      );
      const colonFrames = [];
      for (const variant of art.variants) {
        const url = new URL(built.manifest.previews[art.path][variant.name], host.origin);
        url.searchParams.set("museaGlobals", JSON.stringify(changed));
        await page.goto(url.href);
        await page.locator("button.native-host").waitFor();
        const rendered = await page.evaluate(() => {
          const button = document.querySelector<HTMLButtonElement>("button.native-host")!;
          return {
            variant: document.querySelector("[data-variant]")!.getAttribute("data-variant"),
            text: button.textContent,
            color: getComputedStyle(button).color,
            unresolved: document.querySelectorAll("host,museacomponent").length,
            globals: Reflect.get(window, "__publicFirstGlobals"),
            setupCalls: Reflect.get(window, "__publicSetupCalls"),
          };
        });
        assert.equal(rendered.variant, variant.name);
        assert.equal(rendered.text, "Native host");
        assert.equal(rendered.color, "rgb(12, 34, 56)");
        assert.equal(rendered.unresolved, 0);
        assert.equal(rendered.setupCalls, 1);
        assert.deepEqual(rendered.globals, changed);
        colonFrames.push({ url: url.href, rendered });
      }
      observations.push({ phase: "built-native-literal-colon", frames: colonFrames });
      await page.screenshot({ path: path.join(output, "colon.png"), fullPage: true });
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
      const documents = await Promise.all(
        page
          .frames()
          .map(async (frame) => ({ url: frame.url(), html: await frame.content().catch(String) })),
      );
      await Promise.all(pending);
      await writeFile(
        path.join(output, "failure.json"),
        JSON.stringify(
          { error: String(error), documents, errors, records, server: host.responses },
          null,
          2,
        ),
      );
      await page.screenshot({ path: path.join(output, "failure.png"), fullPage: true });
      throw error;
    }
  },
);
