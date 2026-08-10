import { expect, type Page, type Response } from "@playwright/test";
import assert from "node:assert/strict";
import type { ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

import { collectConsoleErrors, isFatalError } from "../../_helpers/assertions";
import { getProcessLogs } from "../../_helpers/server";
import {
  MISSKEY_HMR_EXTERNAL_TEMPLATE,
  MISSKEY_HMR_TARGETS,
  type MisskeyHmrTarget,
} from "./misskey-hmr-targets";
import { installSourceRestores } from "./source-restore";

const targets = MISSKEY_HMR_TARGETS;

const DASHBOARD_OWNER_RELATIVE_PATH = "src/components/MkVisitorDashboard.vue";
const DASHBOARD_DEPENDENCY_RELATIVE_PATH = "src/components/MkVisitorDashboard.hmr.html";
const DASHBOARD_OWNER_SHA256 = "8b15393e0c6bb8783bbcbe4c18832936a2f04b3d3f5768ac5c1f3566e542a7bc";
const DASHBOARD_TEMPLATE = /<template>\n([\s\S]*?)\n<\/template>/;
// Chokidar coalesces same-path `change` events inside this window. Start the
// distinct repair save in the next window even on fast Linux runners.
const VITE_CHANGE_EVENT_WINDOW_MS = 50;
export interface MisskeyHmrFixture {
  restore(): void;
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

function targetByMarker(marker: string): MisskeyHmrTarget {
  const target = targets.find((candidate) => candidate.marker === marker);
  assert.ok(target, `unknown Misskey HMR target marker: ${marker}`);
  return target;
}

const directTarget = targetByMarker("data-vize-hmr-direct");
const dependencyTarget = targetByMarker("data-vize-hmr-dependency");

/** Convert one pinned authored SFC to a real external-template owner before Vite starts. */
export function prepareMisskeyHmrFixture(cwd: string): MisskeyHmrFixture {
  assert.equal(DASHBOARD_DEPENDENCY_RELATIVE_PATH, dependencyTarget.sourceRelativePath);
  const directPath = path.join(cwd, directTarget.sourceRelativePath);
  const ownerPath = path.join(cwd, DASHBOARD_OWNER_RELATIVE_PATH);
  const dependencyPath = path.join(cwd, DASHBOARD_DEPENDENCY_RELATIVE_PATH);
  const directSource = fs.readFileSync(directPath, "utf8");
  const ownerSource = fs.readFileSync(ownerPath, "utf8");
  assert.equal(sha256(directSource), directTarget.expectedSha256, "pinned direct SFC changed");
  assert.equal(sha256(ownerSource), DASHBOARD_OWNER_SHA256, "pinned dashboard owner changed");
  assert.equal(fs.existsSync(dependencyPath), false, "HMR dependency path must start absent");
  const match = DASHBOARD_TEMPLATE.exec(ownerSource);
  assert.ok(match, "dashboard owner must contain one inline template");
  assert.equal(ownerSource.match(new RegExp(DASHBOARD_TEMPLATE.source, "g"))?.length, 1);
  const dependencySource = match[1];
  assert.equal(sha256(dependencySource), dependencyTarget.expectedSha256);
  const externalOwnerSource = ownerSource.replace(
    DASHBOARD_TEMPLATE,
    MISSKEY_HMR_EXTERNAL_TEMPLATE,
  );
  assert.notEqual(externalOwnerSource, ownerSource);

  const guard = installSourceRestores([
    { sourcePath: directPath, originalSource: directSource },
    { sourcePath: ownerPath, originalSource: ownerSource },
    { sourcePath: dependencyPath, originalSource: null },
  ]);
  try {
    fs.writeFileSync(dependencyPath, dependencySource);
    fs.writeFileSync(ownerPath, externalOwnerSource);
  } catch (error) {
    guard.restore();
    guard.detach();
    throw error;
  }

  return {
    restore() {
      guard.restore();
      guard.detach();
      assert.equal(fs.readFileSync(directPath, "utf8"), directSource);
      assert.equal(fs.readFileSync(ownerPath, "utf8"), ownerSource);
      assert.equal(fs.existsSync(dependencyPath), false);
    },
  };
}

function hmrUrlMatches(urlValue: string, target: MisskeyHmrTarget): boolean {
  const url = new URL(urlValue);
  return (
    url.pathname.endsWith(target.moduleSuffix) &&
    url.searchParams.has("t") &&
    url.searchParams.has("vize")
  );
}

function moduleUrlMatches(urlValue: string, target: MisskeyHmrTarget): boolean {
  return new URL(urlValue).pathname.endsWith(target.moduleSuffix);
}

function initialModuleMatches(urlValue: string, target: MisskeyHmrTarget): boolean {
  const url = new URL(urlValue);
  return (
    url.pathname.endsWith(target.moduleSuffix) &&
    url.searchParams.has("vize") &&
    !url.searchParams.has("vize-ssr")
  );
}

async function waitForNextChangeEventWindow(
  frames: readonly string[],
  target: MisskeyHmrTarget,
): Promise<void> {
  let latestTimestamp = 0;
  for (const frame of frames) {
    let payload: unknown;
    try {
      payload = JSON.parse(frame);
    } catch {
      continue;
    }
    if (payload == null || typeof payload !== "object" || !("updates" in payload)) continue;
    const updates = (payload as { updates?: unknown }).updates;
    if (!Array.isArray(updates)) continue;
    for (const update of updates) {
      if (update == null || typeof update !== "object") continue;
      const candidate = update as { path?: unknown; timestamp?: unknown };
      if (
        typeof candidate.path === "string" &&
        candidate.path.includes(target.moduleSuffix) &&
        typeof candidate.timestamp === "number"
      ) {
        latestTimestamp = Math.max(latestTimestamp, candidate.timestamp);
      }
    }
  }
  assert.notEqual(latestTimestamp, 0, `missing Vite update timestamp for ${target.moduleSuffix}`);
  const remaining = latestTimestamp + VITE_CHANGE_EVENT_WINDOW_MS + 1 - Date.now();
  if (remaining > 0) {
    await new Promise<void>((resolve) => setTimeout(resolve, remaining));
  }
}

async function waitForFreshTransform(
  page: Page,
  target: MisskeyHmrTarget,
  expectedMarker: boolean,
  observations: {
    frames: readonly string[];
    requests: readonly string[];
    responses: readonly string[];
  },
): Promise<Response> {
  let response: Response;
  try {
    response = await page.waitForResponse((candidate) => hmrUrlMatches(candidate.url(), target), {
      timeout: 60_000,
    });
  } catch (error) {
    const relevantFrames = observations.frames.filter((frame) =>
      frame.includes(target.moduleSuffix),
    );
    throw new Error(
      [
        `No fresh HMR transform reached the browser for ${target.moduleSuffix}.`,
        `requests=${JSON.stringify(observations.requests)}`,
        `responses=${JSON.stringify(observations.responses)}`,
        `viteFrames=${JSON.stringify(relevantFrames)}`,
      ].join("\n"),
      { cause: error },
    );
  }
  expect(
    response.ok(),
    `${target.moduleSuffix} HMR response ${response.status()} ${response.statusText()}`,
  ).toBe(true);
  const body = await response.text();
  if (expectedMarker) expect(body).toContain(target.marker);
  else expect(body).not.toContain(target.marker);
  return response;
}

async function assertPageIdentity(
  page: Page,
  sentinel: string,
  navigations: number,
): Promise<void> {
  expect(
    await page.evaluate(
      () => (window as Window & { __vizeViteHmrProbe?: string }).__vizeViteHmrProbe,
    ),
  ).toBe(sentinel);
  expect(navigations).toBe(0);
}

export async function verifyMisskeyAuthoredSourceHmr(options: {
  appName: string;
  cwd: string;
  devServer: ChildProcess;
  goto: (pathname: string) => Promise<unknown>;
  mountSelector: string;
  page: Page;
}): Promise<void> {
  const { appName, cwd, devServer, goto, mountSelector, page } = options;
  const sources = targets.map((target) => {
    const sourcePath = path.join(cwd, target.sourceRelativePath);
    const originalSource = fs.readFileSync(sourcePath, "utf8");
    expect(sha256(originalSource), `${target.sourceRelativePath} pinned source hash`).toBe(
      target.expectedSha256,
    );
    expect(originalSource.split(target.originalAnchor)).toHaveLength(2);
    const updatedSource = originalSource.replace(target.originalAnchor, target.updatedAnchor);
    expect(updatedSource).toContain(target.marker);
    return { originalSource, sourcePath, target, updatedSource };
  });

  await page.setViewportSize({ width: 1440, height: 900 });
  const consoleErrors = await collectConsoleErrors(page, appName);
  const hmrFrames: string[] = [];
  page.on("websocket", (socket) => {
    socket.on("framereceived", (event) => {
      const payload =
        typeof event.payload === "string" ? event.payload : event.payload.toString("utf8");
      if (payload.includes('"type":"update"')) hmrFrames.push(payload);
    });
  });
  const initialModules = targets.map((target) =>
    page.waitForResponse(
      (response) => response.ok() && initialModuleMatches(response.url(), target),
      { timeout: 120_000 },
    ),
  );
  await goto("");
  await Promise.all(initialModules);
  await expect(page.locator(mountSelector)).toBeAttached();
  await expect(page.locator("[data-cy-signup]")).toBeVisible();

  const hmrRequests = new Map(targets.map((target) => [target.marker, [] as string[]]));
  const hmrResponses = new Map(targets.map((target) => [target.marker, [] as string[]]));
  const moduleRequests = new Map(targets.map((target) => [target.marker, [] as string[]]));
  const moduleResponses = new Map(targets.map((target) => [target.marker, [] as string[]]));
  const completedUpdates = new Map(targets.map((target) => [target.marker, [] as string[]]));
  const leakedMarkerLogs: string[] = [];
  page.on("request", (request) => {
    for (const target of targets) {
      if (moduleUrlMatches(request.url(), target)) {
        moduleRequests.get(target.marker)?.push(request.url());
      }
      if (hmrUrlMatches(request.url(), target)) hmrRequests.get(target.marker)?.push(request.url());
    }
  });
  page.on("response", (response) => {
    for (const target of targets) {
      if (moduleUrlMatches(response.url(), target)) {
        moduleResponses
          .get(target.marker)
          ?.push(`${response.status()} ${response.statusText()} ${response.url()}`);
      }
      if (hmrUrlMatches(response.url(), target)) {
        hmrResponses
          .get(target.marker)
          ?.push(`${response.status()} ${response.statusText()} ${response.url()}`);
      }
    }
  });
  page.on("console", (message) => {
    for (const target of targets) {
      if (
        message.text().includes("[vite] hot updated:") &&
        message.text().includes(target.marker)
      ) {
        leakedMarkerLogs.push(message.text());
      }
      if (
        message.text().includes("[vite] hot updated:") &&
        message.text().includes(target.moduleSuffix)
      ) {
        completedUpdates.get(target.marker)?.push(message.text());
      }
    }
  });
  let navigations = 0;
  page.on("framenavigated", (frame) => {
    if (frame === page.mainFrame()) navigations += 1;
  });
  const sentinel = `vite-hmr-${Date.now()}`;
  await page.evaluate((value) => {
    (window as Window & { __vizeViteHmrProbe?: string }).__vizeViteHmrProbe = value;
  }, sentinel);

  const updateLogStart = getProcessLogs(devServer).length;
  try {
    for (const source of sources) {
      const { originalSource, sourcePath, target, updatedSource } = source;
      const locator = page.locator(`[${target.marker}="updated"]`);
      await expect(locator).toHaveCount(0);
      const observations = {
        frames: hmrFrames,
        requests: moduleRequests.get(target.marker) ?? [],
        responses: moduleResponses.get(target.marker) ?? [],
      };
      const forwardResponse = waitForFreshTransform(page, target, true, observations);
      fs.writeFileSync(sourcePath, updatedSource);
      await forwardResponse;
      await expect(locator).toBeVisible({ timeout: 60_000 });
      await expect.poll(() => completedUpdates.get(target.marker)?.length).toBe(1);
      await assertPageIdentity(page, sentinel, navigations);
      await waitForNextChangeEventWindow(hmrFrames, target);

      const repairedResponse = waitForFreshTransform(page, target, false, observations);
      fs.writeFileSync(sourcePath, originalSource);
      await repairedResponse;
      await expect(locator).toHaveCount(0, { timeout: 60_000 });
      await expect.poll(() => completedUpdates.get(target.marker)?.length).toBe(2);
      await page.waitForTimeout(1_000);
      expect(hmrRequests.get(target.marker)).toHaveLength(2);
      expect(hmrResponses.get(target.marker)).toHaveLength(2);
      expect(completedUpdates.get(target.marker)).toHaveLength(2);
      await assertPageIdentity(page, sentinel, navigations);
    }
  } finally {
    for (const source of sources) {
      if (fs.readFileSync(source.sourcePath, "utf8") !== source.originalSource) {
        fs.writeFileSync(source.sourcePath, source.originalSource);
      }
    }
  }

  for (const source of sources) {
    expect(fs.readFileSync(source.sourcePath, "utf8")).toBe(source.originalSource);
  }
  const updateLogs = getProcessLogs(devServer).slice(updateLogStart).join("\n");
  for (const target of targets) {
    expect(
      updateLogs
        .split(/\r?\n/)
        .filter((line) => line.includes("hmr update") && line.includes(target.moduleSuffix)),
      `${target.moduleSuffix} must update exactly once forward and once on repair`,
    ).toHaveLength(2);
  }
  expect(updateLogs).not.toMatch(/page reload/i);
  expect(
    leakedMarkerLogs,
    "Vite HMR logs must reference module ids, not authored markers",
  ).toHaveLength(0);
  expect(consoleErrors.filter(isFatalError)).toHaveLength(0);
}
