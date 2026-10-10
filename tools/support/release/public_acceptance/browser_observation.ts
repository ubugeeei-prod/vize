import { createHash } from "node:crypto";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Browser, BrowserContext, Page } from "playwright";

/** Failure artifacts are observations; they never create an acceptance receipt. */
export function observeBrowser(output: string) {
  const directory = path.join(path.dirname(output), "musea-browser-failure");
  const events: Record<string, unknown>[] = [];
  const pending: Promise<void>[] = [];
  const seen = new WeakSet<Page>();
  const digest = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
  const save = async (name: string, bytes: string | Buffer) => {
    await mkdir(directory, { recursive: true });
    await writeFile(path.join(directory, name), bytes);
  };

  function page(page: Page) {
    if (seen.has(page)) return;
    seen.add(page);
    page.on("pageerror", (error) =>
      events.push({
        type: "pageerror",
        url: page.url(),
        message: String(error),
        stack: error.stack,
      }),
    );
    page.on("console", (message) =>
      events.push({
        type: "console",
        kind: message.type(),
        text: message.text(),
        location: message.location(),
      }),
    );
    page.on("requestfailed", (request) =>
      events.push({ type: "requestfailed", url: request.url(), failure: request.failure() }),
    );
    page.on("response", (response) => {
      const contentType = response.headers()["content-type"] ?? "";
      const event: Record<string, unknown> = {
        type: "response",
        url: response.url(),
        status: response.status(),
        contentType,
        resourceType: response.request().resourceType(),
      };
      events.push(event);
      pending.push(
        (async () => {
          try {
            if (!/javascript|text\/|json/u.test(contentType)) return;
            const bytes = await response.body();
            const filename = `${digest(response.url())}-${digest(bytes)}.body`;
            await save(filename, bytes);
            event.bodyFile = filename;
            event.bodySha256 = digest(bytes);
            event.bodyBytes = bytes.length;
          } catch (error) {
            event.bodyError = String(error);
          }
        })(),
      );
    });
  }

  function context(context: BrowserContext) {
    context.on("page", page);
    for (const existing of context.pages()) page(existing);
  }

  async function failure(browser: Browser, error: unknown, scope: unknown) {
    const pages: Record<string, unknown>[] = [];
    for (const [contextIndex, context] of browser.contexts().entries()) {
      for (const [pageIndex, page] of context.pages().entries()) {
        const name = `page-${contextIndex}-${pageIndex}`;
        const frames: Record<string, unknown>[] = [];
        for (const [frameIndex, frame] of page.frames().entries()) {
          const observation: Record<string, unknown> = { url: frame.url() };
          try {
            const html = await frame.content();
            const filename = `${name}-frame-${frameIndex}.html`;
            await save(filename, html);
            observation.htmlFile = filename;
            observation.htmlSha256 = digest(html);
            observation.state = await frame.evaluate(() => ({
              ready: document.readyState,
              title: document.title,
              root: {
                ...Object.fromEntries(Object.entries(document.documentElement.dataset)),
                lang: document.documentElement.lang,
              },
              setupCalls: Reflect.get(window, "__publicSetupCalls"),
              firstGlobals: Reflect.get(window, "__publicFirstGlobals"),
              documentToken: Reflect.get(window, "__publicDocumentToken"),
              buttons: document.querySelectorAll("button.btn").length,
              unresolvedComponents: document.querySelectorAll("museacomponent").length,
            }));
          } catch (captureError) {
            observation.captureError = String(captureError);
          }
          frames.push(observation);
        }
        const observation: Record<string, unknown> = { url: page.url(), frames };
        try {
          await mkdir(directory, { recursive: true });
          const filename = `${name}.png`;
          await page.screenshot({
            path: path.join(directory, filename),
            fullPage: true,
            timeout: 5000,
          });
          observation.screenshotFile = filename;
        } catch (captureError) {
          observation.screenshotError = String(captureError);
        }
        pages.push(observation);
      }
    }
    await Promise.allSettled(pending);
    await save(
      "failure.json",
      JSON.stringify(
        {
          schema: "vize-public-browser-failure-observation-v1",
          checkedAt: new Date().toISOString(),
          error: String(error),
          stack: error instanceof Error ? error.stack : undefined,
          scope,
          pages,
          events,
          success: false,
        },
        null,
        2,
      ) + "\n",
    );
  }

  return { context, page, failure };
}
