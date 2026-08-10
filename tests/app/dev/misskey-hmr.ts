import { expect, type Page, type Response } from "@playwright/test";
import assert from "node:assert/strict";
import type { ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

import { collectConsoleErrors, isFatalError } from "../../_helpers/assertions";
import { getProcessLogs } from "../../_helpers/server";
import { installSourceRestores } from "./source-restore";

interface HmrTarget {
  expectedSha256: string;
  marker: string;
  moduleSuffix: string;
  originalAnchor: string;
  sourceRelativePath: string;
  updatedAnchor: string;
}

const targets = [
  {
    expectedSha256: "3427b052175dc48cf90c9ba8909ef9557e588cb4a97a8c93328147e589f26506",
    marker: "data-vize-hmr-direct",
    moduleSuffix: "/src/ui/visitor.vue.ts",
    originalAnchor: '<div :class="$style.root">',
    sourceRelativePath: "src/ui/visitor.vue",
    updatedAnchor: '<div :class="$style.root" data-vize-hmr-direct="updated">',
  },
  {
    expectedSha256: "0d0f7fe5b2623c58b18fca7ed2a99d6645a55e78e5582bd50ad14ade3e8ee545",
    marker: "data-vize-hmr-dependency",
    moduleSuffix: "/src/components/MkVisitorDashboard.vue.ts",
    originalAnchor: '<div v-if="instance" :class="$style.root">',
    sourceRelativePath: "src/components/MkVisitorDashboard.hmr.html",
    updatedAnchor: '<div v-if="instance" :class="$style.root" data-vize-hmr-dependency="updated">',
  },
] as const satisfies readonly HmrTarget[];

const DASHBOARD_OWNER_RELATIVE_PATH = "src/components/MkVisitorDashboard.vue";
const DASHBOARD_DEPENDENCY_RELATIVE_PATH = "src/components/MkVisitorDashboard.hmr.html";
const DASHBOARD_OWNER_SHA256 = "8b15393e0c6bb8783bbcbe4c18832936a2f04b3d3f5768ac5c1f3566e542a7bc";
const DASHBOARD_TEMPLATE = /<template>\n([\s\S]*?)\n<\/template>/;

export interface MisskeyHmrFixture {
  restore(): void;
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

/** Convert one pinned authored SFC to a real external-template owner before Vite starts. */
export function prepareMisskeyHmrFixture(cwd: string): MisskeyHmrFixture {
  const directPath = path.join(cwd, targets[0].sourceRelativePath);
  const ownerPath = path.join(cwd, DASHBOARD_OWNER_RELATIVE_PATH);
  const dependencyPath = path.join(cwd, DASHBOARD_DEPENDENCY_RELATIVE_PATH);
  const directSource = fs.readFileSync(directPath, "utf8");
  const ownerSource = fs.readFileSync(ownerPath, "utf8");
  assert.equal(sha256(directSource), targets[0].expectedSha256, "pinned direct SFC changed");
  assert.equal(sha256(ownerSource), DASHBOARD_OWNER_SHA256, "pinned dashboard owner changed");
  assert.equal(fs.existsSync(dependencyPath), false, "HMR dependency path must start absent");
  const match = DASHBOARD_TEMPLATE.exec(ownerSource);
  assert.ok(match, "dashboard owner must contain one inline template");
  assert.equal(ownerSource.match(new RegExp(DASHBOARD_TEMPLATE.source, "g"))?.length, 1);
  const dependencySource = match[1];
  assert.equal(sha256(dependencySource), targets[1].expectedSha256);
  const externalOwnerSource = ownerSource.replace(
    DASHBOARD_TEMPLATE,
    '<template src="./MkVisitorDashboard.hmr.html"></template>',
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

function hmrUrlMatches(urlValue: string, target: HmrTarget): boolean {
  const url = new URL(urlValue);
  return (
    url.pathname.endsWith(target.moduleSuffix) &&
    url.searchParams.has("t") &&
    url.searchParams.has("vize")
  );
}

function initialModuleMatches(urlValue: string, target: HmrTarget): boolean {
  const url = new URL(urlValue);
  return (
    url.pathname.endsWith(target.moduleSuffix) &&
    url.searchParams.has("vize") &&
    !url.searchParams.has("vize-ssr")
  );
}

async function waitForFreshTransform(
  page: Page,
  target: HmrTarget,
  expectedMarker: boolean,
): Promise<Response> {
  const response = await page.waitForResponse(
    (candidate) => candidate.ok() && hmrUrlMatches(candidate.url(), target),
    { timeout: 60_000 },
  );
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
  const completedUpdates = new Map(targets.map((target) => [target.marker, [] as string[]]));
  const leakedMarkerLogs: string[] = [];
  page.on("request", (request) => {
    for (const target of targets) {
      if (hmrUrlMatches(request.url(), target)) hmrRequests.get(target.marker)?.push(request.url());
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
      const forwardResponse = waitForFreshTransform(page, target, true);
      fs.writeFileSync(sourcePath, updatedSource);
      await forwardResponse;
      await expect(locator).toBeVisible({ timeout: 60_000 });
      await expect.poll(() => completedUpdates.get(target.marker)?.length).toBe(1);
      await assertPageIdentity(page, sentinel, navigations);

      const repairedResponse = waitForFreshTransform(page, target, false);
      fs.writeFileSync(sourcePath, originalSource);
      await repairedResponse;
      await expect(locator).toHaveCount(0, { timeout: 60_000 });
      await expect.poll(() => completedUpdates.get(target.marker)?.length).toBe(2);
      await page.waitForTimeout(1_000);
      expect(hmrRequests.get(target.marker)).toHaveLength(2);
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
