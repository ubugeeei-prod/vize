import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { createNuxtModuleResolver } from "./resolver.ts";

const packageRoot = fs.realpathSync(fileURLToPath(new URL("..", import.meta.url)));

void test("setup passes nuxt.options.rootDir into the module resolver", () => {
  const source = fs.readFileSync(new URL("./index.ts", import.meta.url), "utf8");
  const resolverSource = fs.readFileSync(new URL("./resolver.ts", import.meta.url), "utf8");
  assert.match(source, /createNuxtModuleResolver\(nuxt\.options\.rootDir\)/);
  assert.doesNotMatch(source, /createNuxtModuleResolver\(\)/);
  assert.doesNotMatch(resolverSource, /process\.cwd\(\)/);
  assert.doesNotMatch(resolverSource, /import\.meta/);
});

void test("createNuxtModuleResolver resolves the package from rootDir while cwd is the parent", () => {
  const removeEntry = ensurePackageEntry();
  const parent = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-resolver-")));
  const appDir = path.join(parent, "app");
  const previousCwd = process.cwd();
  try {
    fs.mkdirSync(path.join(appDir, "node_modules", "@vizejs"), { recursive: true });
    fs.writeFileSync(
      path.join(parent, "package.json"),
      JSON.stringify({ private: true, name: "parent" }),
    );
    fs.writeFileSync(
      path.join(appDir, "package.json"),
      JSON.stringify({
        name: "app",
        dependencies: { "@vizejs/nuxt": "0.474.0" },
      }),
    );
    fs.symlinkSync(packageRoot, path.join(appDir, "node_modules", "@vizejs", "nuxt"));

    process.chdir(parent);
    assert.equal(process.cwd(), parent);

    const resolver = createNuxtModuleResolver(appDir);
    const packageJson = resolver.resolve("../package.json");
    assert.equal(path.basename(packageJson), "package.json");
    assert.equal(
      fs.realpathSync(packageJson),
      fs.realpathSync(path.join(packageRoot, "package.json")),
    );
    assert.equal(JSON.parse(fs.readFileSync(packageJson, "utf8")).name, "@vizejs/nuxt");

    assert.throws(
      () => createNuxtModuleResolver(parent),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.match(error.message, /Cannot find module '@vizejs\/nuxt'/);
        assert.ok(error.message.includes(path.join(parent, "package.json")));
        return true;
      },
    );
  } finally {
    process.chdir(previousCwd);
    fs.rmSync(parent, { recursive: true, force: true });
    removeEntry();
  }
});

// require.resolve follows package exports to dist/nuxt2-entry.cjs. This worktree
// does not build dist, so the lookup needs that entry to exist.
function ensurePackageEntry(): () => void {
  const entry = path.join(packageRoot, "dist", "nuxt2-entry.cjs");
  if (fs.existsSync(entry)) return () => {};
  fs.mkdirSync(path.dirname(entry), { recursive: true });
  fs.writeFileSync(entry, '"use strict";\nmodule.exports = {};\n');
  return () => {
    fs.rmSync(entry, { force: true });
    const dist = path.dirname(entry);
    if (fs.existsSync(dist) && fs.readdirSync(dist).length === 0) fs.rmdirSync(dist);
  };
}
