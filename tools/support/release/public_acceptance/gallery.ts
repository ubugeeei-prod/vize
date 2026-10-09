import assert from "node:assert/strict";
import type { Browser, Page } from "playwright";

export async function frames(
  page: Page,
  values: { brand: string; scheme: string; locale: string },
  art = "Button",
) {
  await page.waitForFunction(
    ({ expected, selector }) => {
      const elements = [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")];
      return (
        elements.length >= 2 &&
        elements.every((frame) => {
          const root = frame.contentDocument?.documentElement;
          return (
            root?.dataset.brand === expected.brand &&
            root.dataset.scheme === expected.scheme &&
            root.lang === expected.locale &&
            frame.contentDocument?.querySelector(selector)
          );
        })
      );
    },
    { expected: values, selector: art === "Button" ? "button.btn" : ".alert" },
  );
  const result = await page.evaluate(() =>
    [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map((frame) => ({
      url: frame.src,
      brand: frame.contentDocument!.documentElement.dataset.brand,
      scheme: frame.contentDocument!.documentElement.dataset.scheme,
      locale: frame.contentDocument!.documentElement.lang,
      accent: frame.contentDocument!.documentElement.style.getPropertyValue("--musea-accent"),
      paper: frame.contentDocument!.documentElement.style.getPropertyValue("--musea-paper"),
      setupCalls: Reflect.get(frame.contentWindow!, "__publicSetupCalls"),
      documentToken: Reflect.get(frame.contentWindow!, "__publicDocumentToken"),
      firstGlobals: Reflect.get(frame.contentWindow!, "__publicFirstGlobals"),
    })),
  );
  for (const frame of result) {
    assert.equal(frame.setupCalls, 1);
    assert.match(frame.documentToken, /^[0-9a-f-]{36}$/u);
    if (values.brand === "ocean") assert.equal(frame.accent, "#176b92");
    if (values.scheme === "dark") assert.equal(frame.paper, "#1b2430");
  }
  return result;
}

export async function rememberDocuments(page: Page) {
  await page.evaluate(() => {
    Reflect.set(
      window,
      "__publicPreviewDocuments",
      [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map(
        (frame) => frame.contentDocument,
      ),
    );
  });
}

export async function documentIdentity(page: Page) {
  return page.evaluate(() => {
    const initial = Reflect.get(window, "__publicPreviewDocuments") as Document[];
    const current = [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")];
    return {
      count: current.length,
      unchanged:
        initial.length === current.length &&
        current.every((frame, index) => frame.contentDocument === initial[index]),
    };
  });
}

export async function openArt(page: Page, title: string) {
  await page.locator(".art-item").filter({ hasText: title }).first().click();
  await page.getByRole("heading", { name: title, exact: true }).waitFor();
}

export async function checkLegacy(browser: Browser, errors: string[]) {
  const { createServer } = await import("vite");
  const { default: vize } = await import("@vizejs/vite-plugin");
  const { musea } = await import("@vizejs/vite-plugin-musea");
  const legacy = await createServer({
    root: process.cwd(),
    configFile: false,
    plugins: [
      vize(),
      musea({
        include: ["src/**/*.vue"],
        inlineArt: true,
        previewSetup: "preview.one-argument.ts",
      }),
    ],
    server: { host: "127.0.0.1", port: 0 },
  });
  try {
    await legacy.listen();
    const legacyAddress = legacy.httpServer!.address();
    assert.ok(legacyAddress && typeof legacyAddress !== "string");
    const legacyContext = await browser.newContext();
    const legacyPage = await legacyContext.newPage();
    legacyPage.on("pageerror", (error) => errors.push(String(error)));
    await legacyPage.goto(`http://127.0.0.1:${legacyAddress.port}/__musea__/`);
    await openArt(legacyPage, "Button");
    await legacyPage.waitForFunction(() => {
      const elements = [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")];
      return (
        elements.length >= 2 &&
        elements.every(
          (frame) =>
            frame.contentDocument?.querySelector("button.btn") &&
            Reflect.get(frame.contentWindow!, "__publicLegacySetupCalls") === 1,
        )
      );
    });
    assert.equal(await legacyPage.locator(".global-control").count(), 0);
    const legacyCalls = await legacyPage.evaluate(() =>
      [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map((frame) =>
        Reflect.get(frame.contentWindow!, "__publicLegacyArguments"),
      ),
    );
    assert.ok(legacyCalls.every((count) => count === 1));
    await legacyContext.close();
    return { phase: "unconfigured-gallery-one-argument-hook", argumentCounts: legacyCalls };
  } finally {
    await legacy.close();
  }
}
