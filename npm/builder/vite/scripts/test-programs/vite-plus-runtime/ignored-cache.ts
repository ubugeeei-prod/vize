import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";

import {
  availableVueRules,
  resetAvailableVueRulesForTests,
} from "../../../src/vite-plus/runtime.ts";

assert.equal(process.versions.bun, undefined, "exercise Vite's Node host");
const installed =
  process.env.VIZE_TEST_VP_PACKAGE ??
  createRequire(import.meta.url).resolve("vite-plus/package.json");
const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ignored-rule-catalog-"));
const previous = { cwd: process.cwd(), cache: process.env.XDG_CACHE_HOME, tmp: process.env.TMPDIR };
try {
  const project = path.join(owner, "project with spaces");
  fs.mkdirSync(path.join(project, "node_modules"), { recursive: true });
  fs.writeFileSync(path.join(project, "package.json"), '{"private":true,"type":"module"}');
  fs.writeFileSync(path.join(project, ".gitignore"), ".cache/\ndist/\nnode_modules/\n");
  fs.writeFileSync(
    path.join(project, "vite.config.ts"),
    'export default { lint: { ignorePatterns: ["dist/**", "node_modules/**"] } };',
  );
  const output = path.join(project, "dist");
  fs.mkdirSync(output);
  fs.copyFileSync(
    process.env.VIZE_TEST_RUNTIME_SOURCE ??
      new URL("../../../src/vite-plus/runtime.ts", import.meta.url),
    path.join(output, "runtime.ts"),
  );
  const packed = await import(pathToFileURL(path.join(output, "runtime.ts")).href);
  fs.symlinkSync(path.dirname(installed), path.join(project, "node_modules/vite-plus"), "junction");
  const initialized = spawnSync("git", ["init", "--quiet", project], { encoding: "utf8" });
  assert.equal(initialized.status, 0, initialized.stderr);
  const temporary = path.join(owner, "process-temp");
  fs.mkdirSync(temporary);
  process.env.TMPDIR = temporary;
  process.chdir(project);
  process.env.XDG_CACHE_HOME = path.join(owner, "ordinary-cache");
  resetAvailableVueRulesForTests();
  const expected = [...availableVueRules()].sort();
  assert.ok(expected.includes("valid-define-props"));
  process.env.XDG_CACHE_HOME = path.join(project, ".cache");
  resetAvailableVueRulesForTests();
  assert.deepEqual([...availableVueRules()].sort(), expected);
  // Installed packages often execute from dist under a consumer ignore rule.
  // Their catalog must use an independently owned input as well.
  packed.resetAvailableVueRulesForTests();
  process.env.XDG_CACHE_HOME = path.join(project, ".cache-packed");
  assert.deepEqual(
    [...packed.availableVueRules()].sort((left: string, right: string) =>
      left.localeCompare(right),
    ),
    expected,
  );
  assert.deepEqual(fs.readdirSync(temporary), [], "catalog retries leave no temporary files");
} finally {
  process.chdir(previous.cwd);
  if (previous.cache == null) delete process.env.XDG_CACHE_HOME;
  else process.env.XDG_CACHE_HOME = previous.cache;
  if (previous.tmp == null) delete process.env.TMPDIR;
  else process.env.TMPDIR = previous.tmp;
  resetAvailableVueRulesForTests();
  fs.rmSync(owner, { recursive: true, force: true });
}
