import assert from "node:assert/strict";
import { lstat, mkdir, readFile, readdir, realpath } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright";
import {
  assertApiReport,
  assertSummary,
  pngDetails,
  type ApiCapture,
  type Evidence,
  type VrtReport,
} from "./vrt_artifacts.ts";
import { sha256, type Side } from "./vrt_fixtures.ts";
import type { TrustedColorBuild } from "./vrt_trusted_color_build.ts";
import { connectPublicSession, publicArtifact, type PublicSession } from "./vrt_trusted_ui.ts";
import type { PublicBodyCustody } from "./vrt_trusted_gallery.ts";

export type ColorPhase = "new" | "match" | "diff" | "update" | "updated-match";
interface ColorPacket extends Pick<ApiCapture, "success" | "summary"> {
  results: Array<ApiCapture["results"][number] & { images: Record<string, string> }>;
  reports: { json: string; html: string };
}

/** Distinguish absent/empty attributes and about:blank without retaining URL credentials. */
export function colorPreviewAddress(value: string | null, base: string) {
  if (value === null || value === "") return value;
  try {
    const url = new URL(value, base),
      protocol = url.protocol;
    if (!["http:", "https:", "about:"].includes(protocol)) return { protocol, opaque: true };
    const query = !!url.search,
      fragment = !!url.hash,
      credentials = !!(url.username || url.password);
    url.username = url.password = url.search = url.hash = "";
    return { value: url.href, protocol, query, fragment, credentials };
  } catch {
    return { invalid: true };
  }
}

/** Preserve the original refusal even when inspecting or retaining the live page also fails. */
export async function retainColorPreviewFailure(
  error: unknown,
  inspect: () => Promise<unknown>,
  e: Pick<Evidence, "json">,
  prefix: string,
): Promise<never> {
  let observation: unknown;
  try {
    observation = { page: await inspect() };
  } catch (inspectionError) {
    observation = { inspectionError: String(inspectionError) };
  }
  try {
    await e.json(`${prefix}/preview-refusal.json`, {
      error:
        error instanceof Error
          ? { name: error.name, message: error.message, stack: error.stack }
          : { kind: typeof error },
      observation,
    });
  } catch (retentionError) {
    throw new AggregateError(
      [error, retentionError],
      "Color preview and failure retention refused",
    );
  }
  throw error;
}
export function colorCaptureLaw(data: ColorPacket, art: string, side: Side, phase: ColorPhase) {
  const failed = side === "left" && (phase === "diff" || phase === "update");
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
    [1, phase === "new" ? 1 : 0, phase !== "new" && !failed ? 1 : 0, failed ? 1 : 0, 0, 0],
  );
  assert.ok(Number.isFinite(data.summary.duration) && data.summary.duration >= 0);
  assert.equal(data.results.length, 1);
  const result = data.results[0];
  assert.equal(result.variantName, "Default");
  assert.equal(result.viewport, "authored-compact");
  assert.equal(result.error, undefined);
  assert.equal(result.passed, !failed);
  if (phase === "new") {
    assert.equal(result.isNew, true);
    assert.equal(result.diffPercentage, undefined);
  } else assert.equal(Object.hasOwn(result, "isNew"), false);
  assert.equal(data.results[0].artPath, art);
  assert.deepEqual(
    Object.keys(data.results[0].images),
    failed ? ["snapshot", "current", "diff"] : ["snapshot", "current"],
  );
  if (phase !== "new") {
    const difference = data.results[0].diffPercentage;
    if (failed) assert.ok(typeof difference === "number" && difference > 0);
    else assert.equal(difference, 0);
  }
}

/** Discover the actual physical report by its portable owner, never infer a basename collision. */
export async function colorReports(build: TrustedColorBuild, side: Side) {
  const directory = path.join(build.literal.root, "trusted-vrt-output/reports");
  let names: string[];
  try {
    names = await readdir(directory);
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") return undefined;
    throw error;
  }
  const identity = `src/${side}/Button.art.vue`,
    owned: Array<{ file: string; json: Buffer; data: VrtReport }> = [];
  for (const name of names.filter((name) => name.endsWith("-report.json"))) {
    const file = path.join(directory, name),
      info = await lstat(file);
    assert.ok(info.isFile() && !info.isSymbolicLink());
    const json = await readFile(file),
      data = JSON.parse(json.toString()) as VrtReport;
    if (data.reportOwner?.artIdentity === identity) owned.push({ file, json, data });
  }
  assert.ok(owned.length <= 1);
  if (!owned[0]) return undefined;
  assert.deepEqual(owned[0].data.reportOwner, { version: 1, artIdentity: identity });
  const htmlFile = owned[0].file.replace(/\.json$/, ".html");
  const info = await lstat(htmlFile);
  assert.ok(info.isFile() && !info.isSymbolicLink());
  return { ...owned[0], htmlFile, html: await readFile(htmlFile) };
}

/** All other-Art image paths belong to its exact physical report owner and regular root files. */
export async function otherColorPNGs(root: string, art: string, report: VrtReport) {
  assert.ok(path.isAbsolute(root) && path.resolve(root) === root);
  assert.ok(path.isAbsolute(art) && path.resolve(art) === art && art.startsWith(root + path.sep));
  assert.deepEqual(report.reportOwner, {
    version: 1,
    artIdentity: path.relative(root, art).split(path.sep).join("/"),
  });
  assert.equal(report.results.length, 1);
  const canonicalRoot = await realpath(root),
    images: Array<{ field: string; file: string; bytes: Buffer }> = [];
  for (const result of report.results) {
    assert.equal(result.artPath, art);
    for (const field of ["snapshotPath", "currentPath", "diffPath"] as const) {
      const file = result[field];
      if (field === "diffPath" && file === undefined) continue;
      assert.ok(file && path.isAbsolute(file) && path.resolve(file) === file);
      assert.ok(file.startsWith(root + path.sep) && path.extname(file) === ".png");
      const info = await lstat(file);
      assert.ok(info.isFile() && !info.isSymbolicLink());
      assert.equal(await realpath(file), path.join(canonicalRoot, path.relative(root, file)));
      images.push({ field, file, bytes: await readFile(file) });
    }
  }
  return images;
}

/** Every phase is a genuine UI POST and authenticated whole PNG/report/download join. */
export async function capturePublicColor(input: {
  page: Page;
  session: PublicSession;
  build: TrustedColorBuild;
  gallery: string;
  bodies: PublicBodyCustody;
  side: Side;
  phase: ColorPhase;
  e: Evidence;
  first?: Buffer;
  updated?: Buffer;
}) {
  const { page, session, build, gallery, bodies, side, phase, e } = input;
  const title = side === "left" ? "Left" : "Right",
    art = build.arts[side];
  const prefix = `trusted-public/colors/${phase}/${side}`,
    update = phase === "update";
  const other = side === "left" ? "right" : "left";
  const prior = await colorReports(build, other);
  const otherPNGs = prior
    ? await otherColorPNGs(build.literal.root, build.arts[other], prior.data)
    : [];
  for (const [index, image] of otherPNGs.entries())
    await e.save(`${prefix}/other-before-${index}-${image.field}.png`, image.bytes);
  session.assertRunning();
  await page.getByRole("button", { name: "Variants", exact: true }).click();
  await page.locator(".art-item").filter({ hasText: title }).click();
  const preview = new URL(build.manifest.previews[art].Default, gallery);
  const physical = await bodies.fetch(preview);
  const button = page
    .frameLocator(".variant-card iframe")
    .getByRole("button", { name: title, exact: true });
  try {
    await button.waitFor({ timeout: 15000 });
  } catch (error) {
    await retainColorPreviewFailure(
      error,
      async () => {
        // No input values, query, fragment, credentials, request headers or session data.
        const raw = await page.evaluate(() => ({
          location: location.href,
          heading: document.querySelector(".component-title")?.textContent,
          activeArts: [...document.querySelectorAll(".art-item.active")].map((art) => ({
            tag: art.tagName,
            text: art.textContent,
            href: art.getAttribute("href"),
          })),
          tabs: [...document.querySelectorAll(".component-tabs button")].map((tab) => ({
            text: tab.textContent,
            pressed: tab.getAttribute("aria-pressed"),
          })),
          scroll: { x: scrollX, y: scrollY, width: innerWidth, height: innerHeight },
          frames: [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map(
            (frame) => {
              const rect = frame.getBoundingClientRect(),
                css = getComputedStyle(frame);
              const content = (() => {
                try {
                  const doc = frame.contentDocument;
                  return doc
                    ? {
                        location: doc.URL,
                        readyState: doc.readyState,
                        buttons: [...doc.querySelectorAll("button")].map(
                          (item) => item.textContent,
                        ),
                        message: doc.querySelector(".musea-loading, .musea-error")?.textContent,
                      }
                    : { accessible: false, reason: "contentDocument is null" };
                } catch (accessError) {
                  return { accessible: false, error: String(accessError) };
                }
              })();
              return {
                tag: frame.tagName,
                src: frame.getAttribute("src"),
                currentSrc: frame.src,
                loading: frame.loading,
                connected: frame.isConnected,
                rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
                css: { display: css.display, visibility: css.visibility, opacity: css.opacity },
                intersectsViewport:
                  rect.bottom > 0 &&
                  rect.right > 0 &&
                  rect.top < innerHeight &&
                  rect.left < innerWidth,
                content,
              };
            },
          ),
        }));
        const observation = {
          ...raw,
          location: colorPreviewAddress(raw.location, raw.location),
          activeArts: raw.activeArts.map((art) => ({
            ...art,
            href: colorPreviewAddress(art.href, raw.location),
          })),
          frames: raw.frames.map((frame) => ({
            ...frame,
            src: colorPreviewAddress(frame.src, raw.location),
            currentSrc: colorPreviewAddress(frame.currentSrc, raw.location),
            content:
              "location" in frame.content && typeof frame.content.location === "string"
                ? {
                    ...frame.content,
                    location: colorPreviewAddress(frame.content.location, raw.location),
                  }
                : frame.content,
          })),
        };
        if ((await page.getByLabel("Session token", { exact: true }).count()) === 0) {
          // The original connected-session law already removes the credential input.
          try {
            const screenshot = await page.screenshot({ timeout: 5000 });
            await e.save(`${prefix}/preview-refusal.png`, screenshot);
          } catch (screenshotError) {
            return { ...observation, screenshotError: String(screenshotError) };
          }
        }
        return observation;
      },
      e,
      prefix,
    );
  }
  const actualPreview = await button.evaluate((element) => ({
    color: getComputedStyle(element).backgroundColor,
    url: element.ownerDocument.URL,
    unresolved: element.ownerDocument.querySelectorAll("museacomponent").length,
  }));
  assert.equal(actualPreview.url, preview.href);
  assert.equal(actualPreview.unresolved, 0);
  assert.equal(
    actualPreview.color,
    side === "right"
      ? "rgb(255, 0, 0)"
      : phase === "new" || phase === "match"
        ? "rgb(0, 0, 255)"
        : "rgb(0, 255, 0)",
  );
  await bodies.settle();
  await page.getByRole("button", { name: "VRT", exact: true }).click();
  await connectPublicSession(page, session);
  await page.getByLabel("Update snapshots", { exact: true }).setChecked(update);
  const [response] = await Promise.all([
    page.waitForResponse(
      (response) =>
        response.url() === `${session.endpoint}/capture` && response.request().method() === "POST",
    ),
    page.getByRole("button", { name: "Run VRT", exact: true }).click(),
  ]);
  assert.deepEqual(response.request().postDataJSON(), { artPath: art, update });
  const raw = await response.body();
  await e.save(`${prefix}/response.json`, raw);
  assert.equal(response.status(), 200);
  const data = JSON.parse(raw.toString()) as ColorPacket;
  colorCaptureLaw(data, art, side, phase);
  const written = await colorReports(build, side);
  assert.ok(written);
  assertApiReport(written.data, data, 320, 180);
  assertSummary(written.data, 1, {
    new: data.summary.new,
    passed: data.summary.passed,
    failed: data.summary.failed,
  });
  const result = data.results[0],
    images: unknown[] = [];
  assert.equal(
    path.dirname(result.snapshotPath),
    path.join(build.literal.root, "trusted-baselines"),
  );
  let baseline: Buffer | undefined;
  for (const [kind, route] of Object.entries(result.images)) {
    const file =
      kind === "snapshot"
        ? result.snapshotPath
        : kind === "current"
          ? result.currentPath
          : result.diffPath;
    assert.ok(file && path.isAbsolute(file) && file.startsWith(build.literal.root + path.sep));
    const bytes = await publicArtifact(session, new URL(gallery).origin, route, "image/png");
    assert.deepEqual(bytes, await readFile(file));
    await e.save(`${prefix}/${kind}.png`, bytes);
    const details = pngDetails(bytes);
    images.push({ kind, file, bytes: bytes.length, ...details });
    if (kind === "snapshot") baseline = bytes;
    if (kind === "current" && (phase === "new" || phase === "update")) {
      assert.ok(baseline);
      assert.deepEqual(bytes, baseline);
    }
    if (kind !== "diff") {
      const color =
        side === "right"
          ? "red"
          : phase === "new" || phase === "match" || (kind === "snapshot" && phase === "diff")
            ? "blue"
            : "green";
      assert.ok(details.colors[color] > 2000, `${phase}/${side}/${kind}/${color}`);
      if (phase === "match" || side === "right") {
        if (input.first) assert.deepEqual(bytes, input.first);
      } else if (phase === "updated-match") {
        assert.ok(input.updated);
        assert.deepEqual(bytes, input.updated);
      } else if (phase === "diff" && kind === "snapshot") {
        assert.ok(input.first);
        assert.deepEqual(bytes, input.first);
      }
    }
  }
  assert.ok(baseline);
  if (side === "left" && (phase === "update" || phase === "updated-match")) {
    assert.ok(input.first);
    assert.notDeepEqual(baseline, input.first);
  }
  const count = Object.keys(result.images).length;
  await page.locator(".vrt-results img").first().waitFor();
  assert.equal(await page.locator(".vrt-results img").count(), count);
  await page.waitForFunction(() =>
    [...document.querySelectorAll<HTMLImageElement>(".vrt-results img")].every(
      (image) =>
        image.complete &&
        image.naturalWidth === 320 &&
        image.naturalHeight === 180 &&
        image.src.startsWith("blob:"),
    ),
  );
  assert.deepEqual(await page.locator(".vrt-variant-name").allTextContents(), ["Default"]);
  const layout = await page.locator(".vrt-results img").evaluateAll((elements) =>
    elements.map((element) => {
      const image = element as HTMLImageElement,
        box = image.getBoundingClientRect(),
        css = getComputedStyle(image);
      return {
        width: box.width,
        height: box.height,
        maxWidth: css.maxWidth,
        scope: [...image.attributes].filter((a) => a.name.startsWith("data-v-")).map((a) => a.name),
      };
    }),
  );
  assert.equal(layout.length, count);
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

  const reports: Array<{ kind: "json" | "html"; file: string; bytes: number; sha256: string }> = [];
  for (const kind of ["json", "html"] as const) {
    const bytes = await publicArtifact(
      session,
      new URL(gallery).origin,
      data.reports[kind],
      kind === "json" ? "application/json" : "text/html",
    );
    assert.deepEqual(bytes, written[kind]);
    await e.save(`${prefix}/authenticated-report.${kind}`, bytes);
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      page
        .getByRole("button", { name: `Download ${kind.toUpperCase()} report`, exact: true })
        .click(),
    ]);
    const directory = path.join(build.literal.root, "trusted-downloads");
    await mkdir(directory, { recursive: true });
    const file = path.join(directory, `${phase}-${side}.${kind}`);
    await download.saveAs(file);
    assert.deepEqual(await readFile(file), bytes);
    await e.save(`${prefix}/downloaded-report.${kind}`, bytes);
    reports.push({
      kind,
      file: kind === "json" ? written.file : written.htmlFile,
      bytes: bytes.length,
      sha256: sha256(bytes),
    });
  }
  const after = await colorReports(build, other);
  assert.deepEqual(after, prior);
  const afterPNGs = after
    ? await otherColorPNGs(build.literal.root, build.arts[other], after.data)
    : [];
  for (const [index, image] of afterPNGs.entries())
    await e.save(`${prefix}/other-after-${index}-${image.field}.png`, image.bytes);
  assert.deepEqual(afterPNGs, otherPNGs);
  await bodies.settle();
  session.assertRunning();
  return {
    baseline,
    receipt: {
      phase,
      side,
      httpStatus: response.status(),
      update,
      data,
      images,
      reports,
      preview: { ...actualPreview, sha256: sha256(physical) },
      otherWholePNGAndReportsUnchanged: other,
      layout,
      snapshotPath: result.snapshotPath,
      reportOwner: written.data.reportOwner,
    },
  };
}
