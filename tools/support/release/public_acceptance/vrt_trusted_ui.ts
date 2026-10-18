import assert from "node:assert/strict";
import { cp, mkdir, readFile, stat } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import {
  assertSessionReport,
  inventory,
  pngDetails,
  type SessionCapture,
  type Evidence,
  type VrtReport,
} from "./vrt_artifacts.ts";
import { literalVariants, type LiteralTitle, type buildLiteralHostedVrt } from "./vrt_literal.ts";
import { sha256 } from "./vrt_fixtures.ts";

export type LiteralBuild = Awaited<ReturnType<typeof buildLiteralHostedVrt>>;
export interface PublicSession {
  endpoint: string;
  token: string;
  assertRunning(): void;
}
type HostedCapture = SessionCapture;
const sectionIds = {
  Controls: ["variant-default", "variant-custom-theme"],
  Keys: ["variant-proto", "variant-constructor", "variant-hasownproperty"],
};

/** A rebuilt gallery is on Home: select its Art before requiring component-only tabs. */
export async function selectPublicColorArt(
  page: {
    locator(selector: string): {
      filter(options: { hasText: string }): { click(): Promise<unknown> };
    };
    getByRole(
      role: "button",
      options: { name: string; exact: true },
    ): { click(): Promise<unknown> };
  },
  title: "Left" | "Right",
) {
  await page.locator(".art-item").filter({ hasText: title }).click();
  await page.getByRole("button", { name: "Variants", exact: true }).click();
}

export async function selectLiteral(
  page: Page,
  title: LiteralTitle,
  build: LiteralBuild,
  gallery: string,
) {
  await page.locator(".art-item").filter({ hasText: title }).click();
  const names = literalVariants[title];
  for (const [index, [, button]] of names.entries())
    await page
      .frameLocator(".variant-card iframe")
      .nth(index)
      .getByRole("button", { name: button, exact: true })
      .waitFor({ timeout: 15000 });
  const rendered = await page.evaluate(() =>
    [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map(
      (frame, index) => ({
        name: frame.closest(".variant-section")?.getAttribute("data-variant-name"),
        id: frame.closest(".variant-section")?.id,
        aria: document.querySelectorAll(".variant-toc-item")[index]?.getAttribute("aria-controls"),
        url: frame.src,
        buttons: [...(frame.contentDocument?.querySelectorAll("button") ?? [])].map(
          (b) => b.textContent,
        ),
        unresolved: frame.contentDocument?.querySelectorAll("museacomponent").length,
      }),
    ),
  );
  assert.deepEqual(
    rendered.map((v) => v.name),
    names.map(([name]) => name),
  );
  assert.deepEqual(
    rendered.map((v) => v.id),
    sectionIds[title],
  );
  assert.deepEqual(
    rendered.map((v) => v.aria),
    sectionIds[title],
  );
  assert.equal(new Set(rendered.map((v) => v.id)).size, names.length);
  const art = path.join(build.root, "src", `${title}.art.vue`);
  for (const [index, [name, button]] of names.entries()) {
    assert.equal(rendered[index].url, new URL(build.manifest.previews[art][name], gallery).href);
    assert.equal(rendered[index].unresolved, 0);
    assert.ok(rendered[index].buttons.includes(button));
  }
  await page.locator(".variant-toc-item").last().click();
  await page.locator(".variant-toc-item").first().click();
  await page.locator('.variant-toc-item[aria-current="true"]').first().waitFor();
  return rendered;
}

export async function connectPublicSession(page: Page, session: PublicSession) {
  session.assertRunning();
  await page.getByRole("textbox", { name: "VRT endpoint", exact: true }).fill(session.endpoint);
  await page.getByLabel("Session token", { exact: true }).fill(session.token);
  await page.getByRole("button", { name: "Connect VRT", exact: true }).click();
  await page.getByText("Connected to your local VRT session.", { exact: true }).waitFor();
  assert.equal(await page.getByLabel("Session token", { exact: true }).count(), 0);
  session.assertRunning();
}

export async function wholeLiteralReports(root: string, title: LiteralTitle) {
  const result: Partial<Record<"json" | "html", Buffer>> = {};
  for (const kind of ["json", "html"] as const) {
    try {
      result[kind] = await readFile(
        path.join(root, "trusted-vrt-output/reports", `vrt-${title}-report.${kind}`),
      );
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
  }
  return result;
}

/** Actual authenticated bytes; no bearer/header persists in evidence or public URLs. */
export async function publicArtifact(
  session: PublicSession,
  origin: string,
  route: string,
  type: string,
) {
  session.assertRunning();
  assert.match(route, /^\/artifacts\/[0-9a-f-]{36}$/);
  const response = await fetch(`${session.endpoint}${route}`, {
    redirect: "error",
    signal: AbortSignal.timeout(30000),
    headers: { Origin: origin, Authorization: `Bearer ${session.token}` },
  });
  assert.equal(response.status, 200);
  assert.equal(response.headers.get("Content-Type"), type);
  const bytes = Buffer.from(await response.arrayBuffer());
  session.assertRunning();
  return bytes;
}

/** Authored names and counts complement the strict public summary and artifact checks. */
export function literalCaptureLaw(
  data: HostedCapture,
  title: LiteralTitle,
  artPath: string,
  phase: "new" | "match",
) {
  const names = literalVariants[title].map(([name]) => name),
    total = names.length;
  assert.equal(data.success, true);
  assert.deepEqual(
    [
      data.summary.total,
      data.summary.new,
      data.summary.passed,
      data.summary.failed,
      data.summary.errors,
      data.summary.skipped,
    ],
    [total, phase === "new" ? total : 0, phase === "match" ? total : 0, 0, 0, 0],
  );
  assert.ok(Number.isFinite(data.summary.duration) && data.summary.duration >= 0);
  assert.deepEqual(
    data.results.map((r) => r.variantName),
    names,
  );
  for (const result of data.results) {
    assert.equal(result.artPath, artPath);
    assert.equal(result.viewport, "authored-compact");
    assert.equal(result.passed, true);
    assert.equal(result.error, undefined);
    assert.deepEqual(Object.keys(result.images), ["snapshot", "current"]);
    if (phase === "new") {
      assert.equal(result.isNew, true);
      assert.equal(result.diffPercentage, undefined);
    } else {
      assert.equal(Object.hasOwn(result, "isNew"), false);
      assert.equal(result.diffPercentage, 0);
    }
  }
}

export async function capturePublicLiteral(input: {
  page: Page;
  session: PublicSession;
  build: LiteralBuild;
  gallery: string;
  title: LiteralTitle;
  phase: "new" | "match";
  label: string;
  e: Evidence;
  first?: Buffer[];
}) {
  const { page, session, build, gallery, title, phase, e } = input;
  const artPath = path.join(build.root, "src", `${title}.art.vue`);
  const prefix = `trusted-public/${input.label}/${title}`;
  session.assertRunning();
  const [response] = await Promise.all([
    page.waitForResponse(
      (r) => r.url() === `${session.endpoint}/capture` && r.request().method() === "POST",
    ),
    page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  ]);
  assert.deepEqual(response.request().postDataJSON(), { artPath, update: false });
  const raw = await response.body();
  await e.save(`${prefix}/response.json`, raw);
  assert.equal(response.status(), 200);
  const data = JSON.parse(raw.toString()) as HostedCapture;
  literalCaptureLaw(data, title, artPath, phase);
  const physical = await wholeLiteralReports(build.root, title);
  assert.ok(physical.json && physical.html);
  const report = JSON.parse(physical.json.toString()) as VrtReport;
  assert.deepEqual(report.reportOwner, { version: 1, artIdentity: `src/${title}.art.vue` });
  await e.save(`${prefix}/physical-report.json`, physical.json);
  await e.save(`${prefix}/physical-report.html`, physical.html);
  const artifacts = await assertSessionReport({
    report: { data: report, json: physical.json, html: physical.html },
    packet: data,
    variantNames: literalVariants[title].map(([name]) => name),
    root: build.root,
    width: 320,
    height: 180,
    fetch: async (route, type) => {
      const bytes = await publicArtifact(session, new URL(gallery).origin, route, type);
      await e.save(`${prefix}/opaque-${route.split("/").at(-1)}.bin`, bytes);
      return bytes;
    },
  });
  const baselines: Buffer[] = [],
    images: unknown[] = [];
  for (const [index, result] of data.results.entries()) {
    for (const [kind, route] of Object.entries(result.images)) {
      assert.ok(kind === "snapshot" || kind === "current");
      const file =
        kind === "snapshot"
          ? report.results[index].snapshotPath
          : report.results[index].currentPath;
      assert.ok(file && path.isAbsolute(file) && file.startsWith(build.root + path.sep));
      const bytes = artifacts.get(route);
      assert.ok(bytes);
      assert.deepEqual(bytes, await readFile(file));
      await e.save(`${prefix}/${index}-${kind}.png`, bytes);
      const details = pngDetails(bytes);
      images.push({ index, variant: result.variantName, kind, bytes: bytes.length, ...details });
      if (kind === "snapshot") baselines[index] = bytes;
      if (phase === "match") {
        assert.ok(input.first?.[index]);
        assert.deepEqual(bytes, input.first[index]);
      }
    }
  }
  assert.equal(new Set(baselines.map(sha256)).size, literalVariants[title].length);
  await page.locator(".vrt-results img").first().waitFor();
  assert.equal(await page.locator(".vrt-results img").count(), literalVariants[title].length * 2);
  await page.waitForFunction(() =>
    [...document.querySelectorAll<HTMLImageElement>(".vrt-results img")].every(
      (image) =>
        image.complete &&
        image.naturalWidth === 320 &&
        image.naturalHeight === 180 &&
        image.src.startsWith("blob:"),
    ),
  );
  assert.deepEqual(
    await page.locator(".vrt-variant-name").allTextContents(),
    literalVariants[title].map(([name]) => name),
  );
  const layout = await page.locator(".vrt-results img").evaluateAll((elements) =>
    elements.map((element) => {
      const image = element as HTMLImageElement,
        box = image.getBoundingClientRect(),
        css = getComputedStyle(image);
      return {
        width: box.width,
        height: box.height,
        naturalWidth: image.naturalWidth,
        naturalHeight: image.naturalHeight,
        maxWidth: css.maxWidth,
        objectFit: css.objectFit,
        scope: [...image.attributes].filter((a) => a.name.startsWith("data-v-")).map((a) => a.name),
      };
    }),
  );
  assert.ok(
    layout.every(
      (v) =>
        v.width > 0 &&
        v.width <= 240 &&
        v.height > 0 &&
        v.height <= 135 &&
        v.maxWidth === "240px" &&
        v.scope.length > 0,
    ),
  );
  const reports: unknown[] = [];
  for (const kind of ["json", "html"] as const) {
    const bytes = artifacts.get(data.reports[kind]);
    assert.ok(bytes);
    assert.deepEqual(bytes, physical[kind]);
    await e.save(`${prefix}/authenticated-report.${kind}`, bytes);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page
        .getByRole("button", { name: `Download ${kind.toUpperCase()} report`, exact: true })
        .click(),
    ]);
    const directory = path.join(build.root, "trusted-downloads");
    await mkdir(directory, { recursive: true });
    const downloaded = path.join(directory, `${input.label}-${title}.${kind}`);
    await download.saveAs(downloaded);
    assert.deepEqual(await readFile(downloaded), bytes);
    await e.save(`${prefix}/downloaded-report.${kind}`, bytes);
    reports.push({ kind, bytes: bytes.length, sha256: sha256(bytes) });
  }
  session.assertRunning();
  return { baselines, receipt: { title, phase, data, images, reports, layout } };
}

export async function currentBlobURLs(page: Page) {
  return page
    .locator(".vrt-results img")
    .evaluateAll((elements) => elements.map((image) => (image as HTMLImageElement).src));
}
export async function assertBlobsRevoked(page: Page, urls: string[]) {
  assert.ok(urls.length > 0 && urls.every((url) => url.startsWith("blob:")));
  const readable = await page.evaluate(
    async (values) =>
      Promise.all(
        values.map(async (url) => {
          try {
            const r = await fetch(url);
            await r.arrayBuffer();
            return true;
          } catch {
            return false;
          }
        }),
      ),
    urls,
  );
  assert.ok(readable.every((value) => value === false));
}

/** Register before the native builder, including partial acquisition failure. */
export function literalProjectRetainer(project: string, e: Evidence) {
  return async () => {
    try {
      await stat(project);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") return;
      throw error;
    }
    await cp(project, path.join(e.output, "trusted-literal/physical-project"), {
      recursive: true,
      errorOnExist: true,
      force: false,
    });
    await e.json("trusted-literal/physical-project-inventory.json", await inventory(project));
  };
}

export async function persistedImagesAndReports(root: string) {
  try {
    return (await inventory(root)).filter(
      (v) => v.path.endsWith(".png") || v.path.startsWith("trusted-vrt-output/reports/"),
    );
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return [];
    throw error;
  }
}
