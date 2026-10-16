import assert from "node:assert/strict";
import type { Page, Request } from "playwright";

/** Observe before navigation so failed resources cannot disappear before readiness. */
export function createPreviewReadinessWaiter(page: Page): () => Promise<void> {
  const pending = new Map<Request, { settled: Promise<void>; finish: () => void }>();
  const failures: string[] = [];
  page.on("request", (request) => {
    const { promise, resolve } = Promise.withResolvers<void>();
    pending.set(request, { settled: promise, finish: resolve });
  });
  page.on("response", (response) => {
    if (response.status() >= 400) failures.push(`${response.status()} ${response.url()}`);
  });
  function finish(request: Request): void {
    pending.get(request)?.finish();
    pending.delete(request);
  }
  page.on("requestfinished", finish);
  page.on("requestfailed", (request) => {
    failures.push(`${request.failure()?.errorText} ${request.url()}`);
    finish(request);
  });
  async function drain(): Promise<void> {
    let timeout: ReturnType<typeof setTimeout> | undefined;
    try {
      await Promise.race([
        (async () => {
          while (pending.size > 0)
            await Promise.all([...pending.values()].map(({ settled }) => settled));
          assert.deepEqual(failures, [], "preview resource failures");
        })(),
        new Promise<never>((_, reject) => {
          timeout = setTimeout(
            () =>
              reject(
                new Error(
                  `Preview resources did not finish: ${[...pending.keys()].map((request) => request.url()).join(", ")}`,
                ),
              ),
            30_000,
          );
        }),
      ]);
    } finally {
      clearTimeout(timeout);
    }
  }
  return async () => {
    await page.waitForLoadState("load");
    await page.evaluate(async () => {
      await document.fonts.ready;
      for (const font of document.fonts)
        if (font.status === "error") throw new Error(`Failed preview font: ${font.family}`);
      await Promise.all([...document.images].map((image) => image.decode()));
      for (const link of document.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]'))
        if (!link.disabled && !link.sheet)
          throw new Error(`Failed preview stylesheet: ${link.href}`);
    });
    await drain();
    const elements = await page.locator(".preview-shell, .preview-shell *").elementHandles();
    assert.ok(elements.length > 0, "preview shell exists before capture");
    try {
      // Native Playwright stability samples two RAF geometries even under page.clock.
      await Promise.all(
        elements.map(async (element) => {
          if (await element.isVisible()) await element.waitForElementState("stable");
        }),
      );
    } finally {
      await Promise.all(elements.map((element) => element.dispose()));
    }
    await drain();
  };
}
