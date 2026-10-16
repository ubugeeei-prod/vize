import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import type { Locator, Page } from "playwright";

async function pointerGeometry(control: Locator) {
  return control.evaluate((element) => {
    const r = element.getBoundingClientRect();
    const points = [
      [r.x + r.width / 2, r.y + r.height / 2],
      [r.left + 2, r.top + 2],
      [r.right - 2, r.top + 2],
      [r.left + 2, r.bottom - 2],
      [r.right - 2, r.bottom - 2],
    ];
    return {
      rect: r.toJSON(),
      points: points.map(([x, y]) => {
        const hit = document.elementFromPoint(x, y);
        return {
          x,
          y,
          owned: !!hit && element.contains(hit),
          hit: hit?.tagName,
          className: hit?.className,
        };
      }),
    };
  });
}

/** Exercise native pointer targets in the loaded site; never inject layout fixes. */
export async function verifyTocPointerLayer(page: Page, device: string, output: string) {
  const pageErrors: string[] = [];
  const onError = (error: Error) => pageErrors.push(String(error));
  page.on("pageerror", onError);
  const viewport = page.viewportSize();
  assert(viewport);
  const original = await page.evaluate(() => ({
    url: location.href,
    x: scrollX,
    y: scrollY,
    mainY: document.querySelector(".main")!.scrollTop,
  }));
  const originalFocus = await page.evaluateHandle(() => document.activeElement);
  const directory = path.join(output, "toc-controls");
  await mkdir(directory, { recursive: true });
  try {
    if (device === "desktop") await page.setViewportSize({ width: 1600, height: 1000 });
    await page.evaluate(() => scrollTo(0, 0));
    const layers = await page.evaluate(() => {
      const layer = (selector: string) => {
        const element = document.querySelector(selector)!;
        const css = getComputedStyle(element);
        return { position: css.position, zIndex: css.zIndex, display: css.display };
      };
      return {
        main: layer(".main"),
        toc: layer(".toc"),
        header: layer(".header"),
        sidebar: layer(".sidebar"),
      };
    });
    const toc = page.locator(".toc");
    const targets = [];
    let clickedHash: string | null = null;
    if (device === "desktop") {
      assert.equal(await toc.isVisible(), true, "Wide native outline is visible");
      const links = toc.locator("a[href^='#']");
      assert((await links.count()) > 0, "Authored outline has real targets");
      for (const link of await links.all()) {
        const geometry = await pointerGeometry(link);
        assert(
          geometry.points.every((point) => point.owned),
          `TOC is covered: ${JSON.stringify({ layers, geometry })}`,
        );
        targets.push(geometry);
      }
      const link = links.first();
      const hash = await link.getAttribute("href");
      assert(hash?.startsWith("#"));
      await link.click();
      await page.waitForFunction(
        (value) => decodeURIComponent(location.hash) === decodeURIComponent(value),
        hash,
      );
      await page.waitForFunction((value) => {
        const top = document
          .getElementById(decodeURIComponent(value.slice(1)))!
          .getBoundingClientRect().top;
        return top >= -1 && top < 200;
      }, hash);
      clickedHash = await page.evaluate(() => location.hash);
      await link.focus();
      assert(
        await link.evaluate((element) => element === document.activeElement),
        "Outline keeps keyboard focus",
      );
      await page.keyboard.press("Enter");
    } else {
      assert.equal(await toc.isVisible(), false, "Narrow native outline stays hidden");
    }
    const paragraph = page.locator(".content p").filter({ hasText: /\S/ }).first();
    await paragraph.scrollIntoViewIfNeeded();
    const textPoint = await paragraph.evaluate((element) => {
      const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT);
      for (let node = walker.nextNode(); node; node = walker.nextNode()) {
        if (!node.textContent?.trim() || node.parentElement?.closest("a, button")) continue;
        const range = document.createRange();
        range.selectNodeContents(node);
        const rect = [...range.getClientRects()].find((r) => r.width > 4 && r.height > 4);
        if (rect)
          return { x: rect.left + Math.min(10, rect.width / 2), y: rect.top + rect.height / 2 };
      }
      throw new Error("Authored paragraph has no selectable text");
    });
    await page.mouse.dblclick(textPoint.x, textPoint.y);
    const selection = await page.evaluate(() => getSelection()?.toString().trim());
    assert(selection, "Main text remains selectable by a real pointer");
    await page.evaluate(() => {
      getSelection()?.removeAllRanges();
      scrollTo({ top: 0, behavior: "instant" });
      document.querySelector(".main")!.scrollTo({ top: 0, behavior: "instant" });
    });
    await page.mouse.move(textPoint.x, Math.min(textPoint.y, viewport.height / 2));
    await page.mouse.wheel(0, 300);
    await page.waitForFunction(
      () => scrollY > 50 || document.querySelector(".main")!.scrollTop > 50,
    );
    const scrolled = await page.evaluate(() => ({
      windowY: scrollY,
      mainY: document.querySelector(".main")!.scrollTop,
    }));
    await page.evaluate(() => scrollTo(0, 0));
    if (device === "mobile") {
      const menu = page.locator("[data-mobile-menu]");
      await menu.click();
      const sidebar = page.locator(".sidebar");
      await sidebar.waitFor({ state: "visible" });
      assert.equal(await toc.isVisible(), false, "Hidden outline cannot overlay the mobile drawer");
      const link = sidebar.locator("a[href]:visible").first();
      const geometry = await pointerGeometry(link);
      assert(
        geometry.points.every((point) => point.owned),
        `Drawer target covered: ${JSON.stringify(geometry)}`,
      );
      await link.click({ trial: true });
      await link.focus();
      await page.screenshot({ path: path.join(directory, "mobile-drawer.png") });
      await page.keyboard.press("Escape");
      await sidebar.waitFor({ state: "hidden" });
      assert(
        await menu.evaluate((element) => element === document.activeElement),
        "Drawer returns focus to its control",
      );
      assert(
        await sidebar.evaluate((element) => (element as HTMLElement).inert),
        "Closed drawer remains inert",
      );
    }
    const screenshot = path.join(directory, `${device}.png`);
    await page.screenshot({ path: screenshot });
    assert.deepEqual(pageErrors, [], "Native outline interactions produce no runtime errors");
    return { layers, targets, clickedHash, selection, scrolled, screenshot, pageErrors };
  } finally {
    page.off("pageerror", onError);
    await page.evaluate((state) => {
      history.replaceState(null, "", state.url);
      getSelection()?.removeAllRanges();
    }, original);
    await page.setViewportSize(viewport);
    await page.evaluate(({ x, y, mainY }) => {
      scrollTo({ left: x, top: y, behavior: "instant" });
      document.querySelector(".main")!.scrollTo({ top: mainY, behavior: "instant" });
      if (document.activeElement instanceof HTMLElement) document.activeElement.blur();
    }, original);
    await originalFocus.evaluate((element) => {
      if (element instanceof HTMLElement && element.isConnected)
        element.focus({ preventScroll: true });
    });
    await originalFocus.dispose();
  }
}
