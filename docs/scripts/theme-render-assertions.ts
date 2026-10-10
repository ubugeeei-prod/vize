import assert from "node:assert/strict";
import { createRequire } from "node:module";
import type { Page } from "playwright";

const require = createRequire(import.meta.url);
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi") as {
  transform(
    source: string,
    options: { codeAnnotations: boolean },
  ): { html: string; errors: unknown[] };
};
const annotationSource =
  'const label: string = "Readable";\n// Comment outside the focused line\nconst ready = true;';
const annotation = native.transform('```ts annotate="focus:1"\n' + annotationSource + "\n```", {
  codeAnnotations: true,
});
assert.deepEqual(annotation.errors, []);

export async function verifySidebarMotion(page: Page, device: string) {
  const sidebar = page.locator(".sidebar");
  if (!(await sidebar.count()) || (await page.locator("body.entry-page").count())) return;
  await page.emulateMedia({ reducedMotion: "no-preference" });
  if (device === "mobile") {
    const control = page.locator("[data-mobile-menu]");
    assert.equal(await control.getAttribute("aria-expanded"), "false");
    assert.equal(await sidebar.evaluate((element) => (element as HTMLElement).inert), true);
    await control.click();
    await page.waitForFunction(
      () => document.querySelector("[data-mobile-menu]")?.getAttribute("aria-expanded") === "true",
    );
    assert.equal(await sidebar.evaluate((element) => (element as HTMLElement).inert), false);
    assert.match(
      await sidebar.evaluate((element) => getComputedStyle(element).transitionDuration),
      /0\.18s/,
    );
    await page.locator(".sidebar summary").first().focus();
    await page.keyboard.press("Escape");
    await page.waitForFunction(
      () => document.querySelector("[data-mobile-menu]")?.getAttribute("aria-expanded") === "false",
    );
    assert.equal(await control.evaluate((element) => element === document.activeElement), true);
    assert.equal(await sidebar.evaluate((element) => (element as HTMLElement).inert), true);
    await sidebar.waitFor({ state: "hidden" });
    await control.click();
    await sidebar.waitFor({ state: "visible" });
  }
  const collapsed = page.locator(".sidebar details:not([open]) > summary").first();
  if (await collapsed.count()) {
    await collapsed.evaluate((element) => element.setAttribute("data-render-motion-control", ""));
    const summary = page.locator("[data-render-motion-control]");
    await summary.focus();
    // Measure intermediate rendered geometry, rather than merely checking CSS text.
    const frames = await summary.evaluate(async (element) => {
      const section = element.parentElement as HTMLDetailsElement;
      const list = section.querySelector("ul")!;
      element.dispatchEvent(new MouseEvent("click", { bubbles: true }));
      await new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      );
      const intermediate = section.getBoundingClientRect().height;
      const animations = section.getAnimations({ subtree: true });
      await Promise.all(animations.map((animation) => animation.finished));
      return {
        intermediate,
        expanded: section.getBoundingClientRect().height,
        listHeight: list.getBoundingClientRect().height,
        listScrollHeight: list.scrollHeight,
        animations: animations.length,
        supported: CSS.supports("interpolate-size", "allow-keywords"),
        open: section.open,
      };
    });
    assert.equal(frames.open, true);
    if (frames.supported) {
      assert(frames.animations > 0, "Native disclosure must animate");
      assert(frames.intermediate < frames.expanded, "Disclosure must interpolate its height");
    }
    assert(frames.listHeight > 0, "Expanded links must have visible geometry");
    assert(frames.listHeight + 1 >= frames.listScrollHeight, "Expanded links must not be clipped");
    await page.keyboard.press("Enter");
    assert.equal(
      await summary.evaluate((element) => (element.parentElement as HTMLDetailsElement).open),
      false,
    );
    assert.equal(await summary.evaluate((element) => element === document.activeElement), true);
    await page.keyboard.press("Tab");
    assert.equal(
      await summary.evaluate((element) =>
        element.parentElement!.querySelector("ul")!.contains(document.activeElement),
      ),
      false,
      "Closing disclosure links must leave the keyboard sequence immediately",
    );
    await summary.focus();
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.keyboard.press("Enter");
    const reduced = await summary.evaluate((element) => ({
      open: (element.parentElement as HTMLDetailsElement).open,
      animations: element.parentElement!.getAnimations({ subtree: true }).length,
    }));
    assert.equal(reduced.open, true);
    assert.equal(reduced.animations, 0, "Reduced motion must skip disclosure animation");
    await page.keyboard.press("Enter");
  }
  await page.emulateMedia({ reducedMotion: "reduce" });
  if (device === "mobile") {
    await page.locator("[data-mobile-menu]").click();
    await sidebar.waitFor({ state: "hidden" });
    assert.equal(
      await sidebar.evaluate((element) => getComputedStyle(element).transitionDuration),
      "0s",
    );
  }
}

export async function verifyThemeReadability(page: Page) {
  // A native focus annotation exercises dimmed comments even when authored
  // guides on this route contain only plain or add/remove code examples.
  const fixture = await page.evaluate(
    ({ html, source }) => {
      const content = document.querySelector(".content");
      if (!content) return false;
      const container = document.createElement("div");
      container.setAttribute("data-render-annotation-control", "");
      container.innerHTML = html;
      content.append(container);
      (
        globalThis as typeof globalThis & {
          __vizeDocsSyntax: { highlightAll(root: Element): void };
        }
      ).__vizeDocsSyntax.highlightAll(container);
      if (container.querySelector("code")?.textContent !== source + "\n")
        throw new Error("Native annotation source changed during highlighting");
      if (!container.querySelector(".ox-code-line--dimmed"))
        throw new Error("Native annotation control did not emit dimmed lines");
      return true;
    },
    { html: annotation.html, source: annotationSource },
  );
  await page.evaluate(async () => {
    await Promise.all(
      document
        .getAnimations()
        .filter((animation) => animation.effect?.getComputedTiming().iterations !== Infinity)
        .map((animation) => animation.finished),
    );
  });
  const contrast = await page.evaluate(() => {
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 1;
    const context = canvas.getContext("2d")!;
    const rgba = (color: string) => {
      context.clearRect(0, 0, 1, 1);
      context.fillStyle = color;
      context.fillRect(0, 0, 1, 1);
      return [...context.getImageData(0, 0, 1, 1).data];
    };
    const composite = (front: number[], back: number[]) => {
      const alpha = front[3] / 255;
      return front
        .slice(0, 3)
        .map((channel, index) => channel * alpha + back[index] * (1 - alpha))
        .concat(255);
    };
    const luminance = (rgb: number[]) => {
      const channels = rgb.slice(0, 3).map((channel) => {
        const value = channel / 255;
        return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
      });
      return channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722;
    };
    const seen = new Set<string>();
    return [
      ...document.querySelectorAll(
        ".content pre code, .content pre .v-code__token, .content pre .ox-code-line",
      ),
    ]
      .map((element) => {
        const style = getComputedStyle(element);
        const parents: Element[] = [];
        for (let parent: Element | null = element; parent; parent = parent.parentElement)
          parents.unshift(parent);
        let background = [255, 255, 255, 255];
        for (const parent of parents)
          background = composite(rgba(getComputedStyle(parent).backgroundColor), background);
        const foreground = composite(rgba(style.color), background);
        const a = luminance(foreground),
          b = luminance(background);
        return {
          kind: element.className || "code",
          color: style.color,
          background,
          ratio: (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05),
          opacity: style.opacity,
          filter: style.filter,
        };
      })
      .filter((token) => {
        const key = `${token.kind}:${token.color}:${token.background.join(",")}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });
  });
  if (fixture)
    await page.locator("[data-render-annotation-control]").evaluate((element) => element.remove());
  for (const token of contrast) {
    assert(
      token.ratio >= 4.5,
      `${await page.title()}: ${token.kind} ${token.color} on ${token.background.join(",")} contrast ${token.ratio.toFixed(2)} < 4.5`,
    );
    if (token.kind.includes("ox-code-line--dimmed")) {
      assert.equal(token.opacity, "1");
      assert.equal(token.filter, "none");
    }
  }
  return contrast;
}

export async function verifyIntroductoryNavigation(page: Page, route: string) {
  const locale = route.match(/^\/(ja|zh-CN|pt-BR|fr)(?:\/|$)/)?.[1] ?? "en";
  const prefix = locale === "en" ? "" : `/${locale}`;
  const groups = await page.locator(".sidebar details").evaluateAll((elements) =>
    elements.map((element) => ({
      paths: [...element.querySelectorAll("a")].map((link) =>
        new URL(link.href).pathname.replace(/\/index\.html$|\/$/g, ""),
      ),
    })),
  );
  if (!groups.length) return;
  const philosophy = `${prefix}/philosophy`;
  const mapper = `${prefix}/guide/content-mapper`;
  assert(groups[0].paths.includes(philosophy), `${locale}: Philosophy belongs in Start`);
  assert.equal(groups.filter((group) => group.paths.includes(philosophy)).length, 1);
  const analysis = groups.find((group) => group.paths.includes(`${prefix}/guide/static-analysis`));
  assert(analysis?.paths.includes(mapper), `${locale}: Content Mapper belongs in static analysis`);
  assert.equal(groups.filter((group) => group.paths.includes(mapper)).length, 1);
}

export async function switchDocsTheme(page: Page, theme: string, device: string) {
  if ((await page.locator("html").getAttribute("data-theme")) === theme) return;
  const entry = device === "desktop" && (await page.locator("body.entry-page").count()) > 0;
  if (entry) {
    await page.evaluate(() => scrollTo({ top: 120, behavior: "instant" }));
    await page.locator(".header.header-visible").waitFor();
  }
  await page.locator(device === "mobile" ? "[data-mobile-theme]" : ".theme-toggle").click();
  if (entry) await page.evaluate(() => scrollTo({ top: 0, behavior: "instant" }));
}
