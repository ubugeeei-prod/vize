import assert from "node:assert/strict";
import { readFile, realpath } from "node:fs/promises";
import path from "node:path";
import type { Page, Response } from "playwright";
import type { Evidence } from "./vrt_artifacts.ts";
import { sha256 } from "./vrt_fixtures.ts";
import { literalVariants } from "./vrt_literal.ts";
import type { LiteralBuild } from "./vrt_trusted_ui.ts";

/** Current emitted files only; no source, edge injection, unknown URL or redirect fallback. */
export async function physicalPublicBody(
  build: Pick<LiteralBuild, "directory">,
  value: string,
  origin: string,
) {
  const url = new URL(value);
  assert.equal(url.origin, origin);
  assert.equal(url.username + url.password, "");
  assert.equal(url.search + url.hash, "");
  assert.ok(url.pathname.startsWith("/built/"));
  const root = await realpath(build.directory);
  let relative = decodeURIComponent(url.pathname.slice("/built/".length));
  if (relative.endsWith("/")) relative += "index.html";
  let file = path.resolve(root, relative);
  assert.ok(file.startsWith(root + path.sep));
  try {
    file = await realpath(file);
  } catch (error) {
    if (
      (error as NodeJS.ErrnoException).code !== "ENOENT" ||
      path.extname(url.pathname) ||
      !url.pathname.startsWith("/built/gallery/vrt/")
    )
      throw error;
    file = await realpath(path.join(root, "gallery/vrt/index.html"));
  }
  assert.ok(file.startsWith(root + path.sep));
  return { file, bytes: await readFile(file) };
}

/** All actual observer public response bodies, including rebuilt assets, join whole native bytes. */
export function publicBodyCustody(
  page: Page,
  build: LiteralBuild,
  gallery: string,
  e: Evidence,
  phase: () => string = () => "",
) {
  const origin = new URL(gallery).origin;
  const allowed = new Set([origin]);
  const records: unknown[] = [],
    failures: unknown[] = [];
  const pending = new Set<Promise<void>>(),
    saved = new Set<string>();
  const save = async (bytes: Buffer) => {
    const hash = sha256(bytes);
    if (!saved.has(hash)) {
      saved.add(hash);
      await e.save(`trusted-public/http-bodies/${hash}.bin`, bytes);
    }
    return hash;
  };
  const join = async (
    value: string,
    status: number,
    bytes: Buffer,
    method: string,
    observedPhase = phase(),
  ) => {
    const digest = await save(bytes);
    const retained: {
      phase: string;
      url: string;
      status: number;
      method: string;
      bytes: number;
      sha256: string;
      file?: string;
      physicalSha256?: string;
    } = {
      phase: observedPhase,
      url: value,
      status,
      method,
      bytes: bytes.length,
      sha256: digest,
    };
    records.push(retained);
    const physical = await physicalPublicBody(build, value, origin);
    retained.file = physical.file;
    retained.physicalSha256 = sha256(physical.bytes);
    assert.equal(status, 200);
    assert.deepEqual(bytes, physical.bytes);
    return bytes;
  };
  const observe = (response: Response) => {
    const value = response.url(),
      url = new URL(value);
    if (!/^https?:$/.test(url.protocol)) return;
    const observedPhase = phase();
    const metadata = {
      phase: observedPhase,
      url: `${url.origin}${url.pathname}`,
      status: response.status(),
    };
    records.push(metadata);
    const work = (async () => {
      assert.equal(url.search + url.hash + url.username + url.password, "");
      assert.ok(allowed.has(url.origin), "Unowned public browser response origin");
      if (url.origin !== origin) return;
      assert.equal(response.request().redirectedFrom(), null);
      await join(
        value,
        response.status(),
        await response.body(),
        response.request().method(),
        observedPhase,
      );
    })();
    pending.add(work);
    void work.then(
      () => pending.delete(work),
      (error) => {
        failures.push(error);
        pending.delete(work);
      },
    );
  };
  const settle = async () => {
    while (pending.size) await Promise.allSettled(pending);
    if (failures.length) throw new AggregateError(failures, "Public response custody refused");
  };
  page.on("response", observe);
  return {
    records,
    settle,
    allowLoopback(endpoint: string) {
      const url = new URL(endpoint);
      assert.equal(url.protocol, "http:");
      assert.equal(url.hostname, "127.0.0.1");
      assert.equal(url.origin, endpoint);
      allowed.add(endpoint);
    },
    async fetch(value: string | URL) {
      const url = String(value);
      await physicalPublicBody(build, url, origin);
      const response = await fetch(url, { redirect: "error", signal: AbortSignal.timeout(30000) });
      return join(url, response.status, Buffer.from(await response.arrayBuffer()), "GET");
    },
    async close() {
      page.off("response", observe);
      await settle();
    },
  };
}
export type PublicBodyCustody = ReturnType<typeof publicBodyCustody>;

/** The v1 five-literal public native preview/global assertions remain intact in this split. */
export async function observePublicLiteralPreviews(input: {
  page: Page;
  build: LiteralBuild;
  gallery: string;
  bodies: PublicBodyCustody;
  records: unknown[];
  assertRunning: () => void;
}) {
  const { page, build, gallery: galleryUrl, bodies, records, assertRunning } = input;
  const origin = new URL(galleryUrl).origin,
    phase = "secure-public-previews";
  assert.deepEqual(await bodies.fetch(new URL("api/static.json", galleryUrl)), build.raw);
  const previewURLs = new Set<string>();
  for (const title of ["Controls", "Keys"] as const) {
    const art = path.join(build.root, "src", `${title}.art.vue`);
    for (const [name, button] of literalVariants[title]) {
      const url = new URL(build.manifest.previews[art][name], galleryUrl);
      assert.equal(url.origin, origin);
      assert.equal(url.search + url.hash, "");
      assert.equal(previewURLs.has(url.href), false);
      previewURLs.add(url.href);
      const physical = await readFile(
        path.join(build.directory, url.pathname.slice("/built/".length)),
      );
      assert.deepEqual(await bodies.fetch(url), physical);
      await page.goto(url.href);
      await page.getByRole("button", { name: button, exact: true }).waitFor();
      assert.equal(await page.locator("museacomponent").count(), 0);
      records.push({
        phase,
        title,
        name,
        ownPreviewURL: url.href,
        physicalSha256: sha256(physical),
      });
      assertRunning();
    }
  }
  assert.equal(previewURLs.size, 5);
  await page.goto(galleryUrl);
  assert.equal(await page.evaluate(() => window.isSecureContext), true);
  const globals = await page.evaluate(() => {
    const previews = Reflect.get(window, "__MUSEA_STATIC_PREVIEWS__") as Record<
      string,
      Record<string, string>
    >;
    return Object.keys(previews).map((art) => ({
      art,
      keys: Object.keys(previews[art]),
      own: Object.keys(previews[art]).every(
        (key) => Object.hasOwn(previews[art], key) && typeof previews[art][key] === "string",
      ),
      prototype: Object.getPrototypeOf(previews[art]) === Object.prototype,
      inheritedToString: !Object.hasOwn(previews[art], "toString"),
    }));
  });
  for (const title of ["Controls", "Keys"] as const) {
    const actual = globals.find((v) => v.art === path.join(build.root, "src", `${title}.art.vue`));
    assert.ok(actual?.own && actual.prototype && actual.inheritedToString);
    assert.deepEqual(
      actual.keys,
      literalVariants[title].map(([name]) => name),
    );
  }
  await bodies.settle();
  return globals;
}
