import type { Page } from "@playwright/test";

export async function setupMisskeyMocks(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const _origFetch = window.fetch;
    window.fetch = function (input, init) {
      const url =
        typeof input === "string" ? input : input instanceof URL ? input.toString() : input.url;
      if (url.includes("/api/")) {
        let body = "{}";
        if (url.includes("/api/meta")) {
          body = JSON.stringify({
            name: "Misskey",
            uri: "http://localhost:3000",
            version: "2024.11.0",
            description: "A Misskey instance",
            disableRegistration: false,
            federation: "all",
            iconUrl: null,
            backgroundImageUrl: null,
            defaultDarkTheme: null,
            defaultLightTheme: null,
            clientOptions: {},
            policies: { ltlAvailable: true, gtlAvailable: true },
            maxNoteTextLength: 3000,
            features: {
              registration: true,
              localTimeline: true,
              globalTimeline: true,
              miauth: true,
            },
          });
        } else if (url.includes("/api/emojis")) {
          body = JSON.stringify({ emojis: [] });
        }
        return Promise.resolve(
          new Response(body, {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }
      if (url.includes("/assets/locales/")) {
        return Promise.resolve(
          new Response(JSON.stringify({}), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }
      return _origFetch.call(window, input, init);
    } as typeof window.fetch;
  });
}

export async function mockRoute(
  page: Page,
  pattern: string | RegExp,
  response: { status?: number; body?: string; contentType?: string },
): Promise<void> {
  await page.route(pattern, (route) => {
    return route.fulfill({
      status: response.status ?? 200,
      contentType: response.contentType ?? "application/json",
      body: response.body ?? "{}",
    });
  });
}
