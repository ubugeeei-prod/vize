import assert from "node:assert/strict";
import type { Page } from "playwright";

const languages = [
  ["en", "English"],
  ["ja", "日本語"],
  ["zh-CN", "简体中文"],
  ["pt-BR", "Português"],
  ["fr", "Français"],
];

/** Exercise the shipped disclosure and native links on an actual rendered page. */
export async function verifyLocaleDropdown(page: Page) {
  const button = page.locator(".docs-locale-select");
  const options = page.locator(".docs-locale-options");
  const links = page.locator(".docs-locale-option");
  const current = new URL(page.url());
  const first = current.pathname.split("/").find(Boolean);
  const locale = languages.some(([code]) => code === first) ? first : "en";
  const logical = locale === "en" ? current.pathname : current.pathname.slice(locale!.length + 1);
  const canonical = logical.replace(/\/index\.html$/, "").replace(/\/$/, "");
  const hrefs = languages.map(
    ([code]) =>
      new URL(
        `${code === "en" ? "" : `/${code}`}${canonical}/index.html${current.search}${current.hash}`,
        current.origin,
      ).href,
  );
  assert.equal(await button.count(), 1, "exactly one language disclosure");
  assert.equal(await button.getAttribute("aria-controls"), "docs-locale-options");
  assert.equal(await button.getAttribute("aria-expanded"), "false");
  assert.equal(await options.isVisible(), false, "options start closed");
  const packet = await links.evaluateAll((elements) =>
    elements.map((element) => {
      if (!(element instanceof HTMLAnchorElement)) throw new Error("Language target is not a link");
      return {
        code: element.dataset.locale,
        name: element.firstElementChild?.textContent,
        href: element.href,
        lang: element.lang,
        hreflang: element.hreflang,
        current: element.getAttribute("aria-current"),
      };
    }),
  );
  assert.deepEqual(
    packet,
    languages.map(([code, name], index) => ({
      code,
      name,
      href: hrefs[index],
      lang: code,
      hreflang: code,
      current: code === locale ? "page" : null,
    })),
    "all five unchanged language labels and whole hrefs, including query and hash",
  );
  const focused = () =>
    page.evaluate(() => (document.activeElement as HTMLElement | null)?.dataset.locale ?? null);
  const steps: { key: string; focused: string | null }[] = [];
  for (const [key, expected] of [
    ["ArrowDown", "en"],
    ["ArrowUp", "fr"],
    ["Home", "en"],
    ["End", "fr"],
    ["ArrowDown", "en"],
  ]) {
    if (!steps.length) await button.press(key);
    else await page.keyboard.press(key);
    const actual = await focused();
    assert.equal(actual, expected, `focus after ${key}`);
    steps.push({ key, focused: actual });
  }
  assert.equal(await options.isVisible(), true);
  await page.evaluate(() => {
    const probe = { event: null as KeyboardEvent | null, bubbles: 0 };
    const capture = (event: KeyboardEvent) => {
      if (event.key === "Escape") probe.event = event;
    };
    const bubble = (event: KeyboardEvent) => {
      if (event.key === "Escape") probe.bubbles++;
    };
    document.addEventListener("keydown", capture, true);
    document.addEventListener("keydown", bubble);
    (
      globalThis as typeof globalThis & { __localeEscapeProbe?: () => unknown }
    ).__localeEscapeProbe = () => {
      document.removeEventListener("keydown", capture, true);
      document.removeEventListener("keydown", bubble);
      return { prevented: probe.event?.defaultPrevented, bubbles: probe.bubbles };
    };
  });
  await page.keyboard.press("Escape");
  const escape = await page.evaluate(() => {
    const scope = globalThis as typeof globalThis & { __localeEscapeProbe?: () => unknown };
    const result = scope.__localeEscapeProbe?.();
    delete scope.__localeEscapeProbe;
    return result;
  });
  assert.deepEqual(escape, { prevented: true, bubbles: 0 }, "own Escape is contained");
  assert.equal(await button.evaluate((element) => element === document.activeElement), true);
  assert.equal(await options.isVisible(), false);
  await button.press("Enter");
  assert.equal(await focused(), locale, "keyboard opening focuses the selected link");
  await page.keyboard.press("End");
  await page.keyboard.press("Tab");
  assert.equal(await options.isVisible(), false, "Tab dismisses after the final native link");
  const nextFocusable = (await page.locator(".search-button").isVisible())
    ? page.locator(".search-button")
    : page.locator(".content a[href]").first();
  assert.equal(
    await nextFocusable.evaluate((element) => element === document.activeElement),
    true,
    "Tab follows the native header focus order",
  );
  await button.click();
  await page
    .locator("h1")
    .first()
    .click({ position: { x: 8, y: 8 } });
  assert.equal(await options.isVisible(), false, "outside click dismisses");
  await button.click();
  const themeControl = (await page.locator(".theme-toggle").isVisible())
    ? page.locator(".theme-toggle")
    : page.locator("[data-mobile-theme]");
  await themeControl.focus();
  assert.equal(await options.isVisible(), false, "outside focus dismisses");
  assert.equal(
    await themeControl.evaluate((element) => element === document.activeElement),
    true,
    "outside dismissal does not steal focus",
  );

  const lifecycle = await page.evaluate(async () => {
    const navigation = (
      globalThis as typeof globalThis & {
        __vizeDocsNavigation?: { initialize: (root: Document) => void };
      }
    ).__vizeDocsNavigation;
    if (!navigation) throw new Error("The shipped navigation module is missing");
    const oldWrapper = document.querySelector(".docs-locale");
    const oldButton = oldWrapper?.querySelector("button");
    const header = document.querySelector(".header-actions");
    if (!oldWrapper || !oldButton || !header)
      throw new Error("Mounted language control is missing");
    navigation.initialize(document);
    navigation.initialize(document);
    const repeated = {
      controls: document.querySelectorAll(".docs-locale").length,
      same: document.querySelector(".docs-locale") === oldWrapper,
    };
    const replacement = header.cloneNode(true) as HTMLElement;
    replacement.querySelector(".docs-locale")?.remove();
    header.replaceWith(replacement);
    navigation.initialize(document);
    oldButton.click();
    const replaced = {
      controls: document.querySelectorAll(".docs-locale").length,
      oldConnected: oldWrapper.isConnected,
      oldExpanded: oldButton.getAttribute("aria-expanded"),
    };
    const nextWrapper = document.querySelector(".docs-locale");
    const nextButton = nextWrapper?.querySelector("button");
    if (!nextWrapper || !nextButton) throw new Error("Replacement control did not mount");
    nextWrapper.remove();
    await new Promise<void>((resolve) => queueMicrotask(resolve));
    nextButton.click();
    const detached = {
      controls: document.querySelectorAll(".docs-locale").length,
      oldExpanded: nextButton.getAttribute("aria-expanded"),
    };
    navigation.initialize(document);
    return {
      repeated,
      replaced,
      detached,
      finalControls: document.querySelectorAll(".docs-locale").length,
    };
  });
  assert.deepEqual(lifecycle, {
    repeated: { controls: 1, same: true },
    replaced: { controls: 1, oldConnected: false, oldExpanded: "false" },
    detached: { controls: 0, oldExpanded: "false" },
    finalControls: 1,
  });
  return { url: page.url(), packet, steps, escape, lifecycle };
}

/** Measure the actual open panel rather than inferring spacing from its stylesheet. */
export async function measureLocaleDropdown(page: Page) {
  const metrics = await page.evaluate(() => {
    const button = document.querySelector(".docs-locale-select");
    const icon = document.querySelector(".docs-locale-chevron");
    const options = document.querySelector(".docs-locale-options");
    if (!button || !icon || !(options instanceof HTMLElement) || options.hidden)
      throw new Error("The measured language panel is not open");
    const rect = (element: Element) => element.getBoundingClientRect().toJSON();
    return {
      viewport: innerWidth,
      bodyWidth: document.documentElement.scrollWidth,
      theme: document.documentElement.getAttribute("data-theme"),
      paddingRight: getComputedStyle(button).paddingRight,
      iconRightGap: button.getBoundingClientRect().right - icon.getBoundingClientRect().right,
      button: rect(button),
      panel: rect(options),
      options: [...options.querySelectorAll("a")].map((link) => ({
        locale: link.dataset.locale,
        current: link.getAttribute("aria-current"),
        focused: link === document.activeElement,
        hovered: link.matches(":hover"),
        focusOutline: getComputedStyle(link).outlineStyle,
        background: getComputedStyle(link).backgroundColor,
        check: getComputedStyle(link.querySelector(".docs-locale-check")!).visibility,
        bounds: rect(link),
      })),
    };
  });
  assert(metrics.button.height >= 44, "trigger has a 44px touch target");
  assert(metrics.iconRightGap >= 12, "chevron has at least 12px of measured right gap");
  assert(metrics.panel.left >= 0 && metrics.panel.right <= metrics.viewport, "panel fits viewport");
  assert(
    metrics.bodyWidth <= metrics.viewport,
    "language control introduces no horizontal overflow",
  );
  for (const option of metrics.options) {
    assert(option.bounds.height >= 44, `${option.locale}: 44px option target`);
    assert.equal(option.check, option.current ? "visible" : "hidden", "selected check is explicit");
    if (option.focused) assert.equal(option.focusOutline, "solid", "keyboard focus is visible");
  }
  return metrics;
}

/** Preserve live URL state, including native modified-click and a reused SPA mount. */
export async function verifyLiveLocaleLinks(page: Page) {
  const original = page.url();
  const button = page.locator(".docs-locale-select");
  const options = page.locator(".docs-locale-options");
  const links = page.locator(".docs-locale-option");
  const mounted = await page.locator(".docs-locale").elementHandle();
  assert(mounted, "live link check starts with the actual mounted control");
  const packets: { url: string; hrefs: string[] }[] = [];
  const collect = async () => {
    const url = new URL(page.url());
    const logical = url.pathname.replace(/^\/(?:ja|zh-CN|pt-BR|fr)(?=\/)/, "");
    const hrefs = await links.evaluateAll((elements) =>
      elements.map((element) => (element as HTMLAnchorElement).href),
    );
    assert.deepEqual(
      hrefs,
      languages.map(
        ([code]) =>
          new URL(
            `${code === "en" ? "" : `/${code}`}${logical}${url.search}${url.hash}`,
            url.origin,
          ).href,
      ),
      "all native language hrefs use current URL state",
    );
    packets.push({ url: page.url(), hrefs });
    return hrefs;
  };
  await page.evaluate(() =>
    history.replaceState(null, "", `${location.pathname}?locale-check=after%20mount#after-mount`),
  );
  await button.click();
  await collect();
  const anchor = page.locator(".toc-link").last();
  let anchorPacket: { source: string; href: string } | null = null;
  if (await anchor.count()) {
    anchorPacket = await anchor.evaluate((element) => {
      if (!(element instanceof HTMLAnchorElement))
        throw new Error("Authored TOC anchor is missing");
      const packet = { source: element.outerHTML, href: element.href };
      element.click();
      return packet;
    });
    const hash = new URL(anchorPacket.href).hash;
    await page.waitForURL((url) => url.hash === hash);
  } else
    await page.evaluate(() => {
      location.hash = "after-anchor";
    });
  if (await options.isVisible()) await button.press("Escape");
  await button.click();
  await collect();
  await page.evaluate(() =>
    history.replaceState(null, "", `${location.pathname}?locale-check=before%20click#native-link`),
  );
  const popupPromise = page.context().waitForEvent("page");
  await links.nth(1).click({ modifiers: ["ControlOrMeta"] });
  const popup = await popupPromise;
  await popup.waitForLoadState("domcontentloaded");
  const nativeTarget = popup.url();
  const hrefs = await collect();
  assert.equal(nativeTarget, hrefs[1], "modified click navigates to the current query and hash");
  await popup.close();
  const spa = await page.evaluate(() => {
    history.pushState(null, "", "/ja/getting-started/index.html?locale-check=spa#selected");
    const navigation = (
      globalThis as typeof globalThis & {
        __vizeDocsNavigation?: { initialize: (root: Document) => void };
      }
    ).__vizeDocsNavigation;
    if (!navigation) throw new Error("Shipped SPA initializer is missing");
    navigation.initialize(document);
    return {
      name: document.querySelector(".docs-locale-name")?.textContent,
      current: [...document.querySelectorAll('.docs-locale-option[aria-current="page"]')].map(
        (element) => (element as HTMLElement).dataset.locale,
      ),
      controls: document.querySelectorAll(".docs-locale").length,
    };
  });
  assert.deepEqual(spa, { name: "日本語", current: ["ja"], controls: 1 });
  assert.equal(
    await mounted.evaluate((element) => element === document.querySelector(".docs-locale")),
    true,
    "SPA reuses the mount",
  );
  assert.equal(await options.isVisible(), false, "SPA language selection closes the disclosure");
  await collect();
  await page.evaluate((url) => {
    history.replaceState(null, "", url);
    (
      globalThis as typeof globalThis & {
        __vizeDocsNavigation: { initialize: (root: Document) => void };
      }
    ).__vizeDocsNavigation.initialize(document);
  }, original);
  await mounted.dispose();
  return {
    packets,
    anchorPacket,
    anchorActivation: "native authored anchor.click",
    nativeTarget,
    spa,
  };
}
