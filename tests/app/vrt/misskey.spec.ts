import { test, expect, type Browser, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createMisskeyVisualParityApps, type AppConfig } from "../../_helpers/apps";
import {
  ensurePortFree,
  killProcess,
  startDevServer,
  waitForHttpReady,
  waitForServerReady,
} from "../../_helpers/server";
import { setupMisskeyMocks, MISSKEY_TEST_ACCOUNT } from "../../_helpers/mocking";
import {
  expectVisualParity,
  installVisualStabilityHooks,
  prepareStableVisualState,
} from "../../_helpers/visual-parity";
import { waitForMountedAppContent } from "../../_helpers/assertions";

interface VisualRoute {
  account?: boolean;
  maxDiffRatio?: number;
  name: string;
  path: string;
  ready?: (page: Page) => Promise<void>;
  viewport?: { height: number; width: number };
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR =
  process.env.VIZE_MISSKEY_VRT_OUTPUT_DIR ??
  path.resolve(__dirname, "../../../__agent_only/misskey-vrt/artifacts");
const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
const MOBILE_VIEWPORT = { width: 390, height: 844 };
const MISSKEY_VRT_TIMEOUT = 900_000;
const apps = createMisskeyVisualParityApps();

const publicRoutes: VisualRoute[] = [
  { name: "visitor-home", path: "/" },
  { name: "visitor-mobile-home", path: "/", viewport: MOBILE_VIEWPORT },
  { name: "timeline", path: "/timeline" },
  { name: "user-profile", path: "/@bob" },
  { name: "user-profile-notes", path: "/@bob/notes" },
  { name: "user-following", path: "/@bob/following" },
  { name: "user-followers", path: "/@bob/followers" },
  { name: "note", path: "/notes/9testnote000001" },
  { name: "instance-info", path: "/instance-info/example.com" },
  { name: "announcements", path: "/announcements" },
  { name: "announcement", path: "/announcements/9announcement001" },
  { name: "about", path: "/about" },
  { name: "about-emojis", path: "/about#emojis" },
  { name: "about-federation", path: "/about#federation" },
  { name: "contact", path: "/contact" },
  { name: "about-misskey", path: "/about-misskey" },
  { name: "ads", path: "/ads" },
  { name: "role", path: "/roles/9testrole000001" },
  { name: "user-tag", path: "/user-tags/core" },
  { name: "explore", path: "/explore" },
  { name: "explore-roles", path: "/explore#roles" },
  { name: "search", path: "/search?q=hello" },
  { name: "preview", path: "/preview" },
  { name: "auth", path: "/auth/test-token" },
  { name: "miauth", path: "/miauth/test-session?name=Fixture" },
  { name: "oauth", path: "/oauth/authorize" },
  { name: "tag", path: "/tags/vize" },
  { name: "pages", path: "/pages" },
  { name: "page", path: "/@bob/pages/fixture" },
  { name: "play", path: "/play" },
  { name: "play-item", path: "/play/9testflash0001" },
  { name: "gallery", path: "/gallery" },
  { name: "gallery-post", path: "/gallery/9testgallery001" },
  { name: "channels", path: "/channels" },
  { name: "channel", path: "/channels/9testchannel001" },
  { name: "custom-emojis-manager", path: "/custom-emojis-manager" },
  { name: "avatar-decorations", path: "/avatar-decorations" },
  { name: "registry", path: "/registry" },
  { name: "registry-keys", path: "/registry/keys/client/foo" },
  { name: "registry-value", path: "/registry/value/client/foo" },
  { name: "games", path: "/games" },
  { name: "reversi", path: "/reversi" },
  { name: "debug", path: "/debug" },
  { name: "not-found", path: "/not-found-route" },
];

const accountRoutes: VisualRoute[] = [
  { name: "account-home", path: "/", account: true },
  { name: "account-home-mobile", path: "/", account: true, viewport: MOBILE_VIEWPORT },
  { name: "account-deck-home", path: "/?ui=deck", account: true },
  { name: "account-zen-about", path: "/about?ui=zen", account: true },
  { name: "settings", path: "/settings", account: true },
  { name: "settings-profile", path: "/settings/profile", account: true },
  { name: "settings-avatar-decoration", path: "/settings/avatar-decoration", account: true },
  { name: "settings-privacy", path: "/settings/privacy", account: true },
  { name: "settings-emoji-palette", path: "/settings/emoji-palette", account: true },
  { name: "settings-drive", path: "/settings/drive", account: true },
  { name: "settings-drive-cleaner", path: "/settings/drive/cleaner", account: true },
  { name: "settings-notifications", path: "/settings/notifications", account: true },
  { name: "settings-email", path: "/settings/email", account: true },
  { name: "settings-security", path: "/settings/security", account: true },
  { name: "settings-preferences", path: "/settings/preferences", account: true },
  { name: "settings-theme", path: "/settings/theme", account: true },
  { name: "settings-theme-install", path: "/settings/theme/install", account: true },
  { name: "settings-theme-manage", path: "/settings/theme/manage", account: true },
  { name: "settings-navbar", path: "/settings/navbar", account: true },
  { name: "settings-statusbar", path: "/settings/statusbar", account: true },
  { name: "settings-sounds", path: "/settings/sounds", account: true },
  { name: "settings-plugin", path: "/settings/plugin", account: true },
  { name: "settings-plugin-install", path: "/settings/plugin/install", account: true },
  { name: "settings-account-data", path: "/settings/account-data", account: true },
  { name: "settings-mute-block", path: "/settings/mute-block", account: true },
  { name: "settings-connect", path: "/settings/connect", account: true },
  { name: "settings-apps", path: "/settings/apps", account: true },
  { name: "settings-webhook-new", path: "/settings/webhook/new", account: true },
  { name: "settings-deck", path: "/settings/deck", account: true },
  { name: "settings-custom-css", path: "/settings/custom-css", account: true },
  { name: "settings-profiles", path: "/settings/profiles", account: true },
  { name: "settings-accounts", path: "/settings/accounts", account: true },
  { name: "settings-other", path: "/settings/other", account: true },
  { name: "theme-editor", path: "/theme-editor", account: true },
  { name: "lookup", path: "/lookup", account: true },
  {
    name: "share",
    path: "/share?text=hello",
    account: true,
    ready: async (page) => {
      const textbox = page.getByRole("textbox", { name: "What's on your mind?" });
      await expect(textbox).toBeVisible({ timeout: 15_000 });
      if ((await textbox.inputValue()) !== "hello") {
        await textbox.fill("hello");
      }
      await expect(textbox).toHaveValue("hello");
    },
  },
  { name: "api-console", path: "/api-console", account: true },
  { name: "scratchpad", path: "/scratchpad", account: true },
  { name: "pages-new", path: "/pages/new", account: true },
  { name: "pages-edit", path: "/pages/edit/9testpage000001", account: true },
  { name: "play-new", path: "/play/new", account: true },
  { name: "play-edit", path: "/play/9testflash0001/edit", account: true },
  { name: "gallery-new", path: "/gallery/new", account: true },
  { name: "gallery-edit", path: "/gallery/9testgallery001/edit", account: true },
  { name: "channels-new", path: "/channels/new", account: true },
  { name: "channels-edit", path: "/channels/9testchannel001/edit", account: true },
  { name: "install-extensions", path: "/install-extensions", account: true },
  { name: "my-notifications", path: "/my/notifications", account: true },
  { name: "my-favorites", path: "/my/favorites", account: true },
  { name: "my-achievements", path: "/my/achievements", account: true },
  { name: "my-drive", path: "/my/drive", account: true },
  { name: "my-drive-file", path: "/my/drive/file/9testfile000001", account: true },
  { name: "my-follow-requests", path: "/my/follow-requests", account: true },
  { name: "my-lists", path: "/my/lists", account: true },
  { name: "my-list", path: "/my/lists/9testlist000001", account: true },
  { name: "my-clips", path: "/my/clips", account: true },
  { name: "my-antennas", path: "/my/antennas", account: true },
  { name: "my-antennas-create", path: "/my/antennas/create", account: true },
  { name: "timeline-list", path: "/timeline/list/9testlist000001", account: true },
  { name: "timeline-antenna", path: "/timeline/antenna/9testantenna001", account: true },
  { name: "clicker", path: "/clicker", account: true },
  { name: "bubble-game", path: "/bubble-game", account: true },
  { name: "qr", path: "/qr", account: true },
  { name: "admin", path: "/admin", account: true },
  { name: "admin-overview", path: "/admin/overview", account: true },
  { name: "admin-users", path: "/admin/users", account: true },
  { name: "admin-emojis", path: "/admin/emojis", account: true },
  { name: "admin-avatar-decorations", path: "/admin/avatar-decorations", account: true },
  { name: "admin-federation-job-queue", path: "/admin/federation-job-queue", account: true },
  { name: "admin-job-queue", path: "/admin/job-queue", account: true },
  { name: "admin-files", path: "/admin/files", account: true },
  { name: "admin-federation", path: "/admin/federation", account: true },
  { name: "admin-announcements", path: "/admin/announcements", account: true },
  { name: "admin-ads", path: "/admin/ads", account: true },
  { name: "admin-roles", path: "/admin/roles", account: true },
  { name: "admin-roles-new", path: "/admin/roles/new", account: true },
  { name: "admin-database", path: "/admin/database", account: true },
  { name: "admin-abuses", path: "/admin/abuses", account: true },
  { name: "admin-modlog", path: "/admin/modlog", account: true },
  { name: "admin-settings", path: "/admin/settings", account: true },
  { name: "admin-branding", path: "/admin/branding", account: true },
  { name: "admin-moderation", path: "/admin/moderation", account: true },
  { name: "admin-email-settings", path: "/admin/email-settings", account: true },
  { name: "admin-object-storage", path: "/admin/object-storage", account: true },
  { name: "admin-security", path: "/admin/security", account: true },
  { name: "admin-relays", path: "/admin/relays", account: true },
  { name: "admin-external-services", path: "/admin/external-services", account: true },
  { name: "admin-performance", path: "/admin/performance", account: true },
  { name: "admin-invites", path: "/admin/invites", account: true },
  { name: "admin-system-webhook", path: "/admin/system-webhook", account: true },
];

const routes = [...publicRoutes, ...accountRoutes];

test.describe("misskey visual parity", () => {
  test.describe.configure({ mode: "serial", timeout: MISSKEY_VRT_TIMEOUT });

  let candidateServer: ChildProcess | undefined;
  let referenceServer: ChildProcess | undefined;

  test.beforeAll(async () => {
    test.setTimeout(MISSKEY_VRT_TIMEOUT);
    referenceServer = await startApp(apps.reference);
    candidateServer = await startApp(apps.candidate);
  });

  test.afterAll(async () => {
    killProcess(candidateServer);
    killProcess(referenceServer);
  });

  for (const route of routes) {
    test(route.name, async ({ browser }) => {
      await compareRoute(browser, route);
    });
  }
});

async function startApp(app: AppConfig): Promise<ChildProcess> {
  if (app.setup) app.setup();
  await ensurePortFree(app.port);

  const server = startDevServer(app);
  await waitForServerReady(server, app.port, app.readyPattern, app.startupTimeout, app.readyDelay);
  await waitForHttpReady(app.url, app.port);
  return server;
}

async function compareRoute(browser: Browser, route: VisualRoute): Promise<void> {
  const context = await browser.newContext({
    colorScheme: "light",
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    viewport: route.viewport ?? DEFAULT_VIEWPORT,
  });

  try {
    const referencePage = await context.newPage();
    const candidatePage = await context.newPage();

    await Promise.all([setupPage(referencePage, route), setupPage(candidatePage, route)]);
    await Promise.all([
      openRoute(referencePage, apps.reference.url, route),
      openRoute(candidatePage, apps.candidate.url, route),
    ]);

    if (route.ready) {
      await Promise.all([route.ready(referencePage), route.ready(candidatePage)]);
    }

    await Promise.all([
      prepareStableVisualState(referencePage),
      prepareStableVisualState(candidatePage),
    ]);

    await expectVisualParity(referencePage, candidatePage, {
      maxDiffRatio: route.maxDiffRatio,
      name: route.name,
      outputDir: OUTPUT_DIR,
    });
  } finally {
    await context.close();
  }
}

async function setupPage(page: Page, route: VisualRoute): Promise<void> {
  await installVisualStabilityHooks(page);
  await setupMisskeyMocks(page);
  await page.addInitScript(
    ({ account, signedIn }) => {
      localStorage.clear();
      localStorage.setItem("lang", "en-US");
      localStorage.setItem("lastVersion", "9999.0.0");
      localStorage.setItem("hidePreferencesRestoreSuggestion", "true");
      localStorage.setItem("neverShowDonationInfo", "true");
      localStorage.setItem("modifiedVersionMustProminentlyOfferInAgplV3Section13Read", "true");
      if (signedIn) {
        localStorage.setItem("account", JSON.stringify(account));
      }
    },
    { account: MISSKEY_TEST_ACCOUNT, signedIn: route.account === true },
  );
}

async function openRoute(page: Page, baseUrl: string, route: VisualRoute): Promise<void> {
  const response = await page.goto(`${baseUrl}${route.path.replace(/^\//, "")}`, {
    timeout: 60_000,
    waitUntil: "domcontentloaded",
  });
  expect(response?.status()).toBeLessThan(500);
  await expect(page.locator("#misskey_app")).toBeAttached({ timeout: 15_000 });
  await waitForMountedAppContent(page, "#misskey_app");
  await page.waitForLoadState("networkidle", { timeout: 10_000 }).catch(() => undefined);
  await page.waitForTimeout(1000);
}
