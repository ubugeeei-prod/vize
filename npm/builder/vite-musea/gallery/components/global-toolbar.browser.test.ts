import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { createServer as createHttpServer } from "node:http";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright";
import {
  basePath,
  defaults,
  changed,
  prepareContext,
  createFixtureServer,
  checkOpenedPreview,
  checkMobileToolbar,
  openHashComponent,
  checkHashToolbarChanges,
} from "./global-toolbar.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output =
  process.env.MUSEA_GLOBALS_PROOF_DIR || path.join(repository, "artifacts/musea-globals");
async function listenerSnapshot(page: Page) {
  return page.evaluate(() => ({
    count: Reflect.get(window, "__toolbarMessageListenerCount") as number,
    names: Reflect.get(window, "__toolbarMessageListenerNames") as string[],
  }));
}

async function checkGlobals(page: Page, expected: typeof defaults) {
  await page.waitForFunction((values) => {
    const frames = [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")];
    return (
      frames.length === 2 &&
      frames.every((frame) => {
        const element = frame.contentDocument?.querySelector("output");
        return (
          element?.getAttribute("data-is-ref") === "true" &&
          element.getAttribute("data-theme") === values.theme &&
          element.getAttribute("data-locale") === values.locale &&
          element.getAttribute("data-rtl") === String(values.rtl)
        );
      })
    );
  }, expected);
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map((frame) => ({
      url: frame.src,
      globals: JSON.parse(frame.contentDocument!.querySelector("output")!.textContent!),
      setupCalls: Reflect.get(frame.contentWindow!, "__toolbarSetupCalls"),
      receivedProbe: Reflect.get(frame.contentWindow!, "__toolbarReceivedProbe"),
    })),
  );
}

async function sendGlobals(page: Page, payload: Record<string, unknown>) {
  await page.evaluate((values) => {
    for (const frame of document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")) {
      frame.contentWindow!.postMessage(
        { type: "musea:set-globals", payload: values },
        location.origin,
      );
    }
  }, payload);
}

async function settleMessages(page: Page) {
  await page.evaluate(
    () =>
      new Promise<void>((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      }),
  );
}

await test(
  "global toolbar updates real Vue previews, survives navigation and restores shared URLs",
  {
    skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
  },
  async () => {
    const observations: unknown[] = [];
    const errors: string[] = [];
    const server = await createFixtureServer(output);
    const browser = await chromium.launch();
    let galleryOrigin = "";
    const foreign = createHttpServer((_request, response) => {
      response.setHeader("Content-Type", "text/html");
      response.end(
        `<script>parent.frames[0].postMessage({type:'musea:set-globals',browserProbe:'foreign',payload:{theme:'light',locale:'en',rtl:false}},${JSON.stringify(galleryOrigin)});</script><output>Command sent</output>`,
      );
    });
    try {
      await mkdir(output, { recursive: true });
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const origin = `http://127.0.0.1:${address.port}`;
      galleryOrigin = origin;
      await new Promise<void>((resolve) => foreign.listen(0, "127.0.0.1", resolve));
      const foreignAddress = foreign.address();
      assert.ok(foreignAddress && typeof foreignAddress !== "string");
      const context = await browser.newContext();
      await prepareContext(context);
      const page = await context.newPage();
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`${origin}${basePath}/`);
      await page.locator(".art-item").first().waitFor();
      const initialListeners = await listenerSnapshot(page);
      observations.push({ phase: "initial-message-listeners", listeners: initialListeners });
      assert.deepEqual(initialListeners, { count: 0, names: [] });
      // useA11y registers this verified app-lifetime singleton on the first A11yBadge.
      const homeListeners = { count: 1, names: ["handleA11yMessage"] };
      await openHashComponent(page);
      observations.push({ phase: "defaults", frames: await checkGlobals(page, defaults) });
      const componentListeners = await listenerSnapshot(page);
      observations.push({ phase: "component-message-listeners", listeners: componentListeners });
      observations.push({
        phase: "toolbar-hash-retention",
        controls: await checkHashToolbarChanges(page),
      });
      const changedFrames = await checkGlobals(page, changed);
      assert.ok(
        changedFrames.every((frame) => frame.setupCalls === 1),
        "Globals update without remounting previews",
      );
      observations.push({ phase: "changed", frames: changedFrames });
      assert.equal(
        await page.getByRole("button", { name: "RTL", exact: true }).getAttribute("aria-pressed"),
        "true",
      );
      observations.push({
        phase: "open-new-tab-current-globals",
        preview: await checkOpenedPreview(context, page, changed),
      });
      const sharedUrl = page.url();
      assert.deepEqual(JSON.parse(new URL(sharedUrl).searchParams.get("museaGlobals")!), changed);

      // Invalid options reset to declared defaults; unknown keys never reach setup.
      await sendGlobals(page, { theme: "sunset", locale: "unknown", rtl: "true", extra: "no" });
      const sanitized = await checkGlobals(page, defaults);
      for (const frame of sanitized) assert.deepEqual(frame.globals, defaults);
      observations.push({ phase: "invalid-options-sanitized", frames: sanitized });
      await sendGlobals(page, changed);
      await checkGlobals(page, changed);

      // Observe actual delivery before asserting rejection, without handling commands.
      await page.evaluate(() => {
        for (const frame of document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")) {
          frame.contentWindow!.addEventListener("message", (event) => {
            if (event.data?.browserProbe) {
              Reflect.set(frame.contentWindow!, "__toolbarReceivedProbe", {
                id: event.data.browserProbe,
                origin: event.origin,
                sourceIsParent: event.source === window,
              });
            }
          });
        }
      });

      // An actual sibling WindowProxy must not issue privileged parent commands.
      const firstFrame = await page.locator(".variant-card iframe").first().elementHandle();
      const preview = await firstFrame!.contentFrame();
      assert.ok(preview);
      await preview.evaluate(() => {
        const sibling =
          window.parent.document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")[1];
        sibling.contentWindow!.postMessage(
          {
            type: "musea:set-globals",
            browserProbe: "sibling",
            payload: { theme: "light", locale: "en", rtl: false },
          },
          location.origin,
        );
      });
      await page.waitForFunction(
        () =>
          Reflect.get(
            document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")[1].contentWindow!,
            "__toolbarReceivedProbe",
          )?.id === "sibling",
      );
      await settleMessages(page);
      observations.push({
        phase: "sibling-command-rejected",
        frames: await checkGlobals(page, changed),
      });

      // Different HTTP origins exercise Chromium's actual origin assignment.
      await page.evaluate((url) => {
        const iframe = document.createElement("iframe");
        iframe.id = "toolbar-foreign-command";
        iframe.src = url;
        document.body.append(iframe);
      }, `http://127.0.0.1:${foreignAddress.port}/`);
      await page.frameLocator("#toolbar-foreign-command").locator("output").waitFor();
      await page.waitForFunction(
        () =>
          Reflect.get(
            document.querySelector<HTMLIFrameElement>(".variant-card iframe")!.contentWindow!,
            "__toolbarReceivedProbe",
          )?.id === "foreign",
      );
      await settleMessages(page);
      observations.push({
        phase: "foreign-command-rejected",
        frames: await checkGlobals(page, changed),
      });
      await page.locator("#toolbar-foreign-command").evaluate((element) => element.remove());

      // The existing prop-remount path must pass the same reactive Ref to setup.
      await page.evaluate(() => {
        for (const frame of document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")) {
          frame.contentWindow!.postMessage(
            { type: "musea:set-props", payload: { props: { test: "remounted" } } },
            location.origin,
          );
        }
      });
      await page.waitForFunction(() =>
        [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].every(
          (frame) => Reflect.get(frame.contentWindow!, "__toolbarSetupCalls") === 2,
        ),
      );
      observations.push({
        phase: "prop-remount-same-ref",
        frames: await checkGlobals(page, changed),
      });

      await page.getByRole("link", { name: "Home", exact: true }).click();
      await page.waitForFunction(
        () => document.querySelectorAll(".variant-card iframe").length === 0,
      );
      assert.deepEqual(
        await listenerSnapshot(page),
        homeListeners,
        "Unmounted previews must release message listeners",
      );
      observations.push({
        phase: "listener-cleanup",
        initialListeners,
        homeListeners,
        componentListeners,
        unmountedListeners: await listenerSnapshot(page),
      });
      const homeUrl = page.url();
      assert.equal(
        new URL(homeUrl).hash,
        "",
        "Home navigation does not inherit the component anchor",
      );
      await page.locator(".art-item").last().click();
      observations.push({ phase: "second-art-remount", frames: await checkGlobals(page, changed) });
      assert.deepEqual(
        await listenerSnapshot(page),
        componentListeners,
        "Navigation must not accumulate listeners",
      );

      const secondUrl = page.url();
      assert.equal(
        new URL(secondUrl).hash,
        "",
        "Another art does not inherit the component anchor",
      );
      // The globals guard must preserve push navigation and the real browser history.
      for (const step of [
        { direction: "back", url: homeUrl, title: null },
        { direction: "back", url: sharedUrl, title: "First" },
        { direction: "forward", url: homeUrl, title: null },
        { direction: "forward", url: secondUrl, title: "Second" },
      ] as const) {
        if (step.direction === "back") await page.goBack();
        else await page.goForward();
        await page.waitForURL((url) => url.href === step.url);
        if (step.title)
          await page.getByRole("heading", { name: step.title, exact: true }).waitFor();
        else await page.locator(".component-header").waitFor({ state: "detached" });
        const frames = step.title ? await checkGlobals(page, changed) : [];
        if (!step.title) assert.equal(await page.locator(".variant-card iframe").count(), 0);
        assert.deepEqual(
          JSON.parse(new URL(page.url()).searchParams.get("museaGlobals")!),
          changed,
        );
        const listeners = await listenerSnapshot(page);
        assert.deepEqual(listeners, step.title ? componentListeners : homeListeners);
        observations.push({
          phase: `history-${step.direction}-${step.title || "home"}`,
          url: page.url(),
          listeners,
          frames,
        });
      }

      // A clean context has no prior localStorage; the URL alone restores values.
      const cleanContext = await browser.newContext();
      await prepareContext(cleanContext);
      const cleanPage = await cleanContext.newPage();
      cleanPage.on("pageerror", (error) => errors.push(String(error)));
      await cleanPage.goto(sharedUrl);
      observations.push({
        phase: "clean-context-url-replay",
        frames: await checkGlobals(cleanPage, changed),
      });
      await cleanContext.close();
      // Removing the query in the original context exercises persisted defaults.
      await page.goto(`${origin}${basePath}/component/First.art.vue`);
      observations.push({
        phase: "local-storage-replay",
        frames: await checkGlobals(page, changed),
      });
      await page.setViewportSize({ width: 390, height: 900 });
      await page.getByLabel("Theme", { exact: true }).scrollIntoViewIfNeeded();
      observations.push({
        phase: "mobile-toolbar-geometry",
        measured: await checkMobileToolbar(page),
      });
      await page
        .locator(".addon-toolbar")
        .screenshot({ path: path.join(output, "toolbar-mobile.png") });
      assert.deepEqual(errors, []);
    } finally {
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(
          {
            qualification:
              "Real gallery, Vue apps, generated preview modules and setup hook; fixture art API/modules; no native compilation claim",
            observations,
            errors,
          },
          null,
          2,
        ),
      );
      await browser.close();
      await server.close();
      await new Promise<void>((resolve) => foreign.close(() => resolve()));
    }
  },
);
