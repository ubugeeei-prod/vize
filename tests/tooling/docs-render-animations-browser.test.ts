import assert from "node:assert/strict";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import type { Page } from "playwright";
import { resolvePuppeteerExecutablePath } from "../../docs/browser-path.js";
import { waitForAnimations } from "../../docs/scripts/theme-render-assertions.ts";

const { chromium } = createRequire(path.resolve(import.meta.dirname, "../../docs/package.json"))(
  "playwright",
) as typeof import("playwright");

const fixture = `<style>
  a { color: rgb(0, 0, 0); transition: color 300ms linear; }
  html.dark a { color: rgb(255, 255, 255); }
</style>
<details><summary>Closed group</summary><a id="skipped" href="#">Skipped</a></details>
<a id="rendered" href="#">Rendered</a>`;

async function bounded<T>(work: Promise<T>, message: string) {
  let timer: NodeJS.Timeout | undefined;
  const limit = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(message)), 10_000);
  });
  try {
    return await Promise.race([work, limit]);
  } finally {
    clearTimeout(timer);
  }
}

async function withPage(run: (page: Page) => Promise<void>) {
  const browser = await chromium.launch({ executablePath: resolvePuppeteerExecutablePath() });
  try {
    const page = await browser.newPage();
    await page.setContent(fixture);
    await run(page);
  } finally {
    await browser.close();
  }
}

void test("render waits settle transitions inside a closed <details>", { timeout: 60_000 }, () =>
  withPage(async (page) => {
    const animations = await page.evaluateHandle(() => {
      const skipped = document.getElementById("skipped")!;
      // A style read in skipped content computes that subtree, so the theme switch
      // starts a transition there as well as on the rendered link.
      getComputedStyle(skipped).getPropertyValue("color");
      document.documentElement.classList.add("dark");
      getComputedStyle(skipped).getPropertyValue("color");
      return document.getAnimations();
    });
    assert.deepEqual(
      await animations.evaluate((list) =>
        list.map((animation) => {
          const target = (animation.effect as KeyframeEffect).target!;
          return { id: target.id, rendered: target.checkVisibility() };
        }),
      ),
      [
        { id: "skipped", rendered: false },
        { id: "rendered", rendered: true },
      ],
    );
    await bounded(
      animations.evaluate(waitForAnimations),
      "the wait must settle an animation in skipped content",
    );
    // It still waits out the rendered transition: a linear 300ms change reads its end color.
    assert.deepEqual(
      await animations.evaluate((list) => ({
        states: list.map((animation) => animation.playState),
        color: getComputedStyle(document.getElementById("rendered")!).color,
      })),
      { states: ["finished", "finished"], color: "rgb(255, 255, 255)" },
    );
  }),
);

void test("render waits reject an animation cancelled before it ends", { timeout: 60_000 }, () =>
  withPage(async (page) => {
    const animations = await page.evaluateHandle(() => {
      const animation = document
        .getElementById("rendered")!
        .animate([{ opacity: 0 }, { opacity: 1 }], 10_000);
      setTimeout(() => animation.cancel(), 50);
      return [animation];
    });
    await assert.rejects(
      bounded(animations.evaluate(waitForAnimations), "a cancelled animation must reject"),
      /cancelled before it finished|AbortError/,
    );
  }),
);
