// `.tsrx` modules through Rollup and esbuild compile with the native TSRX entry
// (#13140): the bundle holds the stripped entry output and renders, a refused
// module fails with every diagnostic, and `exclude` leaves a module alone.
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { build } from "esbuild";
import { rollup } from "rollup";
import vizeEsbuild from "./esbuild.ts";
import vizeRollup from "./rollup.ts";

const require = createRequire(import.meta.url);
const vueRuntime = require.resolve("vue");
const serverRenderer = require.resolve("vue/server-renderer");
const fixtures = path.resolve(import.meta.dirname, "../../../../tests/_fixtures/tsrx");
const counter = fs.readFileSync(path.join(fixtures, "mount/counter.tsrx"), "utf8");
const rendered =
  '<section><h1>CLICKS</h1><span class="label">clicks: 0</span><button>add</button><p class="none">none</p></section>';

function project(files: Record<string, string>): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-unplugin-tsrx-"));
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), source);
  }
  return root;
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

const counterFiles = {
  "Counter.tsrx": counter,
  "entry.js": 'export { Counter } from "./Counter.tsrx";\n',
};

void test("rollup: a .tsrx module bundles as JavaScript and renders", async (t) => {
  const root = project(counterFiles);
  try {
    const bundle = await rollup({
      input: path.join(root, "entry.js"),
      external: [/^vue/],
      plugins: [vizeRollup({ isProduction: true })],
    });
    const output = await bundle.generate({ format: "esm" });
    const chunk = output.output.find((item) => item.type === "chunk");
    t.assert.ok(chunk && chunk.type === "chunk");
    t.assert.doesNotMatch(chunk.code, /@\{|@if|: \{ title: string/);
    t.assert.equal(await renderCounter(root, chunk.code), rendered);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("esbuild: a .tsrx module bundles as JavaScript and renders", async (t) => {
  const root = project(counterFiles);
  try {
    const result = await build({
      entryPoints: [path.join(root, "entry.js")],
      bundle: true,
      write: false,
      format: "esm",
      outdir: path.join(root, "out"),
      external: ["vue"],
      plugins: [vizeEsbuild({ isProduction: true })],
    });
    const [js] = result.outputFiles;
    t.assert.ok(js);
    t.assert.equal(await renderCounter(root, js.text), rendered);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("rollup: a refused .tsrx module fails with every diagnostic", async (t) => {
  const root = project({
    "App.tsrx": "export const app = () => <p>hi</p>;\n",
    "entry.js": 'export { app } from "./App.tsrx";\n',
  });
  try {
    await t.assert.rejects(
      rollup({
        input: path.join(root, "entry.js"),
        plugins: [vizeRollup({ isProduction: true })],
      }),
      /\[vize\] TSRX compilation failed in .*App\.tsrx:\n {2}- .*App\.tsrx:1:26 \[unsupported\] OutsideComponent is not emitted for the DOM target yet/,
    );
    await t.assert.rejects(
      rollup({
        input: path.join(root, "entry.js"),
        plugins: [vizeRollup({ isProduction: true, ssr: true })],
      }),
      /App\.tsrx:1:1 \[unsupported\] Ssr emission of TSRX modules is not supported yet/,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("rollup: an excluded .tsrx module is left to Rollup", async (t) => {
  const root = project(counterFiles);
  try {
    await t.assert.rejects(
      rollup({
        input: path.join(root, "entry.js"),
        external: [/^vue/],
        plugins: [vizeRollup({ isProduction: true, exclude: /Counter\.tsrx$/ })],
      }),
      (error: Error) => {
        t.assert.doesNotMatch(error.message, /\[vize\]/);
        return true;
      },
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
