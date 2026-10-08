import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Browser } from "playwright";
import { capturePageRender } from "./capture-page-render.ts";

/** Falsify the optimization when fixed element boxes hide changing text layout. */
export async function verifyCaptureRenderControls(
  browser: Browser,
  origin: string,
  output: string,
) {
  const destination = path.join(output, "capture-controls");
  await mkdir(destination, { recursive: true });
  const viewport = { width: 390, height: 844 };
  const page = await browser.newPage({ viewport, reducedMotion: "reduce" });
  try {
    await page.route("**/capture-control", (route) =>
      route.fulfill({
        contentType: "text/html",
        body: `<!doctype html><style>
        body { margin: 0; }
        .content { height: 10000px; }
        p { margin: 0; width: 320px; height: 48px; line-height: 24px;
            font-size: 2vh; white-space: nowrap; }
      </style><div class="content"><p>Viewport-sensitive text</p></div>`,
      }),
    );
    await page.goto(`${origin}/capture-control`);
    const measure = () =>
      page.evaluate(() => {
        const paragraph = document.querySelector("p");
        if (!paragraph?.firstChild)
          throw new Error("Authored capture control has no text authority");
        const range = document.createRange();
        range.selectNodeContents(paragraph.firstChild);
        const rect = (value: DOMRect) => [value.x, value.y, value.width, value.height];
        const content = document.querySelector(".content");
        if (!content) throw new Error("Authored capture control has no content authority");
        return {
          source: content.outerHTML,
          elements: [...document.querySelectorAll(".content, .content *")].map((node) =>
            rect(node.getBoundingClientRect()),
          ),
          text: [...range.getClientRects()].map(rect),
        };
      });
    const before = await measure();
    await page.setViewportSize({ ...viewport, height: 8192 });
    const enlarged = await measure();
    assert.equal(enlarged.source, before.source, "authored control source is unchanged");
    assert.deepEqual(enlarged.elements, before.elements, "fixed boxes hide the text change");
    assert.notDeepEqual(enlarged.text, before.text, "text Range exposes the viewport dependency");
    await page.setViewportSize(viewport);
    await page.evaluate(() => scrollTo(0, 37));
    const capture = await capturePageRender(page, destination, "viewport-sensitive.png");
    assert(capture.layout, "Long capture retains its comparison receipt");
    assert.equal(capture.layout.identical, false, "changing text must reject enlarged capture");
    assert.equal(capture.layout.captureViewportHeight, viewport.height);
    assert.equal(capture.height, 10000);
    assert.equal(capture.tiles[0].top, 0);
    const last = capture.tiles.at(-1);
    assert(last, "Full authored page retains its final tile");
    assert.equal(last.top + last.height, 10000);
    for (let index = 1; index < capture.tiles.length; index += 1)
      assert(
        capture.tiles[index].top < capture.tiles[index - 1].top + capture.tiles[index - 1].height,
      );
    assert.deepEqual(page.viewportSize(), viewport, "original viewport is restored");
    assert.deepEqual(await page.evaluate(() => ({ x: scrollX, y: scrollY })), { x: 0, y: 37 });
    await page.evaluate(() => scrollTo(0, 0));
    assert.deepEqual(await measure(), before, "complete original source and text layout restore");
    await writeFile(
      path.join(destination, "receipt.json"),
      JSON.stringify({ before, enlarged, capture }, null, 2) + "\n",
    );
  } finally {
    await page.close();
  }
}
