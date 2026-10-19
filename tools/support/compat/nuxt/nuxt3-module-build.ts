// Exercise the packed candidate module and Vite adapter in a real Nuxt 3 build.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import type { ChildProcess } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { at, readJson } from "../../json-value.ts";
import { freePort, loadChromium } from "./host-types.ts";
import type { Browser } from "./host-types.ts";
import { prepareNuxtSourceBinding } from "./source-binding.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tools/support/compat/nuxt/fixtures/nuxt3-module-build");
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt3-build"));
fs.mkdirSync(artifacts, { recursive: true });
const sourceBinding = prepareNuxtSourceBinding(root, artifacts);

for (const [name, version] of [
  ["nuxt", "3.19.3"],
  ["vue", "3.5.43"],
  ["@vizejs/nuxt", "0.429.1"],
  ["@vizejs/vite-plugin", "0.429.1"],
] as const) {
  const manifest = readJson(path.join(fixture, "node_modules", name, "package.json"));
  assert.equal(at(manifest, "version"), version, `fixture must use the exact ${name} version`);
}

const swapped = (
  [
    ["nuxt", path.join(root, "npm/framework/nuxt/dist")],
    ["vite-plugin", path.join(root, "npm/builder/vite/dist")],
  ] as const
).map(([name, candidate]) => ({
  name,
  candidate,
  installed: path.join(fixture, "node_modules/@vizejs", name, "dist"),
  backup: path.join(artifacts, `published-${name}-dist`),
}));
for (const item of swapped) {
  assert.ok(fs.existsSync(item.candidate), `candidate ${item.name} must be built`);
  fs.cpSync(item.installed, item.backup, { recursive: true });
  fs.rmSync(item.installed, { recursive: true });
  fs.cpSync(item.candidate, item.installed, { recursive: true });
}

// Children started here; the outer cleanup stops any that are still running.
const servers: ChildProcess[] = [];
try {
  for (const name of [".nuxt", ".output"])
    fs.rmSync(path.join(fixture, name), { recursive: true, force: true });
  // Nuxt replaces module-import exceptions with a generic installation error.
  // Keep the original exception under the exact same source-binding environment.
  const importLog = path.join(artifacts, "module-import.log");
  const importFd = fs.openSync(importLog, "w");
  let imported: ReturnType<typeof spawnSync>;
  try {
    imported = spawnSync(
      process.execPath,
      ["--input-type=module", "--eval", 'await import("@vizejs/nuxt")'],
      {
        cwd: fixture,
        env: { ...sourceBinding.environment, NO_COLOR: "1" },
        stdio: ["ignore", importFd, importFd],
        timeout: 30_000,
      },
    );
  } finally {
    fs.closeSync(importFd);
  }
  assert.equal(imported.error, undefined);
  assert.equal(imported.status, 0, `Source-bound Nuxt module import failed; see ${importLog}`);
  const buildLog = path.join(artifacts, "build.log");
  const buildFd = fs.openSync(buildLog, "w");
  let build: ReturnType<typeof spawnSync>;
  try {
    build = spawnSync(process.execPath, ["node_modules/nuxt/bin/nuxt.mjs", "build"], {
      cwd: fixture,
      env: { ...sourceBinding.environment, NO_COLOR: "1" },
      stdio: ["ignore", buildFd, buildFd],
      timeout: 300_000,
    });
  } finally {
    fs.closeSync(buildFd);
  }
  assert.equal(build.error, undefined);
  assert.equal(build.status, 0, `Nuxt 3 build failed; see ${buildLog}`);

  const port = await freePort();

  const serverFd = fs.openSync(path.join(artifacts, "server.log"), "w");
  const child = spawn(process.execPath, [path.join(fixture, ".output/server/index.mjs")], {
    cwd: fixture,
    env: { ...process.env, HOST: "127.0.0.1", PORT: String(port), NO_COLOR: "1" },
    stdio: ["ignore", serverFd, serverFd],
  });
  servers.push(child);
  const serverExited = new Promise((resolve) => child.once("exit", resolve));
  let browser: Browser | undefined;
  try {
    for (const [route, heading] of [
      ["/", "Nuxt 3 with Vize"],
      ["/about", "Nuxt 3 route"],
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
      assert.ok(response, `Nuxt 3 server did not answer ${route}`);
      assert.equal(response.status, 200, `${route} returned ${response.status}`);
      const html = await response.text();
      fs.writeFileSync(path.join(artifacts, `${route === "/" ? "index" : "about"}.html`), html);
      assert.match(html, new RegExp(`<h1[^>]*>${heading}</h1>`));
      if (route === "/") {
        assert.match(html, /<article[^>]*data-card="Alpha"[^>]*>/);
        assert.match(html, /<article[^>]*data-card="Beta"[^>]*>/);
        assert.match(html, /<strong[^>]*>Alpha<\/strong>/);
        assert.match(html, /<strong[^>]*>Beta<\/strong>/);
        assert.match(html, /<span[^>]*class="card-id"[^>]*>alpha<\/span>/);
        assert.match(html, /<span[^>]*class="card-id"[^>]*>beta<\/span>/);
        assert.equal((html.match(/class="featured"/g) ?? []).length, 1);
        assert.match(html, /<span[^>]*class="featured"[^>]*>Featured<\/span>/);
        assert.match(html, /<button[^>]*data-test="counter"[^>]*>Clicks: 0<\/button>/);
        const cssFiles = fs
          .readdirSync(path.join(fixture, ".output/public/_nuxt"))
          .filter((name) => name.endsWith(".css"));
        assert.ok(
          cssFiles.some((name) =>
            /rebeccapurple|#639(?:\b|;|})|#663399\b|rgb\(102,\s*51,\s*153\)/i.test(
              fs.readFileSync(path.join(fixture, ".output/public/_nuxt", name), "utf8"),
            ),
          ),
          "scoped component CSS must reach the Nuxt build",
        );
      }
    }
    for (const [route, marker] of [
      ["/page-meta", "page-meta-explicit"],
      ["/page-meta-global", "page-meta-global"],
    ]) {
      const response = await fetch(`http://127.0.0.1:${port}${route}`, {
        signal: AbortSignal.timeout(5000),
      });
      assert.equal(response.status, 200);
      const html = await response.text();
      fs.writeFileSync(path.join(artifacts, `${marker}.html`), html);
      assert.match(
        html,
        new RegExp(
          `<main[^>]*id="bare-layout"[^>]*><!--\\[--><p[^>]*id="${marker}"[^>]*>page</p><!--\\]--></main>`,
        ),
      );
      assert.ok(
        !html.includes("DEFAULT LAYOUT"),
        "extracted page metadata must select its original layout",
      );
    }
    browser = await loadChromium(root).launch();
    const page = await browser.newPage();
    const pageErrors: string[] = [];
    page.on("pageerror", (error) => pageErrors.push(error.message));
    await page.goto(`http://127.0.0.1:${port}/`, { waitUntil: "domcontentloaded" });
    await page.getByRole("heading", { name: "Nuxt 3 with Vize" }).waitFor();
    await page.getByRole("button", { name: "Clicks: 0" }).click();
    await page.getByRole("button", { name: "Clicks: 1" }).waitFor();
    await page.getByRole("link", { name: "About" }).click();
    await page.getByRole("heading", { name: "Nuxt 3 route" }).waitFor();
    for (const marker of ["page-meta-explicit", "page-meta-global"]) {
      await page.goto(
        `http://127.0.0.1:${port}/${marker === "page-meta-explicit" ? "page-meta" : "page-meta-global"}`,
        { waitUntil: "networkidle" },
      );
      await page.locator(`#bare-layout #${marker}`).waitFor();
      assert.equal(await page.locator("#default-header").count(), 0);
    }
    await page.goto(`http://127.0.0.1:${port}/about`, { waitUntil: "domcontentloaded" });
    await page.getByRole("heading", { name: "Nuxt 3 route" }).waitFor();
    assert.deepEqual(pageErrors, [], "hydration and client navigation must not throw");
    fs.writeFileSync(
      path.join(artifacts, "hydration.json"),
      JSON.stringify({ clicks: 1, route: "/about", pageErrors }, null, 2) + "\n",
    );
  } finally {
    await browser?.close();
    child.kill("SIGTERM");
    await serverExited;
    fs.closeSync(serverFd);
  }
  fs.writeFileSync(
    path.join(artifacts, "proof.json"),
    JSON.stringify(
      {
        candidateHead: process.env.GITHUB_SHA ?? null,
        node: process.version,
        nuxt: "3.19.3",
        buildExit: build.status,
        routes: ["/", "/about", "/page-meta", "/page-meta-global"],
        hydratedInteraction: true,
        sourceBinding: sourceBinding.verify(),
      },
      null,
      2,
    ) + "\n",
  );
  console.log("Candidate Nuxt 3 build, SSR, hydration, and client navigation passed");
} finally {
  for (const server of servers) if (server.exitCode === null) server.kill("SIGTERM");
  for (const item of swapped) {
    fs.rmSync(item.installed, { recursive: true, force: true });
    fs.cpSync(item.backup, item.installed, { recursive: true });
  }
}
