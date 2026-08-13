import type { Page, Route } from "@playwright/test";
import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { MISSKEY_WORK_DIR } from "./apps.ts";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const MISSKEY_WORKTREE_INSTANCE_DIR = path.dirname(MISSKEY_WORK_DIR);
const MISSKEY_LOCALES_DIRS = [
  path.join(MISSKEY_WORKTREE_INSTANCE_DIR, "vrt-reference", "misskey"),
  path.join(MISSKEY_WORKTREE_INSTANCE_DIR, "vrt-candidate", "misskey"),
  MISSKEY_WORK_DIR,
].map((dir) => path.resolve(dir, "built/_frontend_dist_/locales"));
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
export const MISSKEY_TEST_ACCOUNT = {
  id: "9testuser0000001",
  token: "test-token",
  username: "alice",
  name: "Alice",
  host: null,
  avatarUrl: null,
  avatarBlurhash: null,
  avatarDecorations: [],
  bannerUrl: null,
  bannerBlurhash: null,
  backgroundImageUrl: null,
  birthday: "2020-01-01",
  createdAt: "2020-01-01T00:00:00.000Z",
  description: "VRT fixture account",
  emojis: {},
  fields: [],
  followersCount: 42,
  followingCount: 24,
  isAdmin: true,
  isBot: false,
  isCat: false,
  isDeleted: false,
  isFollowed: false,
  isFollowing: false,
  isLocked: false,
  isModerator: true,
  isSilenced: false,
  isSuspended: false,
  lang: "en-US",
  loggedInDays: 10,
  memo: null,
  mutedWords: [],
  mutedInstances: [],
  noCrawle: false,
  notesCount: 128,
  onlineStatus: "online",
  pinnedNoteIds: [],
  pinnedNotes: [],
  pinnedPageId: null,
  policies: {
    chatAvailability: "available",
    canCreateContent: true,
    canHideAds: true,
    canInvite: true,
    canManageCustomEmojis: true,
    canPublicNote: true,
    canSearchNotes: true,
    canUpdateBioMedia: true,
    canUseTranslator: true,
    driveCapacityMb: 1024,
    gtlAvailable: true,
    ltlAvailable: true,
    maxFileSizeMb: 50,
    pinLimit: 5,
    rateLimitFactor: 1,
  },
  roles: [],
  badgeRoles: [],
  twoFactorEnabled: false,
  unreadAnnouncements: [],
} satisfies Record<string, unknown>;

const MISSKEY_TEST_USER = {
  ...MISSKEY_TEST_ACCOUNT,
  id: "9fixtureuser0001",
  username: "bob",
  name: "Bob",
  token: undefined,
};

const MISSKEY_TEST_NOTE = {
  id: "9testnote000001",
  createdAt: "2026-01-01T00:00:00.000Z",
  text: "Hello from the Misskey VRT fixture.",
  cw: null,
  visibility: "public",
  localOnly: false,
  renoteCount: 0,
  repliesCount: 0,
  reactions: {},
  reactionEmojis: {},
  fileIds: [],
  files: [],
  user: MISSKEY_TEST_USER,
  userId: MISSKEY_TEST_USER.id,
};

const EMPTY_PAGE = {
  id: "9testpage000001",
  createdAt: "2026-01-01T00:00:00.000Z",
  updatedAt: "2026-01-01T00:00:00.000Z",
  title: "Fixture page",
  name: "fixture",
  summary: "Fixture page",
  content: [],
  variables: [],
  eyeCatchingImage: null,
  user: MISSKEY_TEST_USER,
  userId: MISSKEY_TEST_USER.id,
};

const EMPTY_CHANNEL = {
  id: "9testchannel001",
  createdAt: "2026-01-01T00:00:00.000Z",
  updatedAt: "2026-01-01T00:00:00.000Z",
  name: "Fixture channel",
  description: "Fixture channel",
  userId: MISSKEY_TEST_USER.id,
  bannerUrl: null,
  isFollowing: false,
  usersCount: 0,
  notesCount: 0,
};

const EMPTY_DRIVE_FILE = {
  id: "9testfile000001",
  createdAt: "2026-01-01T00:00:00.000Z",
  name: "fixture.png",
  type: "image/png",
  md5: "fixture",
  size: 1,
  isSensitive: false,
  blurhash: null,
  properties: {},
  url: null,
  thumbnailUrl: null,
  folderId: null,
  userId: MISSKEY_TEST_USER.id,
};

const MISSKEY_API_FIXTURES = {
  i: MISSKEY_TEST_ACCOUNT,
  meta: {
    name: "Misskey",
    uri: "http://localhost:3000",
    version: "2026.2.0-beta.0",
    description: "A Misskey instance",
    ads: [],
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
    serverRules: [],
    maxNoteTextLength: 3000,
    features: {
      registration: true,
      localTimeline: true,
      globalTimeline: true,
      miauth: true,
    },
  },
  "i/notifications": [],
  "i/registry/get-all": {
    accountSetupWizard: -1,
    widgets: [],
  },
  "i/apps": [],
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
  "notes/global-timeline": [],
  "notes/hybrid-timeline": [],
  "notes/timeline": [],
  "notes/user-list-timeline": [],
  "notes/mentions": [],
  "notes/children": [],
  "notes/replies": [],
  "notes/featured": [],
  "notes/search": [],
  "notes/search-by-tag": [],
  "notes/show": MISSKEY_TEST_NOTE,
  "users/show": MISSKEY_TEST_USER,
  "users/notes": [],
  "users/clips": [],
  "users/flashs": [],
  "users/gallery/posts": [],
  "users/following": [],
  "users/followers": [],
  "users/search": [],
  "users/search-by-username-and-host": [],
  users: [],
  "pinned-users": [],
  announcements: [],
  "announcements/show": {
    id: "9announcement001",
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
    title: "Fixture announcement",
    text: "Fixture announcement",
    imageUrl: null,
    display: "normal",
    forYou: false,
    isRead: false,
  },
  "clips/list": [],
  "clips/show": {
    id: "9testclip000001",
    createdAt: "2026-01-01T00:00:00.000Z",
    name: "Fixture clip",
    description: "Fixture clip",
    isPublic: true,
    user: MISSKEY_TEST_USER,
    userId: MISSKEY_TEST_USER.id,
  },
  "channels/featured": [],
  "channels/followed": [],
  "channels/owned": [],
  "channels/search": [],
  "channels/show": EMPTY_CHANNEL,
  drive: [],
  "drive/files": [],
  "drive/files/show": EMPTY_DRIVE_FILE,
  "drive/folders": [],
  "drive/folders/show": {
    id: "9testfolder0001",
    createdAt: "2026-01-01T00:00:00.000Z",
    name: "Fixture folder",
    parentId: null,
  },
  "federation/instances": [],
  "federation/show-instance": {
    id: "9testinstance001",
    firstRetrievedAt: "2026-01-01T00:00:00.000Z",
    infoUpdatedAt: "2026-01-01T00:00:00.000Z",
    latestRequestReceivedAt: null,
    host: "example.com",
    name: "Example",
    softwareName: "misskey",
    softwareVersion: "2026.2.0-beta.0",
    maintainerName: "Fixture admin",
    maintainerEmail: "admin@example.com",
    description: "Fixture federated instance",
    followingCount: 0,
    followersCount: 0,
    isBlocked: false,
    isSilenced: false,
    isMediaSilenced: false,
    suspensionState: "none",
    moderationNote: "",
    faviconUrl: null,
    iconUrl: null,
  },
  "flash/featured": [],
  "flash/my": [],
  "flash/liked": [],
  "flash/show": {
    id: "9testflash0001",
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
    title: "Fixture Play",
    summary: "Fixture Play",
    script: "",
    visibility: "public",
    user: MISSKEY_TEST_USER,
    userId: MISSKEY_TEST_USER.id,
  },
  "gallery/featured": [],
  "gallery/posts": [],
  "gallery/posts/show": {
    id: "9testgallery001",
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
    title: "Fixture gallery",
    description: "Fixture gallery",
    files: [],
    user: MISSKEY_TEST_USER,
    userId: MISSKEY_TEST_USER.id,
  },
  "hashtags/trend": [],
  pages: [],
  "pages/show": EMPTY_PAGE,
  "roles/list": [],
  "roles/users": [],
  "roles/show": {
    id: "9testrole000001",
    createdAt: "2026-01-01T00:00:00.000Z",
    updatedAt: "2026-01-01T00:00:00.000Z",
    name: "Fixture role",
    description: "Fixture role",
    color: null,
    iconUrl: null,
    displayOrder: 0,
    target: "manual",
    condFormula: {},
    isModerator: false,
    isAdministrator: false,
    isPublic: true,
    isExplorable: true,
    asBadge: false,
    usersCount: 0,
  },
  "hashtags/list": [],
  "hashtags/users": [],
  "admin/meta": {
    blockedHosts: [],
    silencedHosts: [],
    serverRules: [],
    ads: [],
    backgroundImageUrl: null,
    iconUrl: null,
    proxyAccountId: null,
  },
  "admin/show-users": [],
  "admin/show-moderation-logs": [],
} satisfies Record<string, unknown>;

function getMisskeyApiFixture(endpoint: string): unknown {
  if (Object.prototype.hasOwnProperty.call(MISSKEY_API_FIXTURES, endpoint)) {
    return MISSKEY_API_FIXTURES[endpoint as keyof typeof MISSKEY_API_FIXTURES];
  }

  if (endpoint.startsWith("notes/")) {
    return [];
  }

  if (endpoint.startsWith("charts/")) {
    return {
      read: [],
      write: [],
    };
  }

  if (
    endpoint.endsWith("/list") ||
    endpoint.endsWith("/search") ||
    endpoint.includes("timeline") ||
    endpoint.includes("notifications") ||
    endpoint.includes("requests") ||
    endpoint.includes("following") ||
    endpoint.includes("followers") ||
    endpoint.includes("files") ||
    endpoint.includes("folders") ||
    endpoint.includes("featured") ||
    endpoint.includes("owned") ||
    endpoint.includes("joined") ||
    endpoint.includes("liked")
  ) {
    return [];
  }

  return {};
}

async function fulfillMisskeyApiRoute(route: Route): Promise<void> {
  const endpoint = new URL(route.request().url()).pathname.slice("/api/".length);
  await route.fulfill({
    status: 200,
    contentType: "application/json",
    body: JSON.stringify(getMisskeyApiFixture(endpoint)),
  });
}

async function fulfillMisskeyServiceWorkerRoute(route: Route): Promise<void> {
  await route.fulfill({
    status: 200,
    contentType: "application/javascript",
    body: MISSKEY_SW_STUB,
  });
}

function loadMisskeyLocaleFixtures(): Map<string, string> {
  const fixtures = new Map<string, string>();

  for (const localesDir of MISSKEY_LOCALES_DIRS) {
    if (!fs.existsSync(localesDir)) {
      continue;
    }

    for (const entry of fs.readdirSync(localesDir, { withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith(".json") || fixtures.has(entry.name)) {
        continue;
      }

      fixtures.set(entry.name, fs.readFileSync(path.join(localesDir, entry.name), "utf-8"));
    }
  }

  return fixtures;
}

export async function setupMisskeyMocks(page: Page): Promise<void> {
  const localeFixtures = loadMisskeyLocaleFixtures();

  await page.context().route("**/api/**", fulfillMisskeyApiRoute);
  await page.context().route("**/sw.js", fulfillMisskeyServiceWorkerRoute);

  await page.route("**/assets/locales/*.json", (route) => {
    const fileName = path.basename(new URL(route.request().url()).pathname);
    return route.fulfill({
      status: 200,
      contentType: "application/json",
      body: localeFixtures.get(fileName) ?? FALLBACK_LOCALE,
    });
  });

  await page.route("**/sw.js", (route) => {
    return fulfillMisskeyServiceWorkerRoute(route);
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
      } else if (
        endpoint.endsWith("/list") ||
        endpoint.endsWith("/search") ||
        endpoint.includes("timeline") ||
        endpoint.includes("notifications") ||
        endpoint.includes("requests") ||
        endpoint.includes("following") ||
        endpoint.includes("followers") ||
        endpoint.includes("files") ||
        endpoint.includes("folders") ||
        endpoint.includes("featured") ||
        endpoint.includes("owned") ||
        endpoint.includes("joined") ||
        endpoint.includes("liked")
      ) {
        body = [];
      }

      return Promise.resolve(
        new Response(JSON.stringify(body), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    } as typeof window.fetch;

    const serviceWorkerRegistration = {
      active: {
        postMessage() {},
      },
      unregister: async () => true,
      update: async () => {},
      scope: "/",
    };

    if ("serviceWorker" in navigator) {
      try {
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
      } catch {
        // Network-level sw.js routing still keeps the VRT environment deterministic.
      }
    }
  }, MISSKEY_API_FIXTURES);
}

// --- npmx.dev VRT mocks ---
//
// The `/org/[org]` route renders LIVE data fetched from external services: the
// npm registry org endpoint (proxied through `/api/registry/org/<org>/packages`)
// and the Algolia `npm-search` index (`getObjects` via
// `https://<appId>-dsn.algolia.net/1/indexes/*/objects`). Two independent dev
// servers (reference Vue vs candidate vize) issue these requests separately, and
// under CI/sandbox rate-limiting they receive different payloads (one shows
// packages, the other empty/loading), so the screenshots diverge. That is a test
// nondeterminism, not a render bug.
//
// To make the route deterministic we intercept both endpoints (mirroring the
// misskey `/api/**` approach) and return a fixed fixture payload for BOTH the
// reference and candidate pages so they render an identical package-list state.

interface NpmxAlgoliaHit {
  objectID: string;
  name: string;
  version: string;
  description: string | null;
  modified: number;
  homepage: string | null;
  repository: {
    url: string;
    host: string;
    user: string;
    project: string;
    path: string;
  } | null;
  owners: { name: string; email?: string; avatar?: string; link?: string }[] | null;
  downloadsLast30Days: number;
  downloadsRatio: number;
  popular: boolean;
  keywords: string[];
  deprecated: boolean | string;
  isDeprecated: boolean;
  license: string | null;
}

// A fixed point in time so `modified`-derived dates render identically.
const NPMX_FIXTURE_MODIFIED = Date.parse("2026-01-01T00:00:00.000Z");

function npmxAlgoliaHit(name: string, overrides: Partial<NpmxAlgoliaHit> = {}): NpmxAlgoliaHit {
  return {
    objectID: name,
    name,
    version: "3.5.29",
    description: `Fixture package ${name} for the npmx VRT.`,
    modified: NPMX_FIXTURE_MODIFIED,
    homepage: "https://github.com/vuejs/core",
    repository: {
      url: "https://github.com/vuejs/core",
      host: "github.com",
      user: "vuejs",
      project: "core",
      path: "",
    },
    owners: [{ name: "vuejs", email: "fixture@example.com" }],
    downloadsLast30Days: 43_000_000,
    downloadsRatio: 1,
    popular: true,
    keywords: ["vue", "framework"],
    deprecated: false,
    isDeprecated: false,
    license: "MIT",
    ...overrides,
  };
}

// Deterministic package roster for `@vue` (sorted so list ordering is stable).
const NPMX_ORG_FIXTURES: Record<string, NpmxAlgoliaHit[]> = {
  vue: [
    npmxAlgoliaHit("@vue/compiler-core", { downloadsLast30Days: 41_000_000 }),
    npmxAlgoliaHit("@vue/compiler-dom", { downloadsLast30Days: 40_000_000 }),
    npmxAlgoliaHit("@vue/compiler-sfc", { downloadsLast30Days: 39_000_000 }),
    npmxAlgoliaHit("@vue/reactivity", { downloadsLast30Days: 42_000_000 }),
    npmxAlgoliaHit("@vue/runtime-core", { downloadsLast30Days: 41_500_000 }),
    npmxAlgoliaHit("@vue/runtime-dom", { downloadsLast30Days: 41_200_000 }),
    npmxAlgoliaHit("@vue/server-renderer", { downloadsLast30Days: 30_000_000 }),
    npmxAlgoliaHit("@vue/shared", { downloadsLast30Days: 43_500_000 }),
  ],
};

function npmxOrgHits(org: string): NpmxAlgoliaHit[] {
  return NPMX_ORG_FIXTURES[org.toLowerCase()] ?? NPMX_ORG_FIXTURES.vue;
}

/**
 * Make the npmx `/org/[org]` route deterministic for visual parity by returning
 * fixed package data for both the npm-registry org proxy and the Algolia
 * `getObjects` lookup. Wire this into BOTH the reference and candidate pages.
 */
export async function setupNpmxOrgMocks(page: Page): Promise<void> {
  // 1) npm-registry org package list (server proxy, hit client-side after
  //    hydration because the page uses `useLazyAsyncData`).
  await page.context().route("**/api/registry/org/**/packages", async (route) => {
    const match = /\/api\/registry\/org\/([^/]+)\/packages/.exec(
      new URL(route.request().url()).pathname,
    );
    const org = match ? decodeURIComponent(match[1]) : "vue";
    const hits = npmxOrgHits(org);
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({ packages: hits.map((hit) => hit.name), count: hits.length }),
    });
  });

  // 2) All Algolia traffic (matched by a host regex to avoid URL-glob quirks).
  //    There are two shapes used by npmx:
  //      - `getObjects` multi-get by objectID  -> `{ results: (Hit|null)[] }`
  //        (used by `getPackagesByName`, the algolia provider path for orgs)
  //      - the lite client multi-query `search` -> `{ results: [{ hits, ... }] }`
  //        (used by the npm-search box / suggestion checks)
  //    Return the fixed org roster in whichever shape the request expects.
  await page.context().route(/\.algolia(net\.com|\.net)\//, async (route) => {
    const request = route.request();
    const pathname = new URL(request.url()).pathname;

    if (pathname.includes("/objects")) {
      let requested: string[] = [];
      try {
        const payload = request.postDataJSON() as
          | { requests?: { objectID?: string }[] }
          | undefined;
        requested = (payload?.requests ?? [])
          .map((entry) => entry.objectID)
          .filter((id): id is string => typeof id === "string");
      } catch {
        requested = [];
      }

      const byName = new Map<string, NpmxAlgoliaHit>();
      for (const hits of Object.values(NPMX_ORG_FIXTURES)) {
        for (const hit of hits) byName.set(hit.name, hit);
      }

      const results =
        requested.length > 0
          ? requested.map((name) => byName.get(name) ?? null)
          : npmxOrgHits("vue");

      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({ results }),
      });
      return;
    }

    const hits = npmxOrgHits("vue");
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        results: [
          {
            hits,
            nbHits: hits.length,
            page: 0,
            nbPages: 1,
            hitsPerPage: hits.length,
            query: "",
            params: "",
            index: "npm-search",
          },
        ],
      }),
    });
  });

  // 3) npm-provider fallback path (`?p=npm`) hits the lightweight package-meta
  //    proxy; keep it deterministic as well.
  await page.context().route("**/api/registry/package-meta/**", async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    const name = decodeURIComponent(
      pathname.slice(pathname.indexOf("/package-meta/") + "/package-meta/".length),
    );
    const hit = npmxOrgHits("vue").find((entry) => entry.name === name) ?? npmxAlgoliaHit(name);
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify({
        name: hit.name,
        version: hit.version,
        description: hit.description ?? "",
        keywords: hit.keywords,
        license: hit.license,
        date: new Date(hit.modified).toISOString(),
        links: { npm: `https://www.npmjs.com/package/${hit.name}` },
        maintainers: hit.owners?.map((owner) => ({ name: owner.name, email: owner.email })) ?? [],
        weeklyDownloads: Math.round(hit.downloadsLast30Days / 4.3),
      }),
    });
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
