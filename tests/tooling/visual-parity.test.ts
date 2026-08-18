import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import { PNG } from "pngjs";

import {
  VISUAL_STABILITY_CSS,
  applyVisualStabilityStyles,
  comparePngBuffers,
  visualComparisonDimensions,
  visualDiffWithinBudget,
} from "../_helpers/visual-parity.ts";

const STABILITY_STYLE_OPTIONS = {
  css: VISUAL_STABILITY_CSS,
  sheetKey: "__vizeVisualStabilitySheetTest",
  styleId: "vize-visual-stability-test",
} as const;

test("visual parity compares the shared viewport width and full page height", () => {
  assert.deepEqual(
    visualComparisonDimensions({ height: 15_617, width: 3_349 }, { height: 15_617, width: 1_280 }),
    { height: 15_617, width: 1_280 },
  );

  assert.deepEqual(
    visualComparisonDimensions({ height: 720, width: 1_280 }, { height: 900, width: 1_280 }),
    { height: 900, width: 1_280 },
  );

  assert.deepEqual(
    visualComparisonDimensions(
      { height: 14_674, width: 1_208 },
      { height: 14_674, width: 1_208 },
      390,
    ),
    { height: 14_674, width: 390 },
  );
});

test("visual parity ignores tiny raster noise but keeps visible pixel changes", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-visual-parity-"));
  const reference = solidPng([250, 250, 250, 255]);
  const nearMatch = solidPng([248, 248, 248, 255]);
  const visibleChange = solidPng([0, 0, 0, 255]);

  assert.equal(
    comparePngBuffers(reference, nearMatch, path.join(dir, "near.png"), { threshold: 0.1 })
      .diffPixels,
    0,
  );
  assert.equal(
    comparePngBuffers(reference, visibleChange, path.join(dir, "visible.png"), { threshold: 0.1 })
      .diffPixels,
    1,
  );
});

test("visual diff budget can cap absolute pixels for narrow long pages", () => {
  assert.equal(
    visualDiffWithinBudget(
      { diffPixels: 41_240, diffRatio: 0.007987279231330897 },
      { maxDiffPixels: 45_000, maxDiffRatio: 0.004 },
    ),
    true,
  );
  assert.equal(
    visualDiffWithinBudget(
      { diffPixels: 41_240, diffRatio: 0.007987279231330897 },
      { maxDiffPixels: 40_000, maxDiffRatio: 0.004 },
    ),
    false,
  );
});

test("visual stability styles adopt a stylesheet instead of injecting a style tag", (t) => {
  const dom = installFakeDom(t, { adoptedStyleSheets: true });

  applyVisualStabilityStyles(STABILITY_STYLE_OPTIONS);
  applyVisualStabilityStyles(STABILITY_STYLE_OPTIONS);

  const adopted = dom.document.adoptedStyleSheets ?? [];
  assert.equal(adopted.length, 1);
  assert.equal(Reflect.get(dom.window, STABILITY_STYLE_OPTIONS.sheetKey), adopted[0]);
  assert.match(adopted[0]!.cssText, /animation-duration: 0s !important/);
  assert.match(adopted[0]!.cssText, /transition-duration: 0s !important/);
  assert.deepEqual(dom.createdStyleElements, []);
});

test("visual stability styles reuse one style element when stylesheets cannot be adopted", (t) => {
  const dom = installFakeDom(t, { adoptedStyleSheets: false });

  applyVisualStabilityStyles(STABILITY_STYLE_OPTIONS);
  applyVisualStabilityStyles(STABILITY_STYLE_OPTIONS);

  assert.equal(dom.createdStyleElements.length, 1);
  assert.equal(dom.createdStyleElements[0]!.id, STABILITY_STYLE_OPTIONS.styleId);
  assert.match(dom.createdStyleElements[0]!.textContent, /animation-duration: 0s !important/);
  assert.match(dom.createdStyleElements[0]!.textContent, /transition-duration: 0s !important/);
});

class FakeCSSStyleSheet {
  cssText = "";

  replaceSync(css: string): void {
    this.cssText = css;
  }
}

class FakeHTMLStyleElement {
  id = "";
  textContent = "";
}

interface FakeDocument {
  adoptedStyleSheets?: FakeCSSStyleSheet[];
  createElement(tagName: string): FakeHTMLStyleElement;
  getElementById(id: string): FakeHTMLStyleElement | null;
  head: { append(element: FakeHTMLStyleElement): void };
}

interface FakeDom {
  createdStyleElements: FakeHTMLStyleElement[];
  document: FakeDocument;
  window: Record<string, unknown>;
}

function installFakeDom(t: TestContext, options: { adoptedStyleSheets: boolean }): FakeDom {
  const createdStyleElements: FakeHTMLStyleElement[] = [];
  const elementsById = new Map<string, FakeHTMLStyleElement>();
  const document: FakeDocument = {
    createElement(tagName: string) {
      assert.equal(tagName, "style");
      const element = new FakeHTMLStyleElement();
      createdStyleElements.push(element);
      return element;
    },
    getElementById: (id) => elementsById.get(id) ?? null,
    head: {
      append: (element) => {
        elementsById.set(element.id, element);
      },
    },
  };
  if (options.adoptedStyleSheets) {
    document.adoptedStyleSheets = [];
  }

  const dom: FakeDom = { createdStyleElements, document, window: {} };
  const globals = globalThis as unknown as Record<string, unknown>;
  const restore = new Map<string, unknown>(
    ["CSSStyleSheet", "HTMLStyleElement", "document", "window"].map((key) => [key, globals[key]]),
  );

  globals.CSSStyleSheet = FakeCSSStyleSheet;
  globals.HTMLStyleElement = FakeHTMLStyleElement;
  globals.document = dom.document;
  globals.window = dom.window;
  t.after(() => {
    for (const [key, value] of restore) {
      if (value === undefined) {
        delete globals[key];
        continue;
      }
      globals[key] = value;
    }
  });

  return dom;
}

function solidPng([red, green, blue, alpha]: [number, number, number, number]): Buffer {
  const png = new PNG({ height: 1, width: 1 });
  png.data[0] = red;
  png.data[1] = green;
  png.data[2] = blue;
  png.data[3] = alpha;
  return PNG.sync.write(png);
}
