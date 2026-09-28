// Exercise the packed candidate module and Vite adapter in a real Nuxt 3 build.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tools/support/compat/nuxt/fixtures/nuxt3-module-build");
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt3-build"));
fs.mkdirSync(artifacts, { recursive: true });

for (const [name, version] of [
  ["nuxt", "3.19.3"],
  ["vue", "3.5.43"],
  ["@vizejs/nuxt", "0.429.1"],
  ["@vizejs/vite-plugin", "0.429.1"],
]) {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(fixture, "node_modules", name, "package.json"), "utf8"),
  );
  assert.equal(manifest.version, version, `fixture must use the exact ${name} version`);
}

const swapped = [
  ["nuxt", path.join(root, "npm/framework/nuxt/dist")],
  ["vite-plugin", path.join(root, "npm/builder/vite/dist")],
].map(([name, candidate]) => ({
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

let server;
try {
  for (const name of [".nuxt", ".output"])
    fs.rmSync(path.join(fixture, name), { recursive: true, force: true });
  const buildLog = path.join(artifacts, "build.log");
  const buildFd = fs.openSync(buildLog, "w");
  let build;
  try {
    build = spawnSync(process.execPath, ["node_modules/nuxt/bin/nuxt.mjs", "build"], {
      cwd: fixture,
      env: { ...process.env, NO_COLOR: "1" },
      stdio: ["ignore", buildFd, buildFd],
      timeout: 300_000,
    });
  } finally {
    fs.closeSync(buildFd);
  }
  assert.equal(build.error, undefined);
  assert.equal(build.status, 0, `Nuxt 3 build failed; see ${buildLog}`);

  const socket = net.createServer();
  await new Promise((resolve, reject) => {
    socket.once("error", reject);
    socket.listen(0, "127.0.0.1", resolve);
  });
  const port = socket.address().port;
  await new Promise((resolve) => socket.close(resolve));

  const serverFd = fs.openSync(path.join(artifacts, "server.log"), "w");
  server = spawn(process.execPath, [path.join(fixture, ".output/server/index.mjs")], {
    cwd: fixture,
    env: { ...process.env, HOST: "127.0.0.1", PORT: String(port), NO_COLOR: "1" },
    stdio: ["ignore", serverFd, serverFd],
  });
  const serverExited = new Promise((resolve) => server.once("exit", resolve));
  try {
    for (const [route, heading] of [
      ["/", "Nuxt 3 with Vize"],
      ["/about", "Nuxt 3 route"],
    ]) {
      let response;
      for (let attempt = 0; attempt < 100; attempt++) {
        try {
          response = await fetch(`http://127.0.0.1:${port}${route}`, {
            signal: AbortSignal.timeout(1000),
          });
          break;
        } catch {
          if (server.exitCode !== null) break;
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
  } finally {
    server.kill("SIGTERM");
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
        routes: ["/", "/about"],
      },
      null,
      2,
    ) + "\n",
  );
  console.log("Candidate Nuxt 3 build and SSR routes passed");
} finally {
  if (server && server.exitCode === null) server.kill("SIGTERM");
  for (const item of swapped) {
    fs.rmSync(item.installed, { recursive: true, force: true });
    fs.cpSync(item.backup, item.installed, { recursive: true });
  }
}
