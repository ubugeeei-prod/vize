// `.tsrx` modules through a real Vite build compile with the native TSRX entry
// (#13140): the bundled component is the entry's output and renders, an SSR
// build (no SSR emission yet) and a refused module fail with every diagnostic,
// and `exclude` leaves a module to Vite.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { compileTsrx } from "@vizejs/native";
import { build, type Rollup } from "vite";

import { vize } from "./index.ts";

const require = createRequire(import.meta.url);
const vueRuntime = require.resolve("vue");
const serverRenderer = require.resolve("vue/server-renderer");
const fixtures = path.resolve(import.meta.dirname, "../../../../tests/_fixtures/tsrx");
const counter = fs.readFileSync(path.join(fixtures, "mount/counter.tsrx"), "utf8");

async function bundle(
  files: Record<string, string>,
  options: { ssr?: boolean; exclude?: RegExp } = {},
): Promise<{ root: string; code: string }> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-tsrx-"));
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), source);
  }
  const entry = path.join(root, "entry.js");
  try {
    const result = (await build({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [
        vize({ configMode: false, ...(options.exclude ? { exclude: options.exclude } : {}) }),
      ],
      build: {
        write: false,
        minify: false,
        ...(options.ssr ? { ssr: entry } : { lib: { entry, formats: ["es"], fileName: "app" } }),
        rollupOptions: { external: [/^vue/] },
      },
    })) as Rollup.RollupOutput | Rollup.RollupOutput[];
    const chunk = (Array.isArray(result) ? result[0] : result).output.find(
      (item): item is Rollup.OutputChunk => item.type === "chunk",
    );
    return { root, code: chunk?.code ?? "" };
  } catch (error) {
    fs.rmSync(root, { recursive: true, force: true });
    throw error;
  }
}

async function renderCounter(root: string, code: string): Promise<string> {
  const file = path.join(root, "app.mjs");
  fs.writeFileSync(
    file,
    code.replace(/from ["']vue["']/g, `from ${JSON.stringify(pathToFileURL(vueRuntime).href)}`),
  );
  const { Counter } = (await import(pathToFileURL(file).href)) as { Counter: object };
  const { createSSRApp } = (await import(pathToFileURL(vueRuntime).href)) as typeof import("vue");
  const { renderToString } = (await import(pathToFileURL(serverRenderer).href)) as {
    renderToString: (app: unknown) => Promise<string>;
  };
  return renderToString(createSSRApp(Counter, { title: "clicks", step: 1 }));
}

void test("vite client: a .tsrx module bundles the native entry's output and renders", async () => {
  const { root, code } = await bundle({
    "Counter.tsrx": counter,
    "entry.js": 'export { Counter } from "./Counter.tsrx";\n',
  });
  try {
    const native = compileTsrx(counter, {
      filename: path.join(root, "Counter.tsrx"),
      target: "dom",
      eraseTypes: true,
    });
    assert.deepEqual(native.errors, []);
    // The bundle holds the compiled component, not TSRX markup.
    assert.doesNotMatch(code, /@\{|@if/);
    assert.match(code, /defineComponent/);
    assert.equal(
      await renderCounter(root, code),
      '<section><h1>CLICKS</h1><span class="label">clicks: 0</span><button>add</button><p class="none">none</p></section>',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("vite: a refused .tsrx module fails the build with every diagnostic", async () => {
  const refused = "export const app = () => <p>hi</p>;\n";
  await assert.rejects(
    bundle({ "App.tsrx": refused, "entry.js": 'export { app } from "./App.tsrx";\n' }),
    (error: Error) => {
      assert.match(
        error.message,
        /\[vize\] TSRX compilation failed in .*App\.tsrx:\n {2}- .*App\.tsrx:1:26 \[unsupported\] OutsideComponent is not emitted for the DOM target yet/,
      );
      return true;
    },
  );
  await assert.rejects(
    bundle(
      { "Counter.tsrx": counter, "entry.js": 'export { Counter } from "./Counter.tsrx";\n' },
      { ssr: true },
    ),
    (error: Error) => {
      assert.match(
        error.message,
        /Counter\.tsrx:1:1 \[unsupported\] Ssr emission of TSRX modules is not supported yet/,
      );
      return true;
    },
  );
});

void test("vite: an excluded .tsrx module is left to Vite", async () => {
  await assert.rejects(
    bundle(
      { "Counter.tsrx": counter, "entry.js": 'export { Counter } from "./Counter.tsrx";\n' },
      { exclude: /Counter\.tsrx$/ },
    ),
    (error: Error) => {
      assert.doesNotMatch(error.message, /\[vize\]/);
      return true;
    },
  );
});
