// #11757/#11758: preserve Nuxt component build modes and standalone server islands.
import assert from "node:assert/strict";
import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { at, readJson } from "../../json-value.ts";
import { assertComponentModesHtml } from "./nuxt-component-modes-assertions.ts";
import { freePort } from "./host-types.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tools/support/compat/nuxt/fixtures/nuxt-component-modes");
const artifacts = path.resolve(
  process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt-component-modes"),
);
const dependencies = fs.realpathSync(path.join(fixture, "node_modules"));
const candidatePackages = new Map([
  ["nuxt", path.join(root, "npm/framework/nuxt/dist")],
  ["vite-plugin", path.join(root, "npm/builder/vite/dist")],
]);
const servers: ChildProcess[] = [];

fs.mkdirSync(artifacts, { recursive: true });

for (const [name, version] of [
  ["nuxt", "4.6.4"],
  ["vue", "3.5.44"],
  ["@vizejs/nuxt", "0.691.0"],
  ["@vizejs/vite-plugin", "0.691.0"],
] as const) {
  assert.equal(at(readJson(path.join(dependencies, name, "package.json")), "version"), version);
}

function copyFixture(workspace: string): void {
  fs.cpSync(fixture, workspace, {
    recursive: true,
    filter: (source) => path.basename(source) !== "node_modules",
  });
}

function linkDependencies(workspace: string, candidate: boolean): void {
  const targetModules = path.join(workspace, "node_modules");
  fs.mkdirSync(path.join(targetModules, "@vizejs"), { recursive: true });
  for (const entry of fs.readdirSync(dependencies)) {
    if (entry === "@vizejs" || entry === ".cache") continue;
    fs.symlinkSync(path.join(dependencies, entry), path.join(targetModules, entry), "dir");
  }
  fs.mkdirSync(path.join(targetModules, ".cache"));
  for (const entry of fs.readdirSync(path.join(dependencies, "@vizejs"))) {
    const installed = path.join(dependencies, "@vizejs", entry);
    const target = path.join(targetModules, "@vizejs", entry);
    const candidateDist = candidatePackages.get(entry);
    if (!candidate || !candidateDist) {
      fs.symlinkSync(installed, target, "dir");
      continue;
    }
    assert.ok(fs.existsSync(path.join(candidateDist, "index.mjs")));
    fs.cpSync(installed, target, { recursive: true });
    fs.rmSync(path.join(target, "dist"), { recursive: true });
    fs.cpSync(candidateDist, path.join(target, "dist"), { recursive: true });
  }
}

async function fetchHtml(child: ChildProcess, port: number): Promise<string> {
  for (let attempt = 0; attempt < 180; attempt++) {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/`, {
        signal: AbortSignal.timeout(1000),
      });
      if (response.ok) return await response.text();
    } catch {
      if (child.exitCode !== null) break;
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  assert.fail(`Nuxt server exited or did not answer on port ${port}`);
}

async function runServer(
  workspace: string,
  label: string,
  command: string[],
  environment: NodeJS.ProcessEnv,
): Promise<string> {
  const port = await freePort();
  const logFd = fs.openSync(path.join(artifacts, `${label}.log`), "w");
  const child = spawn(process.execPath, command, {
    cwd: workspace,
    env: { ...process.env, ...environment, HOST: "127.0.0.1", PORT: String(port), NO_COLOR: "1" },
    stdio: ["ignore", logFd, logFd],
  });
  servers.push(child);
  try {
    const html = await fetchHtml(child, port);
    fs.writeFileSync(path.join(artifacts, `${label}.html`), html);
    return html;
  } finally {
    if (child.exitCode === null) {
      child.kill("SIGTERM");
      await new Promise((resolve) => child.once("exit", resolve));
    }
    fs.closeSync(logFd);
  }
}

try {
  for (const variant of ["stock", "candidate"] as const) {
    const workspace = path.join(artifacts, variant);
    assert.equal(fs.existsSync(workspace), false, `${variant} workspace must start absent`);
    copyFixture(workspace);
    linkDependencies(workspace, variant === "candidate");
    const nuxt = path.join(workspace, "node_modules/nuxt/bin/nuxt.mjs");
    const environment = { VIZE_ENABLED: variant === "candidate" ? "true" : "false" };

    const developmentHtml = await runServer(
      workspace,
      `${variant}-development`,
      [nuxt, "dev", "--host", "127.0.0.1"],
      environment,
    );
    assertComponentModesHtml(developmentHtml, `${variant} development`);

    for (const output of [".nuxt", ".output"]) {
      fs.rmSync(path.join(workspace, output), { recursive: true, force: true });
    }
    const buildLog = fs.openSync(path.join(artifacts, `${variant}-build.log`), "w");
    let build: ReturnType<typeof spawnSync>;
    try {
      build = spawnSync(process.execPath, [nuxt, "build"], {
        cwd: workspace,
        env: { ...process.env, ...environment, NO_COLOR: "1" },
        stdio: ["ignore", buildLog, buildLog],
        timeout: 300_000,
      });
    } finally {
      fs.closeSync(buildLog);
    }
    assert.equal(build.error, undefined);
    assert.equal(build.status, 0, `${variant} production build must pass`);
    const productionHtml = await runServer(
      workspace,
      `${variant}-production`,
      [path.join(workspace, ".output/server/index.mjs")],
      environment,
    );
    assertComponentModesHtml(productionHtml, `${variant} production`);
  }

  fs.writeFileSync(
    path.join(artifacts, "proof.json"),
    JSON.stringify(
      {
        issues: [11757, 11758],
        candidateHead: process.env.GITHUB_SHA ?? null,
        nuxt: "4.6.4",
        vue: "3.5.44",
        variants: ["stock", "candidate"],
        modes: ["development", "production"],
      },
      null,
      2,
    ) + "\n",
  );
  console.log("Nuxt component modes and server islands passed in development and production");
} finally {
  for (const server of servers) if (server.exitCode === null) server.kill("SIGTERM");
}
