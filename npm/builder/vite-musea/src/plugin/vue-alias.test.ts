import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { createVueRuntimeCompilerAlias } from "./vue-alias.js";

void test("Vue runtime compiler alias only matches the bare vue import", () => {
  const alias = createVueRuntimeCompilerAlias();
  assert.equal(alias.find.test("vue"), true);
  assert.equal(alias.find.test("vue/server-renderer"), false);
  assert.equal(alias.find.test("vue/dist/vue.runtime.esm-bundler.js"), false);
  assert.match(alias.replacement, /vue[\\/]dist[\\/]vue\.esm-bundler\.js$/);
});

void test("Vue runtime compiler alias resolves from the project root", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "musea-vue-alias-"));
  const runtime = path.join(root, "node_modules", "vue", "dist", "vue.esm-bundler.js");
  fs.mkdirSync(path.dirname(runtime), { recursive: true });
  fs.writeFileSync(path.join(root, "package.json"), "{}\n");
  fs.writeFileSync(runtime, "export {}\n");

  const alias = createVueRuntimeCompilerAlias({ root });
  assert.equal(alias.replacement, fs.realpathSync(runtime));
});

void test("Vue runtime compiler alias accepts an explicit replacement", () => {
  const alias = createVueRuntimeCompilerAlias({
    root: "/workspace/app",
    runtimeCompiler: "./vendor/vue-compiler.js",
  });

  assert.equal(alias.replacement, path.resolve("/workspace/app/vendor/vue-compiler.js"));
});
