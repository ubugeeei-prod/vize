import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

import {
  authoredNuxtPrerequisiteTasks,
  retainNuxtPrerequisites,
} from "./nuxt-check-prerequisites.ts";
import { walkFiles } from "./nuxt-check-source.ts";
import { type PinnedFixtureSource, sha256 } from "./pinned-fixture-source.ts";

/** Run the authored generators against the already authenticated installer. */
export function prepareInstalledNpmxNuxt(
  root: string,
  installRoot: string,
  source: PinnedFixtureSource,
  env: NodeJS.ProcessEnv,
) {
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  const tasks = authoredNuxtPrerequisiteTasks("npmx.dev", manifest.scripts?.postinstall);
  assert.match(
    manifest.scripts.postinstall,
    /(?:^|&&)\s*nuxt prepare\s*(?:&&|$)/,
    "npmX must retain its authored root Nuxt preparation",
  );
  const workspaceModules: Record<string, string> = {};
  for (const blob of source.blobs) {
    if (!blob.path.endsWith("/package.json")) continue;
    const workspace = path.dirname(blob.path);
    const installedModules = path.join(installRoot, workspace, "node_modules");
    if (!fs.existsSync(installedModules)) continue;
    assert.deepEqual(
      fs.readFileSync(path.join(root, blob.path)),
      fs.readFileSync(path.join(installRoot, blob.path)),
      `workspace installer manifest differs: ${workspace}`,
    );
    const modules = path.join(root, workspace, "node_modules");
    assert.ok(!fs.existsSync(modules), `workspace dependencies already attached: ${workspace}`);
    fs.symlinkSync(installedModules, modules, process.platform === "win32" ? "junction" : "dir");
    workspaceModules[workspace] = fs.realpathSync(installedModules);
  }

  const require = createRequire(path.join(root, "package.json"));
  const providers = [
    ["vite-plus", "vp"],
    ["nuxt", "nuxt"],
  ].map(([name, command]) => {
    const manifestPath = require.resolve(`${name}/package.json`);
    const bytes = fs.readFileSync(manifestPath);
    const pkg = JSON.parse(bytes.toString());
    assert.equal(pkg.name, name);
    const bin = typeof pkg.bin === "string" ? pkg.bin : pkg.bin?.[command];
    assert.equal(typeof bin, "string", `${name} must supply its authored ${command} command`);
    const entrypoint = path.resolve(path.dirname(manifestPath), bin);
    assert.ok(entrypoint.startsWith(`${path.dirname(manifestPath)}${path.sep}`));
    return {
      name,
      version: pkg.version,
      manifest: manifestPath,
      manifestSha256: sha256(bytes),
      entrypoint,
      entrypointSha256: sha256(fs.readFileSync(entrypoint)),
    };
  });
  const commands: string[][] = [];
  function run(entrypoint: string, args: string[], timeout: number): void {
    commands.push([process.execPath, entrypoint, ...args]);
    // pnpm 11 exec automatically reinstalls when it notices the deliberately
    // separate installer manifest. Use its installed commands directly instead.
    execFileSync(process.execPath, [entrypoint, ...args], {
      cwd: root,
      env,
      stdio: "inherit",
      timeout,
    });
  }
  for (const task of tasks) run(providers[0].entrypoint, ["run", "--no-cache", task], 120_000);
  run(providers[1].entrypoint, ["prepare"], 600_000);

  const generatedInputs: string[] = [];
  retainNuxtPrerequisites("npmx.dev", root, (file) =>
    generatedInputs.push(path.relative(root, file)),
  );
  for (const file of [
    "imports.d.ts",
    "types/imports.d.ts",
    "types/typed-router.d.ts",
    "tsconfig.app.json",
    "tsconfig.server.json",
    "tsconfig.shared.json",
    "tsconfig.node.json",
  ]) {
    assert.ok(fs.statSync(path.join(root, ".nuxt", file)).isFile(), `missing Nuxt input: ${file}`);
  }
  walkFiles(path.join(root, ".nuxt"), (file) => generatedInputs.push(path.relative(root, file)));
  const verify = () => {
    for (const [workspace, expected] of Object.entries(workspaceModules)) {
      assert.equal(fs.realpathSync(path.join(root, workspace, "node_modules")), expected);
    }
    for (const provider of providers) {
      assert.equal(sha256(fs.readFileSync(provider.manifest)), provider.manifestSha256);
      assert.equal(sha256(fs.readFileSync(provider.entrypoint)), provider.entrypointSha256);
    }
  };
  verify();
  return { generatedInputs, receipt: { commands, providers, workspaceModules }, verify };
}
