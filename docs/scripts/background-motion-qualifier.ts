import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { chromium } from "playwright";
import type { Page } from "playwright";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import { buildDocsBackgroundScript, createDocsBackgroundHtml } from "../theme/background.ts";

const root = path.resolve(import.meta.dirname, "../..");
const sourceFiles = [
  "docs/browser-path.js",
  "docs/theme/vein.js",
  "docs/theme/background.ts",
  "docs/theme/shaders/vein.vert",
  "docs/theme/shaders/vein.frag",
  "docs/vite.config.ts",
  "docs/scripts/background-motion-qualifier.ts",
  "tests/tooling/docs-background-motion-browser.test.ts",
];
const sha256 = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
type Frame = { time: number; scroll: number; dark: number; mouse: number[]; velocity: number };
type Observation = { frames: Frame[]; gpuDraws: number; width: number; height: number };
// Observe the genuine renderer and native draw methods without replacing a draw.
const observeRenderer = `
window.__backgroundMotion = { frames: [], gpuDraws: 0 };
for (const Context of [WebGLRenderingContext, WebGL2RenderingContext]) {
  for (const name of ['drawArrays', 'drawElements']) {
    if (!Object.hasOwn(Context.prototype, name)) continue;
    const draw = Context.prototype[name];
    Context.prototype[name] = function(...args) {
      const result = draw.apply(this, args);
      window.__backgroundMotion.gpuDraws++;
      return result;
    };
  }
}
const Renderer = THREE.WebGLRenderer;
THREE.WebGLRenderer = function(...args) {
  const renderer = new Renderer(...args);
  const render = renderer.render;
  renderer.render = function(scene, camera) {
    const result = render.call(this, scene, camera);
    const u = scene.children[0].material.uniforms;
    window.__backgroundMotion.frames.push({ time: u.u_time.value, scroll: u.u_scroll.value,
      dark: u.u_dark.value, mouse: [u.u_mouse.value.x, u.u_mouse.value.y],
      velocity: u.u_mouseVel.value });
    return result;
  };
  return renderer;
};`;

async function observation(page: Page): Promise<Observation> {
  return page.evaluate(() => {
    const packet = (
      window as unknown as { __backgroundMotion: Omit<Observation, "width" | "height"> }
    ).__backgroundMotion;
    const canvas = document.getElementById("vein-canvas");
    if (!(canvas instanceof HTMLCanvasElement)) throw new Error("Authored canvas is missing");
    return { ...packet, width: canvas.width, height: canvas.height };
  });
}
async function nativeFrames(page: Page, count: number) {
  await page.evaluate(async (count) => {
    for (let index = 0; index < count; index++)
      await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  }, count);
}
async function idle(page: Page) {
  await nativeFrames(page, 3);
  const before = await observation(page);
  await nativeFrames(page, 8);
  const after = await observation(page);
  assert.deepEqual(
    { rendererCalls: after.frames.length, gpuDraws: after.gpuDraws },
    { rendererCalls: before.frames.length, gpuDraws: before.gpuDraws },
    "reduced motion has no idle renderer or native GPU draw calls",
  );
  return after;
}
async function repaint(page: Page, before: Observation) {
  await page.waitForFunction(
    (count) =>
      (window as unknown as { __backgroundMotion: Observation }).__backgroundMotion.frames.length >
      count,
    before.frames.length,
  );
  const after = await idle(page);
  assert(after.gpuDraws > before.gpuDraws, "native event still draws the authored shader");
  assert.equal(
    after.frames.at(-1)?.time,
    before.frames.at(-1)?.time,
    "static repaint preserves time",
  );
  return after;
}

export async function qualifyBackgroundMotion(output: string) {
  await mkdir(output, { recursive: true });
  const source = await Promise.all(
    sourceFiles.map(async (file) => {
      const bytes = await readFile(path.join(root, file));
      const retained = path.join(output, "source", file);
      await mkdir(path.dirname(retained), { recursive: true });
      await writeFile(retained, bytes);
      return { file, bytes: bytes.length, sha256: sha256(bytes) };
    }),
  );
  const config = await readFile(path.join(root, "docs/vite.config.ts"), "utf8");
  const libraryUrl = config.match(
    /https:\/\/cdn\.jsdelivr\.net\/npm\/three@[^"\s<]+\/three\.min\.js/,
  )?.[0];
  assert(libraryUrl, "production Three.js URL is retained");
  const libraryResponse = await fetch(libraryUrl, { signal: AbortSignal.timeout(30_000) });
  assert.equal(libraryResponse.status, 200, "actual production library is available");
  const library = Buffer.from(await libraryResponse.arrayBuffer());
  const background = buildDocsBackgroundScript(path.join(root, "docs/theme"));
  const html = `<!doctype html><html data-theme="light"><head><meta charset="utf-8">
    <title>Authored background motion control</title><style>body{margin:0;min-height:12000px}</style>
    <script src="/three.min.js"></script><script>${observeRenderer}</script></head>
    <body class="entry-page"><header class="header"></header>${createDocsBackgroundHtml()}
    <main class="content"><h1>Background motion control</h1></main><script>${background}</script></body></html>`;
  const receipt = {
    sourceHead: execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
    sourceChanges: execFileSync("git", ["status", "--porcelain", "--", ...sourceFiles], {
      cwd: root,
      encoding: "utf8",
    }).trim(),
    workflow: { sha: process.env.GITHUB_SHA ?? null, run: process.env.GITHUB_RUN_ID ?? null },
    source,
    library: { url: libraryUrl, bytes: library.length, sha256: sha256(library) },
    scriptSha256: sha256(background),
    htmlSha256: sha256(html),
    browser: {
      executablePath: resolvePuppeteerExecutablePath() ?? chromium.executablePath(),
      version: null as string | null,
      platform: process.platform,
    },
    completed: false,
    failure: null as string | null,
    cases: [] as unknown[],
  };
  await Promise.all([
    writeFile(path.join(output, "production-three.min.js"), library),
    writeFile(path.join(output, "background.js"), background),
    writeFile(path.join(output, "fixture.html"), html),
  ]);
  const server = createServer((request, response) => {
    if (request.url === "/three.min.js") {
      response.writeHead(200, { "content-type": "text/javascript" });
      response.end(library);
    } else {
      response.writeHead(200, { "content-type": "text/html" });
      response.end(html);
    }
  });
  let browser: Awaited<ReturnType<typeof chromium.launch>> | undefined;
  try {
    await new Promise<void>((resolve, reject) => {
      server.once("error", reject);
      server.listen(0, "127.0.0.1", resolve);
    });
    const address = server.address();
    assert(address && typeof address !== "string");
    browser = await chromium.launch({
      executablePath: receipt.browser.executablePath,
      headless: true,
    });
    receipt.browser.version = browser.version();
    for (const [device, viewport] of [
      ["desktop", { width: 1440, height: 960 }],
      ["mobile", { width: 390, height: 844 }],
    ] as const) {
      const context = await browser.newContext({
        viewport,
        deviceScaleFactor: 1,
        reducedMotion: "reduce",
        colorScheme: "light",
      });
      const page = await context.newPage();
      page.setDefaultTimeout(10_000);
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(String(error)));
      try {
        const response = await page.goto(`http://127.0.0.1:${address.port}/`, {
          waitUntil: "networkidle",
        });
        assert.equal(response?.status(), 200);
        await page.waitForFunction(
          () =>
            (window as unknown as { __backgroundMotion: Observation }).__backgroundMotion.frames
              .length > 0,
        );
        const initial = await idle(page);
        assert(initial.gpuDraws > 0, "reduced motion keeps the actual WebGL background");
        assert.equal(initial.frames.at(-1)?.time, 0);
        await page.evaluate(() => scrollTo(0, 400));
        const scrolled = await repaint(page, initial);
        assert.equal(scrolled.frames.at(-1)?.scroll, 400);
        assert.equal(
          await page
            .locator(".header")
            .evaluate((element) => element.classList.contains("header-visible")),
          true,
        );
        const pointer = { x: Math.floor(viewport.width / 4), y: Math.floor(viewport.height / 4) };
        await page.mouse.move(pointer.x, pointer.y);
        const pointed = await repaint(page, scrolled);
        assert.deepEqual(pointed.frames.at(-1)?.mouse, [
          pointer.x / viewport.width,
          1 - pointer.y / viewport.height,
        ]);
        assert.equal(pointed.frames.at(-1)?.velocity, 0);
        const lightPng = await page.screenshot({
          path: path.join(output, `${device}-light.png`),
          animations: "disabled",
        });
        await page.evaluate(() => document.documentElement.setAttribute("data-theme", "dark"));
        const dark = await repaint(page, pointed);
        assert.equal(dark.frames.at(-1)?.dark, 1);
        const darkPng = await page.screenshot({
          path: path.join(output, `${device}-dark.png`),
          animations: "disabled",
        });
        assert.notEqual(
          sha256(darkPng),
          sha256(lightPng),
          "actual shader pixels respond to theme changes",
        );
        await page.evaluate(() => document.documentElement.removeAttribute("data-theme"));
        const fallbackLight = await repaint(page, dark);
        assert.equal(fallbackLight.frames.at(-1)?.dark, 0);
        await page.emulateMedia({ colorScheme: "dark" });
        const fallbackDark = await repaint(page, fallbackLight);
        assert.equal(fallbackDark.frames.at(-1)?.dark, 1);
        await page.evaluate(() => document.documentElement.setAttribute("data-theme", "light"));
        const explicitLight = await repaint(page, fallbackDark);
        assert.equal(explicitLight.frames.at(-1)?.dark, 0);
        await page.setViewportSize({ width: viewport.width, height: 8192 });
        const enlarged = await repaint(page, explicitLight);
        assert.equal(enlarged.width, viewport.width);
        assert.equal(enlarged.height, 8192);
        const file = `${device}-reduced-motion.png`;
        const png = await page.screenshot({
          path: path.join(output, file),
          animations: "disabled",
        });
        assert.equal(png.readUInt32BE(16), viewport.width);
        assert.equal(png.readUInt32BE(20), 8192);
        await page.setViewportSize(viewport);
        const restored = await repaint(page, enlarged);
        await page.emulateMedia({ reducedMotion: "no-preference" });
        await page.waitForFunction(
          (time) =>
            (window as unknown as { __backgroundMotion: Observation }).__backgroundMotion.frames.at(
              -1,
            )!.time > time,
          restored.frames.at(-1)!.time,
        );
        const animatedBefore = await observation(page);
        await nativeFrames(page, 4);
        const animated = await observation(page);
        assert(
          animated.frames.length >= animatedBefore.frames.length + 4,
          "normal motion keeps rendering every frame",
        );
        assert(animated.gpuDraws > animatedBefore.gpuDraws);
        assert(animated.frames.at(-1)!.time > animatedBefore.frames.at(-1)!.time);
        await page.emulateMedia({ reducedMotion: "reduce" });
        const stopped = await idle(page);
        assert(
          stopped.frames.at(-1)!.time > 0,
          "runtime reduction freezes the existing animation time",
        );
        await page.evaluate(() => scrollTo(0, 0));
        const final = await repaint(page, stopped);
        assert.equal(final.frames.at(-1)?.scroll, 0);
        assert.equal(
          await page
            .locator(".header")
            .evaluate((element) => element.classList.contains("header-visible")),
          false,
        );
        assert.deepEqual(errors, [], "complete page error stream");
        receipt.cases.push({
          device,
          viewport,
          initial,
          scrolled,
          pointed,
          dark,
          fallbackLight,
          fallbackDark,
          explicitLight,
          enlarged,
          restored,
          animatedBefore,
          animated,
          stopped,
          final,
          screenshot: { file, bytes: png.length, sha256: sha256(png) },
          themePixels: { lightSha256: sha256(lightPng), darkSha256: sha256(darkPng) },
          errors,
        });
      } finally {
        await writeFile(
          path.join(output, `${device}-operations.json`),
          JSON.stringify(
            {
              device,
              viewport,
              errors,
              observation: await observation(page).catch((error) => ({ failure: String(error) })),
            },
            null,
            2,
          ) + "\n",
        );
        await context.close();
      }
    }
    const finalSource = await Promise.all(
      sourceFiles.map(async (file) => {
        const bytes = await readFile(path.join(root, file));
        return { file, bytes: bytes.length, sha256: sha256(bytes) };
      }),
    );
    assert.deepEqual(finalSource, source, "source bytes stay fixed throughout qualification");
    if (receipt.workflow.sha !== null)
      assert.equal(
        receipt.workflow.sha,
        receipt.sourceHead,
        "workflow evidence belongs to this source checkout",
      );
    receipt.completed = true;
  } catch (error) {
    receipt.failure = error instanceof Error ? (error.stack ?? error.message) : String(error);
    throw error;
  } finally {
    await browser?.close();
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
    await writeFile(path.join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
  }
  return receipt;
}
