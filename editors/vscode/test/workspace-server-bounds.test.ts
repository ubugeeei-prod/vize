import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test, { type TestContext } from "node:test";

import { discoverWorkspaceServers, workspaceServerProbePaths } from "../src/workspace-server.ts";

/** A directory other accounts could also write to, with one project inside it. */
function sharedParent(t: TestContext): { shared: string; project: string } {
  const shared = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-shared-parent-")));
  t.after(() => fs.rmSync(shared, { recursive: true, force: true }));
  const project = path.join(shared, "project");
  fs.mkdirSync(project);
  return { shared, project };
}

function manifest(directory: string): void {
  fs.mkdirSync(directory, { recursive: true });
  fs.writeFileSync(path.join(directory, "package.json"), "{}");
}

/** Installs a `vize` package under `directory/node_modules` and returns its executable. */
function install(directory: string): string {
  const root = path.join(directory, "node_modules", "vize");
  const executable = path.join(root, "bin", "vize.cjs");
  fs.mkdirSync(path.dirname(executable), { recursive: true });
  fs.writeFileSync(executable, "#!/usr/bin/env node\n");
  fs.writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({ name: "vize", version: "9.8.7", bin: { vize: "bin/vize.cjs" } }),
  );
  return executable;
}

/** Links `directory/node_modules/.bin/vize` the way npm does for a Node entry. */
function installBin(directory: string): string {
  const bin = path.join(directory, "node_modules", ".bin", "vize");
  fs.mkdirSync(path.dirname(bin), { recursive: true });
  fs.writeFileSync(bin, "#!/usr/bin/env node\n");
  return bin;
}

void test("a server installed above the workspace folder is not used", (t) => {
  const { shared, project } = sharedParent(t);
  manifest(project);
  install(shared);
  installBin(shared);
  assert.deepEqual(discoverWorkspaceServers([project], true, "linux"), { candidates: [] });
});

void test("a package manifest above the workspace folder does not select a server", (t) => {
  const { shared, project } = sharedParent(t);
  manifest(shared);
  install(shared);
  installBin(shared);
  assert.deepEqual(discoverWorkspaceServers([project], true, "linux"), { candidates: [] });
});

void test("the workspace folder's own installation is used", (t) => {
  const { shared, project } = sharedParent(t);
  manifest(project);
  install(shared);
  const own = install(project);
  assert.deepEqual(discoverWorkspaceServers([project], true, "linux"), {
    candidates: [{ path: own, source: "workspace" }],
  });
});

void test("the workspace folder's own bin link is used", (t) => {
  const { shared, project } = sharedParent(t);
  manifest(project);
  installBin(shared);
  const own = installBin(project);
  assert.deepEqual(discoverWorkspaceServers([project], true, "linux"), {
    candidates: [{ path: own, source: "workspace" }],
  });
});

void test("a nested package folder uses the server installed at its workspace folder root", (t) => {
  const { shared, project } = sharedParent(t);
  manifest(project);
  install(shared);
  const hoisted = install(project);
  const nested = path.join(project, "packages", "app");
  manifest(nested);
  assert.deepEqual(discoverWorkspaceServers([project, nested], true, "linux"), {
    candidates: [{ path: hoisted, source: "workspace" }],
  });
  // A folder below the package still resolves through its package to the root.
  const source = path.join(nested, "src");
  fs.mkdirSync(source);
  assert.deepEqual(discoverWorkspaceServers([source, project], true, "linux"), {
    candidates: [{ path: hoisted, source: "workspace" }],
  });
});

void test("a nested package opened on its own does not reach the installation above it", (t) => {
  const { project } = sharedParent(t);
  manifest(project);
  install(project);
  const nested = path.join(project, "packages", "app");
  manifest(nested);
  assert.deepEqual(discoverWorkspaceServers([nested], true, "linux"), { candidates: [] });
});

void test("discovery probes no path above the workspace folders", (t) => {
  const { project } = sharedParent(t);
  const nested = path.join(project, "packages", "app");
  const probes = (directory: string) => [
    path.join(directory, "package.json"),
    path.join(directory, "node_modules", "vize", "package.json"),
    path.join(directory, "node_modules", ".bin", "vize"),
  ];
  assert.deepEqual(workspaceServerProbePaths([project], "linux"), probes(project));
  assert.deepEqual(workspaceServerProbePaths([nested, project], "linux"), [
    ...probes(nested),
    ...probes(path.join(project, "packages")),
    ...probes(project),
  ]);
});
