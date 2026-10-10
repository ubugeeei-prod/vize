import assert from "node:assert/strict";
import type { Page } from "playwright";

/** Observe the real completed gallery pane; temporary scope probes never enter captures. */
export async function assertHostedVrtStyles(page: Page) {
  const receipt = await page.locator(".vrt-panel").evaluate((panel) => {
    const rem = Number.parseFloat(getComputedStyle(document.documentElement).fontSize);
    const properties = [
      "display",
      "flex-direction",
      "align-items",
      "gap",
      "padding-top",
      "padding-left",
      "border-top-width",
      "border-top-style",
      "font-size",
      "font-weight",
      "margin-bottom",
      "color",
      "white-space",
      "overflow-wrap",
      "max-width",
      "height",
    ];
    const read = (element: Element) => {
      const computed = getComputedStyle(element);
      return Object.fromEntries(properties.map((name) => [name, computed.getPropertyValue(name)]));
    };
    const find = (selector: string) => {
      const element = panel.querySelector(selector);
      if (!element) throw new Error(`Missing authored VRT element: ${selector}`);
      return element;
    };
    const scopedAttributes = (element: Element) =>
      [...element.attributes]
        .map((attribute) => attribute.name)
        .filter((name) => name.startsWith("data-v-"));
    const selectors: string[] = [];
    const visit = (rules: CSSRuleList) => {
      for (const rule of rules) {
        if (rule instanceof CSSStyleRule) selectors.push(rule.selectorText);
        if (rule instanceof CSSGroupingRule) visit(rule.cssRules);
      }
    };
    for (const sheet of document.styleSheets) {
      if (sheet.href && new URL(sheet.href).origin !== location.origin) continue;
      visit(sheet.cssRules);
    }
    const probe = document.createElement("div");
    const outside: Record<
      string,
      { before: Record<string, string>; after: Record<string, string> }
    > = {};
    document.body.append(probe);
    try {
      for (const className of [
        "vrt-panel",
        "vrt-results",
        "vrt-variant",
        "vrt-status pass",
        "vrt-hosted-image",
      ]) {
        probe.className = "";
        const before = read(probe);
        probe.className = className;
        outside[className] = { before, after: read(probe) };
      }
    } finally {
      probe.remove();
    }
    const results = find(".vrt-results");
    return {
      rem,
      selectors,
      parentScopes: scopedAttributes(panel),
      resultScopes: scopedAttributes(results),
      panel: read(panel),
      header: read(find(".vrt-header")),
      summary: read(find(".vrt-summary")),
      stat: read(find(".vrt-stat.total")),
      results: read(results),
      variants: [...results.querySelectorAll(".vrt-variant")].map((variant) => ({
        name: variant.querySelector(".vrt-variant-name")!.textContent,
        style: read(variant),
        label: read(variant.querySelector(".vrt-variant-name")!),
      })),
      viewports: [...results.querySelectorAll(".vrt-viewport")].map(read),
      bodies: [...results.querySelectorAll(".vrt-viewport-body")].map(read),
      statuses: [...results.querySelectorAll(".vrt-status")].map((element) => ({
        style: read(element),
        expectedColor: read(
          find(`.vrt-stat.${element.classList.contains("new") ? "new" : "passed"} .vrt-stat-value`),
        ).color,
      })),
      paths: [...results.querySelectorAll(".vrt-result-path")].map(read),
      images: [...results.querySelectorAll<HTMLImageElement>("img")].map((image) => ({
        src: image.src,
        complete: image.complete,
        natural: [image.naturalWidth, image.naturalHeight],
        bounds: [image.getBoundingClientRect().width, image.getBoundingClientRect().height],
        style: read(image),
        scopes: scopedAttributes(image),
      })),
      outside,
    };
  });
  const px = (factor: number) => `${receipt.rem * factor}px`;
  assert.equal(receipt.panel["padding-top"], px(0.5));
  assert.equal(receipt.header.display, "flex");
  assert.equal(receipt.header["align-items"], "center");
  assert.equal(receipt.summary.display, "flex");
  assert.equal(receipt.summary.gap, px(0.75));
  assert.equal(receipt.stat["padding-left"], px(0.75));
  assert.equal(receipt.stat["border-top-width"], "1px");
  assert.equal(receipt.results.display, "flex");
  assert.equal(receipt.results["flex-direction"], "column");
  assert.equal(receipt.results.gap, px(0.5));
  assert.ok(receipt.variants.length > 0);
  for (const item of receipt.variants) {
    assert.equal(item.style["padding-top"], px(0.75));
    assert.equal(item.style["border-top-width"], "1px");
    assert.equal(item.style["border-top-style"], "solid");
    assert.equal(item.label["font-weight"], "600");
    assert.equal(item.label["font-size"], px(0.8125));
    assert.equal(item.label["margin-bottom"], px(0.5));
  }
  for (const item of receipt.viewports) {
    assert.equal(item.display, "flex");
    assert.equal(item["align-items"], "flex-start");
    assert.equal(item.gap, px(0.5));
    assert.equal(item["padding-top"], px(0.25));
    assert.equal(item["padding-left"], px(0.5));
  }
  for (const item of receipt.bodies) {
    assert.equal(item.display, "grid");
    assert.equal(item.gap, px(0.25));
  }
  for (const item of receipt.statuses) {
    assert.equal(item.style["font-weight"], "600");
    assert.equal(item.style["font-size"], px(0.6875));
    assert.equal(item.style.color, item.expectedColor);
  }
  for (const item of receipt.paths) {
    assert.equal(item["font-size"], px(0.6875));
    assert.equal(item["white-space"], "pre-wrap");
    assert.equal(item["overflow-wrap"], "anywhere");
  }
  assert.ok(receipt.images.length > 0);
  assert.equal(new Set(receipt.images.map((image) => image.src)).size, receipt.images.length);
  for (const image of receipt.images) {
    assert.ok(image.complete && image.src.startsWith("blob:"));
    assert.deepEqual(image.natural, [320, 180]);
    assert.deepEqual(image.bounds, [240, 135]);
    assert.equal(image.style["max-width"], "240px");
    assert.equal(image.style.height, "135px");
  }
  for (const item of Object.values(receipt.outside)) assert.deepEqual(item.after, item.before);
  for (const [className, scopes] of [
    [".vrt-panel", receipt.parentScopes],
    [".vrt-results", receipt.resultScopes],
    [".vrt-status", receipt.resultScopes],
    [".vrt-hosted-image", receipt.images[0].scopes],
  ] as const) {
    assert.ok(scopes.length > 0);
    assert.ok(
      receipt.selectors.some(
        (selector) =>
          selector.includes(className) && scopes.some((scope) => selector.includes(`[${scope}]`)),
      ),
    );
  }
  return receipt;
}
