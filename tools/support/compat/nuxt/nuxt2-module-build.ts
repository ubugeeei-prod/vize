// Exercise the packed candidate module in a real Nuxt 2 webpack build and SSR server.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import type { ChildProcess } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { at, readJson } from "../../json-value.ts";
import { freePort } from "./host-types.ts";
import { verifyNuxt2LintConfig } from "./nuxt2-lint-config.ts";
import { verifyNuxtAutoInitConfig } from "./nuxt-auto-init-config.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tools/support/compat/nuxt/fixtures/nuxt2-module-build");
const packageRoot = path.join(fixture, "node_modules/@vizejs/nuxt");
const candidateDist = path.join(root, "npm/framework/nuxt/dist");
const artifacts = path.resolve(process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt2-build"));
fs.mkdirSync(artifacts, { recursive: true });

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
  let result;
  try {
    result = spawnSync(process.execPath, ["node_modules/nuxt/bin/nuxt.js", "build"], {
      cwd: fixture,
      env: { ...process.env, NO_COLOR: "1" },
      stdio: ["ignore", log, log],
      timeout: 300_000,
    });
  } finally {
    fs.closeSync(log);
  }
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `Nuxt 2 build failed; see ${buildLog}`);

  const port = await freePort();

  const serverLog = fs.openSync(path.join(artifacts, "server.log"), "w");
  const child = spawn(process.execPath, ["node_modules/nuxt/bin/nuxt.js", "start"], {
    cwd: fixture,
    env: { ...process.env, HOST: "127.0.0.1", PORT: String(port), NO_COLOR: "1" },
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
  } finally {
    child.kill("SIGTERM");
    await serverExited;
    fs.closeSync(serverLog);
  }
  verifyNuxt2LintConfig(fixture, artifacts);
  verifyNuxtAutoInitConfig(fixture, artifacts);
  fs.writeFileSync(
    path.join(artifacts, "proof.json"),
    JSON.stringify(
      {
        candidateHead: process.env.GITHUB_SHA ?? null,
        node: process.version,
        nuxt: "2.17.3",
        buildExit: result.status,
        routes: ["/", "/about"],
      },
      null,
      2,
    ) + "\n",
  );
  console.log("Candidate Nuxt 2 webpack build and SSR routes passed");
} finally {
  for (const server of servers) if (server.exitCode === null) server.kill("SIGTERM");
  for (const item of swapped) {
    fs.rmSync(item.installed, { recursive: true, force: true });
    fs.cpSync(item.backup, item.installed, { recursive: true });
  }
}
