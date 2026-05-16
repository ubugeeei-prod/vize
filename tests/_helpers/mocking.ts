import type { Page } from "@playwright/test";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { MISSKEY_WORK_DIR } from "./apps.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const MISSKEY_LOCALES_DIR = path.resolve(MISSKEY_WORK_DIR, "built/_frontend_dist_/locales");
const MISSKEY_SW_STUB = `
self.addEventListener("install", (event) => {
  event.waitUntil(self.skipWaiting());
});
self.addEventListener("activate", (event) => {
  event.waitUntil(self.clients.claim());
});
self.addEventListener("message", () => {});
`;
const EMPTY_SVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"></svg>`;
const TRANSPARENT_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2P8z/C/HwAFgwJ/lE6nWQAAAABJRU5ErkJggg==",
  "base64",
);
const FALLBACK_LOCALE = JSON.stringify({
  _lang_: "English",
  headlineMisskey: "A network connected by notes",
  login: "Sign In",
  signup: "Sign Up",
  joinThisServer: "Sign up at this instance",
  exploreOtherServers: "Look for another instance",
  letsLookAtTimeline: "Have a look at the timeline",
  notifications: "Notifications",
  drive: "Drive",
  followRequests: "Follow requests",
  explore: "Explore",
  announcements: "Announcements",
  search: "Search",
  lookup: "Lookup",
  qr: "QR",
  lists: "Lists",
  antennas: "Antennas",
  favorites: "Favorites",
  pages: "Pages",
  gallery: "Gallery",
  clip: "Clip",
  channel: "Channel",
  directMessage_short: "DM",
  achievements: "Achievements",
  switchUi: "Switch UI",
  about: "About",
  tools: "Tools",
  reload: "Reload",
  profile: "Profile",
  clearCache: "Clear cache",
  _bootErrors: {
    title: "Boot error",
  },
  users: "Users",
  notes: "Notes",
  noNotes: "No notes",
  notFound: "Not found",
  notFoundDescription: "The page could not be found.",
  recentUsed: "Recently used",
  customEmojis: "Custom emojis",
  emoji: "Emoji",
});
const MISSKEY_API_FIXTURES = {
  meta: {
    name: "Misskey",
    uri: "http://localhost:3000",
    version: "2026.2.0-beta.0",
    description: "A Misskey instance",
    disableRegistration: false,
    federation: "all",
    iconUrl: null,
    backgroundImageUrl: null,
    defaultDarkTheme: null,
    defaultLightTheme: null,
    clientOptions: {
      showActivitiesForVisitor: true,
      showTimelineForVisitor: true,
    },
    policies: { ltlAvailable: true, gtlAvailable: true },
    maxNoteTextLength: 3000,
    features: {
      registration: true,
      localTimeline: true,
      globalTimeline: true,
      miauth: true,
    },
  },
  emojis: { emojis: [] },
  stats: {
    originalUsersCount: 1234,
    originalNotesCount: 56789,
  },
  "charts/active-users": {
    read: Array.from({ length: 30 }, (_, index) => 240 - index * 5),
    write: Array.from({ length: 30 }, (_, index) => 120 - index * 2),
  },
  "notes/local-timeline": [],
} satisfies Record<string, unknown>;

function loadMisskeyLocaleFixtures(): Map<string, string> {
  const fixtures = new Map<string, string>();

  if (!fs.existsSync(MISSKEY_LOCALES_DIR)) {
    return fixtures;
  }

  for (const entry of fs.readdirSync(MISSKEY_LOCALES_DIR, { withFileTypes: true })) {
    if (!entry.isFile() || !entry.name.endsWith(".json")) {
      continue;
    }

    fixtures.set(entry.name, fs.readFileSync(path.join(MISSKEY_LOCALES_DIR, entry.name), "utf-8"));
  }

  return fixtures;
}

export async function setupMisskeyMocks(page: Page): Promise<void> {
  const localeFixtures = loadMisskeyLocaleFixtures();

  await page.route("**/assets/locales/*.json", (route) => {
    const fileName = path.basename(new URL(route.request().url()).pathname);
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: localeFixtures.get(fileName) ?? FALLBACK_LOCALE,
    });
  });

  await page.route("**/sw.js", (route) => {
    return route.fulfill({
      status: 200,
      contentType: "application/javascript",
      body: MISSKEY_SW_STUB,
    });
  });

  await page.route("**/twemoji/*.svg", (route) => {
    return route.fulfill({
      status: 200,
      contentType: "image/svg+xml",
      body: EMPTY_SVG,
    });
  });

  await page.route("**/fluent-emoji/*.png", (route) => {
    return route.fulfill({
      status: 200,
      contentType: "image/png",
      body: TRANSPARENT_PNG,
    });
  });

  await page.addInitScript((apiFixtures) => {
    const _origFetch = window.fetch.bind(window);
    const serviceWorkerRegistration = {
      active: {
        postMessage() {},
      },
      unregister: async () => true,
      update: async () => {},
      scope: "/",
    };

    if ("serviceWorker" in navigator) {
      navigator.serviceWorker.register = (async () =>
        serviceWorkerRegistration) as typeof navigator.serviceWorker.register;
      navigator.serviceWorker.getRegistrations = (async () => [
        serviceWorkerRegistration,
      ]) as typeof navigator.serviceWorker.getRegistrations;
      Object.defineProperty(navigator.serviceWorker, "controller", {
        configurable: true,
        value: serviceWorkerRegistration.active,
      });
      Object.defineProperty(navigator.serviceWorker, "ready", {
        configurable: true,
        value: Promise.resolve(serviceWorkerRegistration),
      });
    }

    window.fetch = function (input, init) {
      const url =
        typeof input === "string"
          ? new URL(input, window.location.href)
          : input instanceof URL
            ? input
            : new URL(input.url, window.location.href);

      if (!url.pathname.startsWith("/api/")) {
        return _origFetch(input, init);
      }

      const endpoint = url.pathname.slice("/api/".length);
      let body: unknown = {};

      if (Object.prototype.hasOwnProperty.call(apiFixtures, endpoint)) {
        body = apiFixtures[endpoint as keyof typeof apiFixtures];
      } else if (endpoint.startsWith("notes/")) {
        body = [];
      } else if (endpoint.startsWith("charts/")) {
        body = {
          read: [],
          write: [],
        };
      }

      return Promise.resolve(
        new Response(JSON.stringify(body), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    } as typeof window.fetch;
  }, MISSKEY_API_FIXTURES);
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
