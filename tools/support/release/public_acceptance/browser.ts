import assert from "node:assert/strict";
import { mkdirSync, writeFileSync } from "node:fs";
import { createServer as createHttpServer } from "node:http";
import path from "node:path";
import { publicNative, rejectOverrides } from "./installed.ts";
import { frames, openArt, checkLegacy, rememberDocuments, documentIdentity } from "./gallery.ts";

rejectOverrides();
const [version, output] = process.argv.slice(2);
assert.ok(version && output);
const { createServer } = await import("vite");
const { default: vize } = await import("@vizejs/vite-plugin");
const { musea } = await import("@vizejs/vite-plugin-musea");
const { chromium } = await import("playwright");
const { stringifyQuery } = await import("vue-router");
const defaults = { brand: "default", scheme: "light", locale: "en" };
const changed = { brand: "ocean", scheme: "dark", locale: "ja" };
const toolbar = [
  {
    id: "brand",
    title: "Brand",
    type: "select" as const,
    options: ["default", "ocean"],
    default: "default",
  },
  {
    id: "scheme",
    title: "Component theme",
    type: "toggle" as const,
    default: "light",
    options: [
      { value: "light", label: "Light" },
      { value: "dark", label: "Dark" },
    ],
  },
  { id: "locale", title: "Locale", type: "select" as const, options: ["en", "ja"], default: "en" },
];
const observations: unknown[] = [];
const errors: string[] = [];
const browser = await chromium.launch({ headless: true });
const base = "/__musea__/";
const server = await createServer({
  root: process.cwd(),
  configFile: false,
  plugins: [
    vize(),
    musea({
      include: ["src/**/*.vue"],
      inlineArt: true,
      previewSetup: "preview.setup.ts",
      toolbar,
    }),
  ],
  server: { host: "127.0.0.1", port: 0 },
});
let origin = "";
const foreign = createHttpServer((_request, response) => {
  response.setHeader("Content-Type", "text/html");
  response.end(`<script>parent.frames[0].postMessage({type:'musea:set-globals',
    publicProbe:'foreign',payload:${JSON.stringify(defaults)}},${JSON.stringify(origin)});</script>`);
});

try {
  await server.listen();
  const address = server.httpServer!.address();
  assert.ok(address && typeof address !== "string");
  origin = `http://127.0.0.1:${address.port}`;
  const context = await browser.newContext();
  const page = await context.newPage();
  page.on("pageerror", (error) => errors.push(String(error)));
  await page.goto(origin + base);
  await page.locator(".art-item").first().waitFor();
  const href = await page
    .locator(".art-item")
    .filter({ hasText: "Button" })
    .first()
    .getAttribute("href");
  assert.ok(href);
  const initial = new URL(href, page.url());
  initial.search = stringifyQuery({
    ...Object.fromEntries(initial.searchParams),
    museaGlobals: JSON.stringify(defaults),
  });
  initial.hash = "#variant-default";
  await page.goto(initial.href);
  await page.waitForURL((url) => url.href === initial.href);
  const before = await frames(page, defaults);
  await rememberDocuments(page);
  observations.push({ phase: "defaults", frames: before });
  for (const [control, value, action] of [
    [
      "brand",
      "ocean",
      () => page.getByRole("combobox", { name: "Brand", exact: true }).selectOption("1"),
    ],
    [
      "scheme",
      "dark",
      () =>
        page
          .getByRole("group", { name: "Component theme", exact: true })
          .getByRole("button", { name: "Dark", exact: true })
          .click(),
    ],
    [
      "locale",
      "ja",
      () => page.getByRole("combobox", { name: "Locale", exact: true }).selectOption("1"),
    ],
  ] as const) {
    const expected = new URL(page.url());
    const values = { ...JSON.parse(expected.searchParams.get("museaGlobals")!), [control]: value };
    expected.search = stringifyQuery({
      ...Object.fromEntries(expected.searchParams),
      museaGlobals: JSON.stringify(values),
    });
    await action();
    await page.waitForURL((url) => url.href === expected.href);
    assert.equal(new URL(page.url()).hash, "#variant-default");
    const current = await frames(page, values);
    const documents = await documentIdentity(page);
    assert.equal(documents.unchanged, true, `${control} must retain every actual preview Document`);
    assert.deepEqual(
      current.map((frame) => frame.documentToken),
      before.map((frame) => frame.documentToken),
    );
    observations.push({ phase: "toolbar-operation", control, value, documents, frames: current });
  }
  const after = await frames(page, changed);
  assert.deepEqual(
    after.map((frame) => frame.url),
    before.map((frame) => frame.url),
  );
  for (const frame of after) assert.deepEqual(frame.firstGlobals, defaults);
  observations.push({ phase: "globals-update-no-remount", frames: after, url: page.url() });
  const sharedUrl = page.url();
  await page.screenshot({
    path: path.join(path.dirname(output), "gallery-globals.png"),
    fullPage: true,
  });

  // Observe actual delivery before asserting rejection of sibling/foreign commands.
  await page.evaluate(() => {
    for (const frame of document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")) {
      frame.contentWindow!.addEventListener("message", (event) => {
        if (event.data?.publicProbe)
          Reflect.set(frame.contentWindow!, "__publicReceivedProbe", {
            id: event.data.publicProbe,
            origin: event.origin,
            parent: event.source === window,
          });
      });
    }
  });
  const first = await page.locator(".variant-card iframe").first().elementHandle();
  const preview = await first!.contentFrame();
  assert.ok(preview);
  await preview.evaluate((values) => {
    window.parent.document
      .querySelectorAll<HTMLIFrameElement>(".variant-card iframe")[1]
      .contentWindow!.postMessage(
        { type: "musea:set-globals", publicProbe: "sibling", payload: values },
        location.origin,
      );
  }, defaults);
  await page.waitForFunction(
    () =>
      Reflect.get(
        document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")[1].contentWindow!,
        "__publicReceivedProbe",
      )?.id === "sibling",
  );
  const sibling = await page.evaluate(() =>
    Reflect.get(
      document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")[1].contentWindow!,
      "__publicReceivedProbe",
    ),
  );
  assert.equal(sibling.parent, false);
  assert.equal(sibling.origin, origin);
  observations.push({
    phase: "sibling-refused",
    delivered: sibling,
    frames: await frames(page, changed),
  });
  await new Promise<void>((resolve) => foreign.listen(0, "127.0.0.1", resolve));
  const foreignAddress = foreign.address();
  assert.ok(foreignAddress && typeof foreignAddress !== "string");
  await page.evaluate((url) => {
    const iframe = document.createElement("iframe");
    iframe.src = url;
    document.body.append(iframe);
  }, `http://127.0.0.1:${foreignAddress.port}/`);
  await page.waitForFunction(
    () =>
      Reflect.get(
        document.querySelector<HTMLIFrameElement>(".variant-card iframe")!.contentWindow!,
        "__publicReceivedProbe",
      )?.id === "foreign",
  );
  const foreignEvent = await page.evaluate(() =>
    Reflect.get(
      document.querySelector<HTMLIFrameElement>(".variant-card iframe")!.contentWindow!,
      "__publicReceivedProbe",
    ),
  );
  assert.equal(foreignEvent.parent, false);
  assert.equal(foreignEvent.origin, `http://127.0.0.1:${foreignAddress.port}`);
  observations.push({
    phase: "foreign-refused",
    delivered: foreignEvent,
    frames: await frames(page, changed),
  });

  const opening = context.waitForEvent("page");
  await page
    .locator(".variant-card")
    .first()
    .getByTitle("Open in new tab", { exact: true })
    .click();
  const opened = await opening;
  opened.on("pageerror", (error) => errors.push(String(error)));
  await opened.waitForFunction((values) => {
    const root = document.documentElement;
    return (
      root.dataset.brand === values.brand &&
      root.dataset.scheme === values.scheme &&
      root.lang === values.locale
    );
  }, changed);
  await opened.locator("button.btn").first().waitFor();
  const newTab = await opened.evaluate(() => ({
    url: location.href,
    firstGlobals: Reflect.get(window, "__publicFirstGlobals"),
    calls: Reflect.get(window, "__publicSetupCalls"),
  }));
  assert.deepEqual(newTab.firstGlobals, changed);
  assert.equal(newTab.calls, 1);
  assert.deepEqual(JSON.parse(new URL(newTab.url).searchParams.get("museaGlobals")!), changed);
  observations.push({ phase: "new-tab-before-setup", ...newTab });
  await opened.close();
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.waitForFunction(() => document.querySelectorAll(".variant-card iframe").length === 0);
  assert.equal(new URL(page.url()).hash, "");
  await openArt(page, "Alert");
  assert.equal(new URL(page.url()).hash, "");
  observations.push({ phase: "another-art", frames: await frames(page, changed, "Alert") });
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await openArt(page, "Button");
  observations.push({ phase: "return-to-art", frames: await frames(page, changed) });
  const clean = await browser.newContext();
  const copied = await clean.newPage();
  copied.on("pageerror", (error) => errors.push(String(error)));
  await copied.goto(sharedUrl);
  const copiedFrames = await frames(copied, changed);
  for (const frame of copiedFrames) assert.deepEqual(frame.firstGlobals, changed);
  assert.equal(new URL(copied.url()).hash, "#variant-default");
  observations.push({ phase: "copied-url-clean-context", frames: copiedFrames });
  await rememberDocuments(copied);
  const reloadElement = await copied.locator(".variant-card iframe").first().elementHandle();
  const reloaded = await reloadElement!.contentFrame();
  assert.ok(reloaded);
  await reloaded.goto(reloaded.url());
  const reloadFrames = await frames(copied, changed);
  const refusal = await documentIdentity(copied);
  assert.equal(refusal.unchanged, false, "same-URL reload must fail the Document identity oracle");
  assert.equal(reloadFrames[0].url, copiedFrames[0].url);
  assert.notEqual(reloadFrames[0].documentToken, copiedFrames[0].documentToken);
  observations.push({ phase: "same-url-reload-oracle-control", refusal, frames: reloadFrames });
  await clean.close();
  await context.close();
  observations.push(await checkLegacy(browser, errors));
  const { packages, loaded } = publicNative(version);
  assert.deepEqual(errors, []);
  mkdirSync(path.dirname(output), { recursive: true });
  writeFileSync(
    output,
    JSON.stringify(
      { issue: 8329, version, packages, loaded, observations, errors, success: true },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
} finally {
  await browser.close();
  await server.close();
  if (foreign.listening)
    await new Promise<void>((resolve, reject) =>
      foreign.close((error) => (error ? reject(error) : resolve())),
    );
}
