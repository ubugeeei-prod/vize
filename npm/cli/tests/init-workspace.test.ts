import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import { test } from "vite-plus/test";

import { detectPackageManager } from "../src/init/detect.ts";

import { manifest, runInit, temporaryProject, write, writeManifest } from "./init-support.ts";

const VITE_CONFIG = `import { defineConfig } from "vite";

export default defineConfig({
  plugins: [],
});
`;

/** A workspace root with one Vite package at `packages/app`; returns the package. */
function workspacePackage(name: string, rootFiles: Readonly<Record<string, string>>): string {
  const workspace = temporaryProject(name);
  fs.mkdirSync(path.join(workspace, ".git"));
  writeManifest(workspace, manifest({ name: "workspace-root" }));
  for (const [filename, source] of Object.entries(rootFiles)) {
    write(workspace, filename, source);
  }
  const project = path.join(workspace, "packages", "app");
  writeManifest(project, manifest({ devDependencies: { typescript: "^5.9.0", vite: "^8.0.0" } }));
  write(project, "vite.config.ts", VITE_CONFIG);
  write(project, "tsconfig.json", "{}\n");
  return project;
}

test("a workspace package installs with the workspace's package manager", async () => {
  const project = workspacePackage("workspace-pnpm", {
    "pnpm-workspace.yaml": 'packages:\n  - "packages/*"\n',
    "pnpm-lock.yaml": "lockfileVersion: '9.0'\n",
  });

  const result = await runInit(project, ["--yes", "--no-lint", "--vite", "--no-fmt", "--dry-run"]);

  assert.match(result.output, /\n {2}package manager: pnpm\n/);
  assert.match(result.output, /\[vize init\] would run: pnpm add -D @vizejs\/vite-plugin vize\n/);
  assert.doesNotMatch(result.output, /npm install/);
});

test("the workspace root's packageManager field applies to its packages", () => {
  const project = workspacePackage("workspace-field", {});
  const workspace = path.dirname(path.dirname(project));
  writeManifest(workspace, manifest({ name: "workspace-root", packageManager: "yarn@4.9.2" }));

  assert.equal(detectPackageManager(project), "yarn");
});

test("the nearest lockfile wins over one further up", () => {
  const project = workspacePackage("workspace-nearest", { "pnpm-lock.yaml": "" });
  write(project, "package-lock.json", "{}\n");

  assert.equal(detectPackageManager(project), "npm");
});

test("a lockfile above the repository root is not the project's", () => {
  const outside = temporaryProject("workspace-outside");
  write(outside, "yarn.lock", "");
  const repository = path.join(outside, "repository");
  fs.mkdirSync(path.join(repository, ".git"), { recursive: true });
  writeManifest(repository, manifest());
  const nested = path.join(repository, "packages", "app");
  writeManifest(nested, manifest());

  assert.equal(detectPackageManager(repository), null);
  assert.equal(detectPackageManager(nested), null);
});
