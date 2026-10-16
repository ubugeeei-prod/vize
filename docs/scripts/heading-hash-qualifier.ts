import assert from "node:assert/strict";
import fs from "node:fs";
import { createServer } from "node:http";
import path from "node:path";
import os from "node:os";
import { chromium } from "playwright";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import {
  buildHeadingHashFixture,
  headingCases,
  missingHashes,
  verifyCoreAsset,
} from "./heading-hash-fixture.ts";

export async function qualifyHeadingHashes(output: string, site?: string) {
  fs.mkdirSync(output, { recursive: true });
  if (site) verifyCoreAsset(site, true);
  const scratch = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-heading-hashes-")));
  const fixture = await buildHeadingHashFixture(scratch).catch((error: unknown) => {
    fs.rmSync(scratch, { recursive: true, force: true });
    throw error;
  });
  const errors: string[] = [];
  let siteRoot = fixture.before;
  const server = createServer((request, response) => {
    const pathname = decodeURIComponent(new URL(request.url!, "http://localhost").pathname);
    const file = path.resolve(siteRoot, `.${pathname}`);
    if (
      !file.startsWith(siteRoot + path.sep) ||
      !fs.existsSync(file) ||
      !fs.statSync(file).isFile()
    ) {
      response.writeHead(404).end();
      return;
    }
    response
      .writeHead(200, {
        "content-type": file.endsWith(".js")
          ? "text/javascript"
          : file.endsWith(".css")
            ? "text/css"
            : "text/html",
      })
      .end(fs.readFileSync(file));
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert(address && typeof address !== "string");
  const origin = `http://127.0.0.1:${address.port}`;
  const browser = await chromium
    .launch({ executablePath: resolvePuppeteerExecutablePath() })
    .catch(async (error: unknown) => {
      fs.rmSync(scratch, { recursive: true, force: true });
      await new Promise<void>((resolve) => server.close(() => resolve()));
      throw error;
    });
  const observations: { flow: string; hash: string; target: string | null }[] = [];
  const inherited: string[] = [];
  let completed = false;
  const receipt = () => ({
    completed,
    inheritedErrors: inherited,
    errors,
    observations,
    originalCore: fixture.originalCore,
    repairedCore: fixture.repairedCore,
  });
  try {
    const before = await browser.newPage();
    before.on("pageerror", (error) => inherited.push(String(error)));
    await before.goto(`${origin}/index.html${headingCases[1][1]}`);
    await before.waitForTimeout(200);
    assert(
      inherited.some(
        (error) => error.includes("querySelector") && error.includes("valid selector"),
      ),
      "the real unpatched build must reproduce the inherited exception",
    );
    await before.close();
    siteRoot = fixture.after;
    const page = await browser.newPage({ reducedMotion: "reduce" });
    page.on("pageerror", (error) => errors.push(String(error)));
    await page.addInitScript(() => {
      const runtime = window as typeof window & { headingScrolls: string[] };
      runtime.headingScrolls = [];
      const original = Reflect.get(Element.prototype, "scrollIntoView") as (
        this: Element,
        arg?: boolean | ScrollIntoViewOptions,
      ) => void;
      Element.prototype.scrollIntoView = function (...args) {
        runtime.headingScrolls.push(this.id);
        original.apply(this, args);
      };
    });
    const reset = () =>
      page.evaluate(() => {
        (window as typeof window & { headingScrolls: string[] }).headingScrolls = [];
      });
    const observe = async (flow: string, hash: string, target: string | null) => {
      if (target) {
        await page
          .waitForFunction(
            (id) =>
              (window as typeof window & { headingScrolls: string[] }).headingScrolls.includes(id),
            target,
          )
          .catch(async (cause: unknown) => {
            const state = await page.evaluate(() => ({
              hash: location.hash,
              scrolls: (window as typeof window & { headingScrolls: string[] }).headingScrolls,
            }));
            throw new Error(`${flow} ${hash}: ${JSON.stringify({ state, errors, observations })}`, {
              cause,
            });
          });
        await page.waitForFunction((id) => {
          const top = document.getElementById(id)!.getBoundingClientRect().top;
          return top >= -1 && top < 200;
        }, target);
      } else {
        await page.waitForTimeout(150);
        assert.deepEqual(
          await page.evaluate(
            () => (window as typeof window & { headingScrolls: string[] }).headingScrolls,
          ),
          [],
        );
      }
      assert.deepEqual(errors, [], `${flow} ${hash}: no uncaught runtime exception`);
      observations.push({ flow, hash, target });
    };
    for (const [index, [id, hash]] of headingCases.entries()) {
      await page.goto("about:blank");
      await page.goto(`${origin}/index.html${hash}`);
      await observe("initial", hash, id);
      await reset();
      await page.evaluate(() => history.replaceState(null, "", location.pathname));
      await page.evaluate((value) => {
        location.hash = value;
      }, hash);
      await observe("hashchange", hash, id);
      await reset();
      await page.evaluate(() => history.replaceState(null, "", location.pathname));
      await page.locator(`[data-heading-case="${index}"]`).click();
      await observe("click", hash, id);
    }
    for (const hash of missingHashes) {
      await page.goto("about:blank");
      await page.goto(`${origin}/index.html${hash}`);
      await observe("missing-initial", hash, null);
      await reset();
      await page.evaluate(() => history.replaceState(null, "", location.pathname));
      await page.evaluate((value) => {
        location.hash = value;
      }, hash);
      await observe("missing-hashchange", hash, null);
      await reset();
      await page.evaluate((value) => {
        const link = document.querySelector<HTMLAnchorElement>('[data-heading-case="0"]')!;
        link.setAttribute("href", value);
      }, hash);
      await page.locator('[data-heading-case="0"]').click();
      await observe("missing-click", hash, null);
    }
    await page.goto("about:blank");
    await page.goto(`${origin}/index.html${headingCases[1][1]}`);
    await observe("final-japanese", headingCases[1][1], headingCases[1][0]);
    await page.screenshot({ path: path.join(output, "japanese-anchor.png"), fullPage: false });
    await page.close();
    completed = true;
    return receipt();
  } finally {
    fs.writeFileSync(path.join(output, "receipt.json"), JSON.stringify(receipt(), null, 2) + "\n");
    fs.rmSync(scratch, { recursive: true, force: true });
    await browser.close();
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
}
