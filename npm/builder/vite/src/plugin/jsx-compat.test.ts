import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import type { Plugin, ResolvedConfig, TransformResult } from "vite";

import { vize } from "./index.ts";

const SOURCE = "const A = () => <input disabled/>;";
const NATIVE_OUTPUT = `import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"
export function render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("input", { disabled: "" }))
}`;
const BABEL_OUTPUT = `import { openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"
export function render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("input", { disabled: true }))
}`;

function functionHook<T extends Function>(hook: T | { handler: T } | undefined, name: string): T {
  const handler = typeof hook === "function" ? hook : hook?.handler;
  assert.equal(typeof handler, "function", `${name} hook should exist`);
  return handler as T;
}

async function transformWithProjectConfig(compiler: Record<string, unknown>): Promise<string> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-jsx-compat-"));
  const id = path.join(root, "App.jsx");
  const resolvedConfig = {
    root,
    base: "/",
    mode: "production",
    command: "build",
    isProduction: true,
    build: { assetsDir: "assets", ssr: false, sourcemap: false },
    define: {},
    plugins: [],
    resolve: { alias: [] },
  } as unknown as ResolvedConfig;

  try {
    fs.writeFileSync(path.join(root, "vize.config.json"), JSON.stringify({ compiler }));
    fs.writeFileSync(id, SOURCE);

    const plugin = vize({ include: /\.jsx$/, root }).find(
      (candidate) => candidate.name === "vite-plugin-vize",
    ) as Plugin;
    const configResolved = functionHook(plugin.configResolved, "configResolved") as (
      config: ResolvedConfig,
    ) => Promise<void>;
    await configResolved(resolvedConfig);

    const transform = functionHook(plugin.transform, "transform") as (
      code: string,
      id: string,
      options: { ssr: boolean },
    ) => Promise<TransformResult | null>;
    const result = await transform(SOURCE, id, { ssr: false });
    assert.ok(result && typeof result === "object");
    return result.code;
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
}

void test("compiler.jsxCompat changes the complete Vite JSX output only when opted in", async () => {
  const nativeOutput = await transformWithProjectConfig({});
  const babelOutput = await transformWithProjectConfig({ jsxCompat: "babel" });

  assert.equal(nativeOutput, NATIVE_OUTPUT);
  assert.equal(babelOutput, BABEL_OUTPUT);
  assert.notEqual(babelOutput, nativeOutput);
});
