/// <reference lib="dom" />
/// <reference lib="dom.iterable" />
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import type { JSHandle, Page } from "playwright";

type Rectangle = [number, number, number, number];
type SnapshotEntry = {
  node: Node;
  type: number;
  source: string | null;
  bounds: Rectangle;
  rectangles: Rectangle[];
};
type ContentSnapshot = {
  content: Element;
  snapshot: () => SnapshotEntry[];
  before: SnapshotEntry[];
};
type LayoutProof = {
  descendants: number;
  elements: number;
  textNodes: number;
  rectangles: number;
  beforeSha256: string;
  afterSha256: string;
  identical: boolean;
  originalViewportHeight?: number;
  captureViewportHeight?: number;
};
export type PageCapture = {
  width: number;
  viewportHeight: number;
  height: number;
  mode: "full-page" | "overlapping-viewports";
  layout?: LayoutProof;
  tiles: { top: number; height: number; file: string; sha256: string }[];
};

/** Capture every pixel, enlarging only a viewport proven to retain the content layout. */
export async function capturePageRender(
  page: Page,
  output: string,
  name: string,
): Promise<PageCapture> {
  const viewport = page.viewportSize();
  const scroll = await page.evaluate(() => ({ x: scrollX, y: scrollY }));
  let original: JSHandle<ContentSnapshot> | undefined;
  try {
    await page.evaluate(() => scrollTo(0, 0));
    const dimensions = await page.evaluate(() => ({
      width: innerWidth,
      viewportHeight: innerHeight,
      height: Math.ceil(document.documentElement.scrollHeight),
    }));
    const single = dimensions.height <= 8192;
    let layout: LayoutProof | undefined;
    if (!single && viewport && dimensions.viewportHeight < 8192) {
      // Keep complete node/source/rectangle comparisons in Chromium. Sending
      // hundreds of thousands of descendant rectangles over CDP is unnecessary.
      original = await page.evaluateHandle(() => {
        const content = document.querySelector(".content");
        if (!content) throw new Error("Long-page capture requires its content authority");
        const rectangle = (rect: DOMRect): Rectangle => [rect.x, rect.y, rect.width, rect.height];
        const snapshot = (): SnapshotEntry[] => {
          const walker = document.createTreeWalker(content, NodeFilter.SHOW_ALL);
          const nodes: Node[] = [content];
          for (let node = walker.nextNode(); node; node = walker.nextNode()) nodes.push(node);
          return nodes.map((node) => {
            const range = document.createRange();
            range.selectNodeContents(node);
            const target = node instanceof Element ? node : range;
            return {
              node,
              type: node.nodeType,
              source: node instanceof Element ? node.outerHTML : node.nodeValue,
              bounds: rectangle(target.getBoundingClientRect()),
              rectangles: [...target.getClientRects()].map(rectangle),
            };
          });
        };
        const before = snapshot();
        return { content, snapshot, before };
      });
      await page.setViewportSize({ width: viewport.width, height: 8192 });
      await page.evaluate(() => scrollTo(0, 0));
      layout = await original.evaluate(async ({ content, snapshot, before }) => {
        const after = snapshot();
        const packets = (nodes: SnapshotEntry[]) =>
          nodes.map(({ node: _node, ...packet }) => packet);
        const beforePackets = packets(before);
        const afterPackets = packets(after);
        const hash = async (packet: Omit<SnapshotEntry, "node">[]) =>
          [
            ...new Uint8Array(
              await crypto.subtle.digest(
                "SHA-256",
                new TextEncoder().encode(JSON.stringify(packet)),
              ),
            ),
          ]
            .map((byte) => byte.toString(16).padStart(2, "0"))
            .join("");
        const sameNodes =
          document.querySelector(".content") === content &&
          before.length === after.length &&
          before.every((entry, index) => entry.node === after[index].node);
        return {
          descendants: before.length - 1,
          elements: before.filter((entry) => entry.type === Node.ELEMENT_NODE).length,
          textNodes: before.filter((entry) => entry.type === Node.TEXT_NODE).length,
          rectangles: before.reduce((sum, entry) => sum + entry.rectangles.length, 0),
          beforeSha256: await hash(beforePackets),
          afterSha256: await hash(afterPackets),
          identical: sameNodes && JSON.stringify(beforePackets) === JSON.stringify(afterPackets),
        };
      });
      const enlarged = await page.evaluate(() => ({
        width: innerWidth,
        height: innerHeight,
        documentHeight: Math.ceil(document.documentElement.scrollHeight),
      }));
      layout.identical &&=
        enlarged.width === dimensions.width && enlarged.documentHeight === dimensions.height;
      if (layout.identical) dimensions.viewportHeight = enlarged.height;
      else {
        await page.setViewportSize(viewport);
        await page.evaluate(() => scrollTo(0, 0));
        assert(
          await original.evaluate(({ snapshot, before }) => {
            const restored = snapshot();
            return (
              restored.length === before.length &&
              before.every((entry, index) => {
                const after = restored[index];
                return (
                  entry.node === after.node &&
                  entry.source === after.source &&
                  JSON.stringify(entry.bounds) === JSON.stringify(after.bounds) &&
                  JSON.stringify(entry.rectangles) === JSON.stringify(after.rectangles)
                );
              })
            );
          }),
          `${name}: original content layout must restore before fallback capture`,
        );
      }
      layout.originalViewportHeight = viewport.height;
      layout.captureViewportHeight = dimensions.viewportHeight;
    }
    const last = Math.max(0, dimensions.height - dimensions.viewportHeight);
    const positions: number[] = [];
    if (single) positions.push(0);
    else {
      for (let top = 0; top < last; top += dimensions.viewportHeight - 128) positions.push(top);
      positions.push(last);
    }
    const tiles: PageCapture["tiles"] = [];
    for (const top of positions) {
      await page.evaluate((top) => scrollTo(0, top), top);
      assert.equal(await page.evaluate(() => scrollY), top, `${name}: exact capture position`);
      const height = single ? dimensions.height : dimensions.viewportHeight;
      const file = single
        ? name
        : name.replace(/\.png$/, `-${String(tiles.length).padStart(3, "0")}.png`);
      const png = await page.screenshot({
        path: path.join(output, file),
        fullPage: single,
        animations: "disabled",
      });
      assert.equal(png.readUInt32BE(16), dimensions.width, `${file}: complete page width`);
      assert.equal(png.readUInt32BE(20), height, `${file}: complete tile height`);
      tiles.push({ top, height, file, sha256: createHash("sha256").update(png).digest("hex") });
    }
    assert.equal(tiles[0].top, 0, `${name}: capture starts at the document top`);
    for (let index = 1; index < tiles.length; index += 1)
      assert(tiles[index].top <= tiles[index - 1].top + tiles[index - 1].height);
    const finalTile = tiles.at(-1);
    assert(finalTile, `${name}: full-page capture retains its final tile`);
    assert.equal(finalTile.top + finalTile.height, dimensions.height);
    assert.equal(
      await page.evaluate(() => Math.ceil(document.documentElement.scrollHeight)),
      dimensions.height,
      `${name}: layout changed during full-page capture`,
    );
    return { ...dimensions, mode: single ? "full-page" : "overlapping-viewports", layout, tiles };
  } finally {
    if (viewport) await page.setViewportSize(viewport);
    await page.evaluate(({ x, y }) => scrollTo(x, y), scroll);
    await original?.dispose();
  }
}
