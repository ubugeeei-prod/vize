import { expect, type Page } from "@playwright/test";
import { readNpmxHeadSourceEvidence } from "./npmx-head-contract.ts";

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

export type RouteSnapshot = {
  fullPath: string;
  meta: Record<string, unknown>;
  name: string | null;
  params: Record<string, unknown>;
  path: string;
};

const ABOUT_DESCRIPTION =
  "npmx is a fast, modern browser for the npm registry. A better UX/DX for exploring npm packages.";
const ACCESSIBILITY_DESCRIPTION = "We want npmx to be usable by as many people as possible.";

function expectedHead(
  title: string,
  description: string,
  options: { canonical?: string; includeSocial?: boolean } = {},
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
    ogTitle: includeSocial ? [title] : [],
    title: [title],
    twitterDescription: includeSocial ? [description] : [],
    twitterTitle: includeSocial ? [title] : [],
  };
}

async function captureHead(page: Page, html: string | null = null): Promise<HeadState> {
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
              currentRoute?: {
                value?: {
                  fullPath?: string;
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
    const route = root?.__vue_app__?.config?.globalProperties?.$router?.currentRoute?.value;
    if (!route?.path || !route.fullPath) throw new Error("Nuxt router currentRoute is unavailable");
    return {
      fullPath: route.fullPath,
      meta: JSON.parse(JSON.stringify(route.meta ?? {})) as Record<string, unknown>,
      name: route.name == null ? null : String(route.name),
      params: JSON.parse(JSON.stringify(route.params ?? {})) as Record<string, unknown>,
      path: route.path,
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

async function expectDocsRoute(page: Page, requestedPath: string): Promise<void> {
  await navigateWithNuxtRouter(page, requestedPath);
  const route = await readCurrentRoute(page);
  expect(route).toEqual({
    fullPath: requestedPath,
    meta: {
      alias: ["/package/docs/:path+", "/docs/:path+"],
      name: "docs",
      path: "/package-docs/:path+",
      scrollMargin: 180,
    },
    name: "docs",
    params: { path: ["nuxt", "v", "4.0.0"] },
    path: requestedPath,
  });
  await expectDocumentHead(page, expectedHead("nuxt@4.0.0 docs - npmx", "MIT"));
}

export async function verifyNpmxHeadMacros(
  page: Page,
  baseUrl: string,
  fixtureRoot: string,
): Promise<void> {
  const sourceEvidence = readNpmxHeadSourceEvidence(fixtureRoot);
  const aboutHead = expectedHead("About - npmx", ABOUT_DESCRIPTION);
  const accessibilityHead = expectedHead("accessibility - npmx", ACCESSIBILITY_DESCRIPTION, {
    includeSocial: false,
  });

  try {
    expect(await fetchHead(page, `${baseUrl}/about`)).toEqual(aboutHead);
    expect(await fetchHead(page, `${baseUrl}/accessibility`)).toEqual(accessibilityHead);
    expect(await fetchHead(page, `${baseUrl}/docs/nuxt/v/4.0.0`)).toEqual(
      expectedHead("nuxt@4.0.0 docs - npmx", "MIT"),
    );

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
      await expectDocsRoute(page, docsPath);
    }

    await navigateWithNuxtRouter(page, "/accessibility");
    expect(await readCurrentRoute(page)).toEqual({
      fullPath: "/accessibility",
      meta: {},
      name: "accessibility",
      params: {},
      path: "/accessibility",
    });
    await expectDocumentHead(page, accessibilityHead);
  } finally {
    expect(readNpmxHeadSourceEvidence(fixtureRoot)).toEqual(sourceEvidence);
  }
}
