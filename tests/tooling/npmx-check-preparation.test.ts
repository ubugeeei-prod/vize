import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { prepareInstalledNpmxNuxt } from "../_helpers/npmx-check-preparation.ts";
import type { PinnedFixtureSource } from "../_helpers/pinned-fixture-source.ts";

function fixture(root: string) {
  const checked = path.join(root, "checked");
  const installer = path.join(root, "installer");
  const manifest = JSON.stringify({
    name: "npmx.dev",
    scripts: {
      postinstall: "vp run generate:lexicons && vp run generate:sprite && nuxt prepare",
    },
  });
  for (const directory of [checked, installer]) {
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(path.join(directory, "package.json"), manifest);
    for (const workspace of ["cli", "docs"]) {
      fs.mkdirSync(path.join(directory, workspace));
      fs.writeFileSync(path.join(directory, workspace, "package.json"), '{"name":"workspace"}');
    }
  }
  for (const workspace of ["cli", "docs"]) {
    const provider = path.join(installer, workspace, "node_modules", "provider");
    fs.mkdirSync(provider, { recursive: true });
    fs.writeFileSync(path.join(provider, "package.json"), '{"name":"provider","main":"index.js"}');
    fs.writeFileSync(path.join(provider, "index.js"), "module.exports = 42;\n");
  }
  const child = `
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {createRequire} = require('node:module');
const {execFileSync} = require('node:child_process');
const args = process.argv.slice(2);
fs.appendFileSync(process.env.CALLS, JSON.stringify({args,cwd:process.cwd(),node:process.execPath})+'\\n');
assert.equal(process.env.npm_package_name, 'npmx.dev');
assert.equal(execFileSync('node', ['-p', 'process.execPath'], {encoding:'utf8'}).trim(), process.execPath);
const write = (file, bytes='generated') => { fs.mkdirSync(path.dirname(file),{recursive:true}); fs.writeFileSync(file,bytes); };
if (args.at(-1)==='generate:lexicons') {
  assert.deepEqual(args,['run','--no-cache','generate:lexicons']);
  if(process.env.FAIL_GENERATOR) process.exit(7);
  write('shared/types/lexicons/dev.ts');
} else if(args.at(-1)==='generate:sprite') {
  assert.deepEqual(args,['run','--no-cache','generate:sprite']);
  assert.ok(fs.existsSync('shared/types/lexicons/dev.ts'));
  for(const directory of ['public','public-dev','public-staging']) write(directory+'/file-tree-sprite.svg');
} else {
  assert.deepEqual(args,['prepare']);
  assert.ok(fs.existsSync('public/file-tree-sprite.svg'));
  for(const workspace of ['cli','docs']) assert.equal(createRequire(path.resolve(workspace,'package.json'))('provider'),42);
  for(const file of ['imports.d.ts','types/imports.d.ts','types/typed-router.d.ts','tsconfig.app.json','tsconfig.server.json','tsconfig.shared.json','tsconfig.node.json']) {
    if (process.env.OMIT_IMPORTS && file==='imports.d.ts') continue;
    write('.nuxt/'+file);
  }
}
`;
  for (const [name, command] of [
    ["vite-plus", "vp"],
    ["nuxt", "nuxt"],
  ]) {
    const provider = path.join(installer, "node_modules", name);
    fs.mkdirSync(provider, { recursive: true });
    fs.writeFileSync(
      path.join(provider, "package.json"),
      JSON.stringify({ name, version: "1.2.3", bin: { [command]: "command.cjs" } }),
    );
    fs.writeFileSync(path.join(provider, "command.cjs"), child);
  }
  fs.symlinkSync(
    path.join(installer, "node_modules"),
    path.join(checked, "node_modules"),
    process.platform === "win32" ? "junction" : "dir",
  );
  const source: PinnedFixtureSource = {
    revision: "1".repeat(40),
    inventorySha256: "2".repeat(64),
    gitlinks: [],
    blobs: ["package.json", "cli/package.json", "docs/package.json"].map((file) => ({
      path: file,
      mode: "100644",
      object: "3".repeat(40),
    })),
  };
  const env = {
    ...process.env,
    PATH: `${path.dirname(process.execPath)}${path.delimiter}${process.env.PATH ?? ""}`,
    npm_package_name: "npmx.dev",
    CALLS: path.join(root, "calls.jsonl"),
  };
  return { checked, installer, source, env, manifest };
}

void test("installed npmX commands preserve source and bind workspace providers before Nuxt", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npmx-check-prepare-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const { checked, installer, source, env, manifest } = fixture(root);
  const prepared = prepareInstalledNpmxNuxt(checked, installer, source, env);
  assert.equal(fs.readFileSync(path.join(checked, "package.json"), "utf8"), manifest);
  assert.equal(prepared.generatedInputs.length, 11);
  assert.ok(prepared.generatedInputs.includes(".nuxt/types/typed-router.d.ts"));
  assert.deepEqual(Object.keys(prepared.receipt.workspaceModules), ["cli", "docs"]);
  const calls = fs
    .readFileSync(env.CALLS, "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    calls.map((call) => call.args),
    [
      ["run", "--no-cache", "generate:lexicons"],
      ["run", "--no-cache", "generate:sprite"],
      ["prepare"],
    ],
  );
  assert.ok(calls.every((call) => call.cwd === fs.realpathSync(checked)));
  assert.ok(calls.every((call) => call.node === process.execPath));
  prepared.verify();
  const command = prepared.receipt.providers[0].entrypoint;
  fs.appendFileSync(command, "\n// changed\n");
  assert.throws(prepared.verify, /Expected values to be strictly equal/);
});

void test("failed generators and incomplete Nuxt output cannot produce a check receipt", (t) => {
  for (const failure of ["FAIL_GENERATOR", "OMIT_IMPORTS"] as const) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npmx-check-reject-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    const { checked, installer, source, env } = fixture(root);
    assert.throws(() =>
      prepareInstalledNpmxNuxt(checked, installer, source, { ...env, [failure]: "1" }),
    );
    const calls = fs.readFileSync(env.CALLS, "utf8").trim().split("\n");
    assert.equal(calls.length, failure === "FAIL_GENERATOR" ? 1 : 3);
  }
});
