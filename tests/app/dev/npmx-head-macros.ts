import { expect, test, type Page } from "@playwright/test";
import { readNpmxHeadFixtureContent, readNpmxHeadSourceEvidence } from "./npmx-head-contract.ts";

type HeadState = {
  canonical: string[];
  colorScheme: string[];
  description: string[];
  htmlDir: string | null;
  htmlKeyboardHints: string | null;
  htmlLang: string | null;
  ogDescription: string[];
  ogTitle: string[];
  title: string[];
  twitterDescription: string[];
  twitterTitle: string[];
};

type RouterRouteRecord = {
  aliasOf?: { path: string };
  meta?: Record<string, unknown>;
  name?: string | symbol | null;
  path: string;
};

type RouteRecordSnapshot = {
  aliasOf: string | null;
  meta: Record<string, unknown>;
  name: string | null;
  path: string;
};

export type RouteSnapshot = {
  fullPath: string;
  matched: RouteRecordSnapshot[];
  meta: Record<string, unknown>;
  name: string | null;
  params: Record<string, unknown>;
  path: string;
  records: RouteRecordSnapshot[];
};

function expectedHead(
  title: string,
  description: string,
  options: {
    canonical?: string;
    includeSocial?: boolean;
    ogTitle?: string;
    twitterTitle?: string;
  } = {},
): HeadState {
  const includeSocial = options.includeSocial ?? true;
  return {
    canonical: options.canonical === undefined ? [] : [options.canonical],
    colorScheme: ["dark light"],
    description: [description],
    htmlDir: "ltr",
    htmlKeyboardHints: "false",
    htmlLang: "en-US",
    ogDescription: includeSocial ? [description] : [],
    ogTitle: includeSocial ? [options.ogTitle ?? title] : [],
    title: [title],
    twitterDescription: includeSocial ? [description] : [],
    twitterTitle: includeSocial ? [options.twitterTitle ?? title] : [],
  };
}

export async function captureHead(page: Page, html: string | null = null): Promise<HeadState> {
  return page.evaluate((markup) => {
    const root = markup === null ? document : new DOMParser().parseFromString(markup, "text/html");
    const attributes = (selector: string, attribute: string): string[] =>
      [...root.querySelectorAll(selector)].map((element) => element.getAttribute(attribute) ?? "");

    return {
      canonical: attributes('link[rel="canonical"]', "href"),
      colorScheme: attributes('meta[name="color-scheme"]', "content"),
      description: attributes('meta[name="description"]', "content"),
      htmlDir: root.documentElement.getAttribute("dir"),
      htmlKeyboardHints: root.documentElement.getAttribute("data-kbd-hints"),
      htmlLang: root.documentElement.getAttribute("lang"),
      ogDescription: attributes('meta[property="og:description"]', "content"),
      ogTitle: attributes('meta[property="og:title"]', "content"),
      title: [...root.head.querySelectorAll("title")].map((element) => element.textContent ?? ""),
      twitterDescription: attributes('meta[name="twitter:description"]', "content"),
      twitterTitle: attributes('meta[name="twitter:title"]', "content"),
    };
  }, html);
}

async function fetchHead(page: Page, url: string): Promise<HeadState> {
  const response = await page.request.get(url);
  expect(response.ok(), `SSR request failed: ${url}`).toBe(true);
  return captureHead(page, await response.text());
}

async function expectDocumentHead(page: Page, expected: HeadState): Promise<void> {
  await expect.poll(() => captureHead(page), { timeout: 20_000 }).toEqual(expected);
}

export async function readCurrentRoute(page: Page): Promise<RouteSnapshot> {
  await page.waitForFunction(() => {
    const root = document.querySelector("#__nuxt") as {
      __vue_app__?: { config?: { globalProperties?: { $router?: { currentRoute?: unknown } } } };
    } | null;
    return root?.__vue_app__?.config?.globalProperties?.$router !== undefined;
  });

  return page.evaluate(() => {
    const root = document.querySelector("#__nuxt") as {
      __vue_app__?: {
        config?: {
          globalProperties?: {
            $router?: {
              getRoutes?: () => RouterRouteRecord[];
              currentRoute?: {
                value?: {
                  fullPath?: string;
                  matched?: RouterRouteRecord[];
                  meta?: Record<string, unknown>;
                  name?: string | symbol | null;
                  params?: Record<string, unknown>;
                  path?: string;
                };
              };
            };
          };
        };
      };
    } | null;
    const router = root?.__vue_app__?.config?.globalProperties?.$router;
    const route = router?.currentRoute?.value;
    if (!route?.path || !route.fullPath || !route.matched || !router?.getRoutes) {
      throw new Error("Nuxt router currentRoute or route records are unavailable");
    }
    const captureRecord = (record: RouterRouteRecord): RouteRecordSnapshot => ({
      aliasOf: record.aliasOf?.path ?? null,
      meta: JSON.parse(JSON.stringify(record.meta ?? {})) as Record<string, unknown>,
      name: record.name == null ? null : String(record.name),
      path: record.path,
    });
    return {
      fullPath: route.fullPath,
      matched: route.matched.map(captureRecord),
      meta: JSON.parse(JSON.stringify(route.meta ?? {})) as Record<string, unknown>,
      name: route.name == null ? null : String(route.name),
      params: JSON.parse(JSON.stringify(route.params ?? {})) as Record<string, unknown>,
      path: route.path,
      records: router
        .getRoutes()
        .filter((record) => record.name === route.name)
        .map(captureRecord)
        .sort((left, right) => (left.path < right.path ? -1 : left.path > right.path ? 1 : 0)),
    };
  });
}

export async function navigateWithNuxtRouter(page: Page, targetPath: string): Promise<void> {
  await page.evaluate(async (path) => {
    const root = document.querySelector("#__nuxt") as {
      __vue_app__?: {
        config?: {
          globalProperties?: { $router?: { push?: (target: string) => Promise<unknown> } };
        };
      };
    } | null;
    const push = root?.__vue_app__?.config?.globalProperties?.$router?.push;
    if (push === undefined) throw new Error("Nuxt router is unavailable");
    await push(path);
  }, targetPath);
  await expect.poll(() => new URL(page.url()).pathname, { timeout: 20_000 }).toBe(targetPath);
}

async function expectDocsRoute(
  page: Page,
  requestedPath: string,
  expectedDocsHead: HeadState,
): Promise<void> {
  await navigateWithNuxtRouter(page, requestedPath);
  const route = await readCurrentRoute(page);
  await test.info().attach(`npmx-route-${requestedPath}`, {
    body: JSON.stringify(route, null, 2),
    contentType: "application/json",
  });
  const recordPaths: Record<string, string> = {
    "/package-docs/nuxt/v/4.0.0": "/package-docs/:path+",
    "/package/docs/nuxt/v/4.0.0": "/package/docs/:path+",
    "/docs/nuxt/v/4.0.0": "/docs/:path+",
  };
  const recordPath = recordPaths[requestedPath];
  if (recordPath === undefined) throw new Error(`unknown authored docs route: ${requestedPath}`);
  const canonicalRecordPath = "/package-docs/:path+";
  const expectedRecord = (path: string): RouteRecordSnapshot => ({
    aliasOf: path === canonicalRecordPath ? null : canonicalRecordPath,
    meta: { scrollMargin: 180 },
    name: "docs",
    path,
  });
  expect(route).toEqual({
    fullPath: requestedPath,
    matched: [expectedRecord(recordPath)],
    meta: { scrollMargin: 180 },
    name: "docs",
    params: { path: ["nuxt", "v", "4.0.0"] },
    path: requestedPath,
    records: ["/docs/:path+", canonicalRecordPath, "/package/docs/:path+"].map(expectedRecord),
  });
  await expectDocumentHead(page, expectedDocsHead);
}

export async function verifyNpmxHeadMacros(
  page: Page,
  baseUrl: string,
  fixtureRoot: string,
): Promise<void> {
  const sourceEvidence = readNpmxHeadSourceEvidence(fixtureRoot);
  const fixtureContent = readNpmxHeadFixtureContent(fixtureRoot);
  const aboutHead = expectedHead(
    `${fixtureContent.aboutTitle} - npmx`,
    fixtureContent.aboutDescription,
  );
  const accessibilityHead = expectedHead(
    `${fixtureContent.accessibilityTitle} - npmx`,
    fixtureContent.accessibilityDescription,
    {
      includeSocial: false,
    },
  );
  const docsHead = expectedHead(fixtureContent.docsTitle, "MIT", {
    ogTitle: fixtureContent.docsOgTitle,
  });

  try {
    expect(await fetchHead(page, `${baseUrl}/about`)).toEqual(aboutHead);
    expect(await fetchHead(page, `${baseUrl}/accessibility`)).toEqual(accessibilityHead);
    expect(await fetchHead(page, `${baseUrl}/docs/nuxt/v/4.0.0`)).toEqual(docsHead);

    await page.goto(`${baseUrl}/about`, { waitUntil: "load", timeout: 30_000 });
    await readCurrentRoute(page);
    await expectDocumentHead(page, aboutHead);

    await navigateWithNuxtRouter(page, "/package/vue");
    await expect
      .poll(() => captureHead(page), { timeout: 20_000 })
      .toMatchObject({
        canonical: ["https://npmx.dev/package/vue"],
        ogTitle: ["vue - npmx"],
        title: ["vue - npmx"],
        twitterTitle: ["vue - npmx"],
      });
    const packageHead = await captureHead(page);
    expect(packageHead.description).toHaveLength(1);
    expect(packageHead.description[0]?.length).toBeGreaterThan(0);
    expect(packageHead.ogDescription).toEqual(packageHead.description);
    expect(packageHead.twitterDescription).toEqual(packageHead.description);

    await navigateWithNuxtRouter(page, "/about");
    await expectDocumentHead(page, aboutHead);

    for (const docsPath of [
      "/package-docs/nuxt/v/4.0.0",
      "/package/docs/nuxt/v/4.0.0",
      "/docs/nuxt/v/4.0.0",
    ]) {
      await navigateWithNuxtRouter(page, "/about");
      await expectDocumentHead(page, aboutHead);
      await expectDocsRoute(page, docsPath, docsHead);
    }

    await navigateWithNuxtRouter(page, "/accessibility");
    const accessibilityRoute = await readCurrentRoute(page);
    await test.info().attach("npmx-route-accessibility", {
      body: JSON.stringify(accessibilityRoute, null, 2),
      contentType: "application/json",
    });
    const accessibilityRecord = {
      aliasOf: null,
      meta: {},
      name: "accessibility",
      path: "/accessibility",
    };
    expect(accessibilityRoute).toEqual({
      fullPath: "/accessibility",
      matched: [accessibilityRecord],
      meta: {},
      name: "accessibility",
      params: {},
      path: "/accessibility",
      records: [accessibilityRecord],
    });
    await expectDocumentHead(page, accessibilityHead);
  } finally {
    expect(readNpmxHeadSourceEvidence(fixtureRoot)).toEqual(sourceEvidence);
  }
}
