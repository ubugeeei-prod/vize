import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";

import {
  availableVueRules,
  resetAvailableVueRulesForTests,
} from "../../../src/vite-plus/runtime.ts";

assert.equal(process.versions.bun, undefined, "exercise Vite's Node host");
const installed = createRequire(import.meta.url).resolve("vite-plus/package.json");
const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ignored-rule-catalog-"));
const previous = { cwd: process.cwd(), cache: process.env.XDG_CACHE_HOME };
try {
  const project = path.join(owner, "project with spaces");
  fs.mkdirSync(path.join(project, "node_modules"), { recursive: true });
  fs.writeFileSync(path.join(project, "package.json"), '{"private":true}');
  fs.writeFileSync(path.join(project, ".gitignore"), ".cache/\n");
  fs.symlinkSync(path.dirname(installed), path.join(project, "node_modules/vite-plus"), "junction");
  const initialized = spawnSync("git", ["init", "--quiet", project], { encoding: "utf8" });
  assert.equal(initialized.status, 0, initialized.stderr);
  process.chdir(project);
  process.env.XDG_CACHE_HOME = path.join(owner, "ordinary-cache");
  resetAvailableVueRulesForTests();
  const expected = [...availableVueRules()].sort();
  assert.ok(expected.includes("valid-define-props"));
  process.env.XDG_CACHE_HOME = path.join(project, ".cache");
  resetAvailableVueRulesForTests();
  assert.deepEqual([...availableVueRules()].sort(), expected);
} finally {
  process.chdir(previous.cwd);
  if (previous.cache == null) delete process.env.XDG_CACHE_HOME;
  else process.env.XDG_CACHE_HOME = previous.cache;
  resetAvailableVueRulesForTests();
  fs.rmSync(owner, { recursive: true, force: true });
}
