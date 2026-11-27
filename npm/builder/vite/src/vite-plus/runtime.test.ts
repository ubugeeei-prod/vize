import { readToolingProgram } from "../../../../../tests/tooling/support/program-path.ts";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { availableVueRules, resetAvailableVueRulesForTests } from "./runtime.ts";

function inventory(root: string): { files: number; bytes: number } {
  let files = 0;
  let bytes = 0;
  const pending = [root];
  while (pending.length > 0) {
    const directory = pending.pop();
    assert.ok(directory);
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) pending.push(file);
      else if (entry.isFile()) {
        files += 1;
        bytes += fs.statSync(file).size;
      }
    }
  }
  return { files, bytes };
}

void test("rule discovery uses a bounded owned cache without process-temporary files", (t) => {
  const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-rule-catalog-custody-"));
  t.after(() => fs.rmSync(owner, { recursive: true, force: true }));
  const project = path.join(owner, "project with spaces 界");
  const packageRoot = path.join(project, "node_modules", "vite-plus");
  const binary = path.join(packageRoot, "bin", "vp.mjs");
  const counter = path.join(owner, "spawns.txt");
  const cache = path.join(owner, "cache");
  const temporary = path.join(owner, "temporary");
  fs.mkdirSync(path.dirname(binary), { recursive: true });
  fs.mkdirSync(temporary);
  fs.writeFileSync(path.join(project, "package.json"), '{"private":true}');
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    JSON.stringify({ name: "vite-plus", bin: { vp: "bin/vp.mjs" } }),
  );
  fs.writeFileSync(
    binary,
    readToolingProgram(
      new URL(
        "../../scripts/test-programs/vite-plus-runtime/list-vue-lint-rules.js",
        import.meta.url,
      ),
    ),
  );

  const previous = {
    cwd: process.cwd(),
    cache: process.env.XDG_CACHE_HOME,
    counter: process.env.VIZE_RULE_CATALOG_TEST_COUNTER,
    temporary: process.env.TMPDIR,
  };
  t.after(() => {
    process.chdir(previous.cwd);
    restoreEnvironment("XDG_CACHE_HOME", previous.cache);
    restoreEnvironment("VIZE_RULE_CATALOG_TEST_COUNTER", previous.counter);
    restoreEnvironment("TMPDIR", previous.temporary);
    resetAvailableVueRulesForTests();
  });
  process.chdir(project);
  process.env.XDG_CACHE_HOME = cache;
  process.env.VIZE_RULE_CATALOG_TEST_COUNTER = counter;
  process.env.TMPDIR = temporary;

  resetAvailableVueRulesForTests();
  assert.deepEqual([...availableVueRules()], ["no-v-html"]);
  assert.equal(fs.readFileSync(counter, "utf8"), "spawn\n");
  assert.deepEqual(fs.readdirSync(temporary), []);

  const catalogRoot = path.join(cache, "vize", "vite-plus", "oxlint-rules-v1");
  const catalogs = catalogRoot;
  assert.equal(
    fs.readFileSync(path.join(catalogRoot, "package.json"), "utf8"),
    '{"name":"vize-rule-catalog","private":true}',
  );
  assert.deepEqual(inventory(catalogRoot), { files: 2, bytes: 56 });
  if (process.platform !== "win32") {
    assert.equal(fs.statSync(catalogRoot).mode & 0o777, 0o700);
    const catalog = fs
      .readdirSync(catalogs)
      .find((name) => name.endsWith(".json") && name !== "package.json");
    assert.ok(catalog);
    assert.equal(fs.statSync(path.join(catalogs, catalog)).mode & 0o777, 0o600);
  }

  // Forget process memory: the second request must be served by the published
  // immutable file rather than another CLI process.
  resetAvailableVueRulesForTests();
  assert.deepEqual([...availableVueRules()], ["no-v-html"]);
  assert.equal(fs.readFileSync(counter, "utf8"), "spawn\n");

  // A cache leaf cannot redirect catalog reads through a symlink. Windows
  // developer-mode symlink privileges vary, so its path defense is exercised
  // by the same lstat branch in the cross-platform source tests.
  if (process.platform !== "win32") {
    const name = fs
      .readdirSync(catalogs)
      .find((entry) => entry.endsWith(".json") && entry !== "package.json");
    assert.ok(name);
    const catalog = path.join(catalogs, name);
    const poison = path.join(owner, "poison.json");
    fs.writeFileSync(poison, '["poison"]');
    fs.rmSync(catalog);
    fs.symlinkSync(poison, catalog);
    resetAvailableVueRulesForTests();
    assert.deepEqual([...availableVueRules()], ["no-v-html"]);
    assert.equal(fs.lstatSync(catalog).isSymbolicLink(), false);
  }

  // A killed publisher can leave only an exclusive .tmp inode. Dead owners are
  // recovered, live owners remain isolated, and immutable catalogs stay bounded.
  fs.writeFileSync(path.join(catalogs, `stale.99999999.${randomUUID()}.tmp`), "stale");
  for (let index = 0; index < 12; index += 1) {
    const file = path.join(catalogs, `${index.toString(16).padStart(64, "0")}.json`);
    fs.writeFileSync(file, "[]");
    fs.utimesSync(file, index + 1, index + 1);
  }
  fs.utimesSync(binary, new Date(), new Date());
  resetAvailableVueRulesForTests();
  assert.deepEqual([...availableVueRules()], ["no-v-html"]);
  assert.equal(
    fs.readdirSync(catalogs).filter((name) => /^[a-f0-9]{64}\.json$/u.test(name)).length,
    8,
  );
  assert.equal(
    fs.readdirSync(catalogs).some((name) => name.endsWith(".tmp")),
    false,
  );
});

void test("the installed Vite+ returns its rule catalog from a gitignored cache", (t) => {
  const installed = createRequire(import.meta.url).resolve("vite-plus/package.json");
  const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ignored-rule-catalog-"));
  const project = path.join(owner, "project with spaces");
  fs.mkdirSync(path.join(project, "node_modules"), { recursive: true });
  fs.writeFileSync(path.join(project, "package.json"), '{"private":true}');
  fs.writeFileSync(path.join(project, ".gitignore"), ".cache/\n");
  fs.symlinkSync(path.dirname(installed), path.join(project, "node_modules/vite-plus"), "junction");
  const initialized = spawnSync("git", ["init", "--quiet", project], { encoding: "utf8" });
  assert.equal(initialized.status, 0, initialized.stderr);

  const previous = { cwd: process.cwd(), cache: process.env.XDG_CACHE_HOME };
  t.after(() => {
    process.chdir(previous.cwd);
    restoreEnvironment("XDG_CACHE_HOME", previous.cache);
    resetAvailableVueRulesForTests();
    fs.rmSync(owner, { recursive: true, force: true });
  });
  process.chdir(project);
  process.env.XDG_CACHE_HOME = path.join(owner, "ordinary-cache");
  resetAvailableVueRulesForTests();
  const expected = [...availableVueRules()].sort();
  assert.ok(expected.includes("component-definition-name-casing"));

  process.env.XDG_CACHE_HOME = path.join(project, ".cache");
  resetAvailableVueRulesForTests();
  assert.deepEqual([...availableVueRules()].sort(), expected);
});

function restoreEnvironment(name: string, value: string | undefined): void {
  if (value == null) delete process.env[name];
  else process.env[name] = value;
}
