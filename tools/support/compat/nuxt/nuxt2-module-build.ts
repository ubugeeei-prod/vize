// Exercise the packed candidate module in a real Nuxt 2 webpack build and SSR server.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import type { ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { at, readJson } from "../../json-value.ts";
import { freePort, loadChromium } from "./host-types.ts";
import type { Browser, BrowserElement } from "./host-types.ts";
import { verifyNuxt2LintConfig } from "./nuxt2-lint-config.ts";
import { verifyNuxtAutoInitConfig } from "./nuxt-auto-init-config.ts";
import { verifyNuxt2Typecheck } from "./nuxt2-typecheck.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tools/support/compat/nuxt/fixtures/nuxt2-module-build");
const packageRoot = path.join(fixture, "node_modules/@vizejs/nuxt");
const candidateDist = path.join(root, "npm/framework/nuxt/dist");
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt2-build"));
fs.mkdirSync(artifacts, { recursive: true });
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const webpackEnvironment = {
  ...process.env,
  NO_COLOR: "1",
  // webpack 4 hashes require the legacy provider on the supported modern Node
  // host. Scope it to this pinned webpack fixture's child processes.
  NODE_OPTIONS: [process.env.NODE_OPTIONS, "--openssl-legacy-provider"].filter(Boolean).join(" "),
};
const timings: Array<{ stage: string; milliseconds: number }> = [];
const browserPackets: unknown[] = [];
let browser: Browser | undefined;

function packageIdentity(name: string) {
  const manifest = path.join(fixture, "node_modules", name, "package.json");
  const bytes = fs.readFileSync(manifest);
  return { name, version: at(JSON.parse(bytes.toString()), "version"), sha256: hash(bytes) };
}

async function verifyBrowser(port: number, mode: "production" | "development") {
  browser ??= await loadChromium(root).launch();
  const page = await browser.newPage();
  const pageErrors: string[] = [];
  const consoleMessages: Array<{ type: string; text: string }> = [];
  const packet: Record<string, unknown> = {
    mode,
    browser: browser.version(),
    pageErrors,
    consoleMessages,
    success: false,
  };
  page.on("pageerror", (error) => pageErrors.push(error.message));
  page.on("console", (message) =>
    consoleMessages.push({ type: message.type(), text: message.text() }),
  );
  const started = performance.now();
  const input = path.join(fixture, "pages/index.vue");
  const original = fs.readFileSync(input);
  try {
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "networkidle" });
    await page.getByRole("button", { name: "Clicks: 0" }).click();
    await page.getByRole("button", { name: "Clicks: 1" }).waitFor();
    const runtime = await page.evaluate(() => {
      const host = globalThis as unknown as {
        $nuxt: { $options: { _base: { version: string } } };
        document: { querySelector(selector: string): BrowserElement | null };
        getComputedStyle(element: BrowserElement): { color: string; fontWeight: string };
      };
      const card = host.document.querySelector('[data-test="runtime-card"]')!;
      const title = host.document.querySelector('[data-test="runtime-card"] strong')!;
      const asset = host.document.querySelector('[data-test="asset"]')!;
      return {
        vue: host.$nuxt.$options._base.version,
        title: title.textContent,
        attributes: card.getAttributeNames(),
        color: host.getComputedStyle(card).color,
        fontWeight: host.getComputedStyle(title).fontWeight,
        asset: asset.getAttribute("src"),
      };
    });
    packet.runtime = runtime;
    assert.equal(runtime.vue, "2.7.16");
    assert.equal(runtime.title, "Nuxt 2 alias");
    assert.equal(runtime.color, "rgb(102, 51, 153)");
    assert.equal(runtime.fontWeight, "700");
    assert.equal(runtime.attributes.filter((name) => name.startsWith("data-v-")).length > 0, true);
    assert.ok(runtime.asset);
    const asset = await fetch(new URL(runtime.asset, `http://127.0.0.1:${port}`));
    assert.equal(asset.status, 200);
    const assetBytes = new Uint8Array(await asset.arrayBuffer());
    assert.deepEqual(
      assetBytes,
      new Uint8Array(fs.readFileSync(path.join(fixture, "assets/runtime.svg"))),
    );
    fs.writeFileSync(path.join(artifacts, `${mode}-asset.svg`), assetBytes);
    fs.writeFileSync(path.join(artifacts, `${mode}-hydrated.html`), await page.content());
    if (mode === "development") {
      const changed = original.toString().replace("HMR original", "HMR updated");
      assert.notEqual(changed, original.toString());
      const hmrStarted = performance.now();
      fs.writeFileSync(input, changed);
      await page.waitForFunction(() => {
        const host = globalThis as unknown as {
          document: { querySelector(selector: string): { textContent: string } | null };
        };
        return host.document.querySelector('[data-test="hmr"]')?.textContent === "HMR updated";
      });
      await page.getByRole("button", { name: "Clicks: 1" }).waitFor();
      timings.push({
        stage: "development-template-hmr",
        milliseconds: performance.now() - hmrStarted,
      });
      fs.writeFileSync(path.join(artifacts, "development-hmr.html"), await page.content());
    }
    await page.getByRole("link", { name: "About" }).click();
    await page.getByRole("heading", { name: "Nuxt 2 route" }).waitFor();
    await page.getByRole("link", { name: "Home" }).click();
    await page.getByRole("heading", { name: "Nuxt 2 with Vize" }).waitFor();
    assert.deepEqual(pageErrors, []);
    assert.deepEqual(
      consoleMessages.filter(
        (message) => message.type === "error" || /hydration|mismatch/i.test(message.text),
      ),
      [],
    );
    Object.assign(packet, {
      success: true,
      hydratedClicks: 1,
      routes: ["/", "/about", "/"],
      assetSha256: hash(assetBytes),
    });
  } finally {
    fs.writeFileSync(input, original);
    await page.close();
    browserPackets.push(packet);
    timings.push({ stage: `${mode}-browser`, milliseconds: performance.now() - started });
    fs.writeFileSync(
      path.join(artifacts, "browser-packets.json"),
      JSON.stringify(browserPackets, null, 2) + "\n",
    );
  }
}

assert.equal(at(readJson(path.join(packageRoot, "package.json")), "version"), "0.429.1");
assert.equal(
  at(readJson(path.join(fixture, "node_modules/nuxt/package.json")), "version"),
  "2.17.3",
);
assert.ok(fs.existsSync(path.join(candidateDist, "nuxt2-entry.cjs")));

const swapped = (
  [
    ["nuxt", candidateDist],
    ["nuxt-lint-config", path.join(root, "npm/framework/nuxt-lint-config/dist")],
  ] as const
).map(([name, candidate]) => ({
  candidate,
  installed: path.join(fixture, "node_modules/@vizejs", name, "dist"),
  backup: path.join(artifacts, `published-${name}-dist`),
}));
for (const item of swapped) {
  assert.ok(fs.existsSync(item.candidate), "candidate package must be built");
  fs.cpSync(item.installed, item.backup, { recursive: true });
  fs.rmSync(item.installed, { recursive: true });
  fs.cpSync(item.candidate, item.installed, { recursive: true });
}

// Children started here; the outer cleanup stops any that are still running.
const servers: ChildProcess[] = [];
try {
  for (const name of [".nuxt", "dist"])
    fs.rmSync(path.join(fixture, name), { recursive: true, force: true });
  const buildLog = path.join(artifacts, "build.log");
  const log = fs.openSync(buildLog, "w");
  const buildStarted = performance.now();
  let result;
  try {
    result = spawnSync(process.execPath, ["node_modules/nuxt/bin/nuxt.js", "build"], {
      cwd: fixture,
      env: webpackEnvironment,
      stdio: ["ignore", log, log],
      timeout: 300_000,
    });
  } finally {
    fs.closeSync(log);
  }
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `Nuxt 2 build failed; see ${buildLog}`);
  timings.push({
    stage: "webpack-production-build",
    milliseconds: performance.now() - buildStarted,
  });

  const port = await freePort();

  const serverLog = fs.openSync(path.join(artifacts, "server.log"), "w");
  const child = spawn(process.execPath, ["node_modules/nuxt/bin/nuxt.js", "start"], {
    cwd: fixture,
    env: { ...webpackEnvironment, HOST: "127.0.0.1", PORT: String(port) },
    stdio: ["ignore", serverLog, serverLog],
  });
  servers.push(child);
  const serverExited = new Promise((resolve) => child.once("close", resolve));
  try {
    for (const [route, heading] of [
      ["/", "Nuxt 2 with Vize"],
      ["/about", "Nuxt 2 route"],
    ]) {
      let response: Response | undefined;
      for (let attempt = 0; attempt < 100; attempt++) {
        try {
          response = await fetch(`http://127.0.0.1:${port}${route}`, {
            signal: AbortSignal.timeout(1000),
          });
          break;
        } catch {
          if (child.exitCode !== null) break;
          await new Promise((resolve) => setTimeout(resolve, 100));
        }
      }
      assert.ok(response, `Nuxt 2 server did not answer ${route}`);
      assert.equal(response.status, 200, `${route} returned ${response.status}`);
      const html = await response.text();
      fs.writeFileSync(path.join(artifacts, `${route === "/" ? "index" : "about"}.html`), html);
      assert.match(html, new RegExp(`<h1[^>]*>${heading}</h1>`));
    }
    await verifyBrowser(port, "production");
  } finally {
    child.kill("SIGTERM");
    await serverExited;
    fs.closeSync(serverLog);
  }
  const devPort = await freePort();
  const devLog = fs.openSync(path.join(artifacts, "development.log"), "w");
  const dev = spawn(process.execPath, ["node_modules/nuxt/bin/nuxt.js", "dev"], {
    cwd: fixture,
    env: { ...webpackEnvironment, HOST: "127.0.0.1", PORT: String(devPort) },
    stdio: ["ignore", devLog, devLog],
  });
  servers.push(dev);
  const devExited = new Promise((resolve) => dev.once("close", resolve));
  try {
    const started = performance.now();
    let ready = false;
    for (let attempt = 0; attempt < 120; attempt++) {
      try {
        const response = await fetch(`http://127.0.0.1:${devPort}/`, {
          signal: AbortSignal.timeout(1000),
        });
        if (response.status === 200) {
          fs.writeFileSync(path.join(artifacts, "development-ssr.html"), await response.text());
          ready = true;
          break;
        }
      } catch {
        if (dev.exitCode !== null) break;
      }
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
    assert.equal(ready, true, "Nuxt 2 dev server must serve the actual application");
    timings.push({ stage: "development-startup", milliseconds: performance.now() - started });
    await verifyBrowser(devPort, "development");
  } finally {
    dev.kill("SIGTERM");
    await devExited;
    fs.closeSync(devLog);
  }
  verifyNuxt2LintConfig(fixture, artifacts);
  verifyNuxtAutoInitConfig(fixture, artifacts);
  verifyNuxt2Typecheck(fixture, artifacts);
  fs.writeFileSync(
    path.join(artifacts, "proof.json"),
    JSON.stringify(
      {
        candidateHead: spawnSync("git", ["rev-parse", "HEAD"], {
          cwd: root,
          encoding: "utf8",
        }).stdout.trim(),
        node: process.version,
        nuxt: "2.17.3",
        buildExit: result.status,
        routes: ["/", "/about"],
        compiler: "Nuxt host Vue 2 compiler; native Vue 3 compiler is not substituted",
        packages: [
          "nuxt",
          "vue",
          "vue-template-compiler",
          "vue-server-renderer",
          "vue-loader",
          "webpack",
          "sass",
          "sass-loader",
        ].map(packageIdentity),
        browserPackets,
        timings,
      },
      null,
      2,
    ) + "\n",
  );
  console.log("Candidate Nuxt 2 webpack build and SSR routes passed");
} finally {
  fs.writeFileSync(path.join(artifacts, "timings.json"), JSON.stringify(timings, null, 2) + "\n");
  await browser?.close();
  for (const server of servers) if (server.exitCode === null) server.kill("SIGTERM");
  for (const item of swapped) {
    fs.rmSync(item.installed, { recursive: true, force: true });
    fs.cpSync(item.backup, item.installed, { recursive: true });
  }
}
