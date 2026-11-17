// `.tsrx` modules through Rspack's dist TSRX loader compile with the native TSRX
// entry (#13140): the loaded module is the entry's output, the bundle renders,
// and a refused module fails the build with every diagnostic.
import { test } from "node:test";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { rspack } from "@rspack/core";
import { compileTsrx } from "@vizejs/native";
import "./test/setup.ts";
import { packageLoaderAliases, workspaceRoot } from "./test/helpers.ts";

const require = createRequire(import.meta.url);
const vueRuntime = require.resolve("vue");
const serverRenderer = require.resolve("vue/server-renderer");
const counter = fs.readFileSync(
  path.join(workspaceRoot, "tests", "_fixtures", "tsrx", "mount", "counter.tsrx"),
  "utf8",
);

function project(files: Record<string, string>): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-rspack-tsrx-"));
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), source);
  }
  return root;
}

function compile(root: string) {
  const compiler = rspack({
    mode: "development",
    devtool: false,
    context: root,
    entry: { main: path.join(root, "entry.js") },
    output: {
      path: path.join(root, "dist"),
      filename: "bundle.mjs",
      module: true,
      library: { type: "module" },
      chunkFormat: "module",
    },
    experiments: { outputModule: true },
    externalsType: "module",
    externals: { vue: pathToFileURL(vueRuntime).href },
    infrastructureLogging: { level: "error" },
    resolveLoader: { alias: packageLoaderAliases },
    module: {
      rules: [
        {
          test: /\.tsrx$/,
          use: [
            {
              loader: "builtin:swc-loader",
              options: { jsc: { parser: { syntax: "typescript" } } },
            },
            { loader: "@vizejs/rspack-plugin/tsrx-loader" },
          ],
        },
      ],
    },
  });
  return new Promise<NonNullable<Parameters<Parameters<typeof compiler.run>[0]>[1]>>(
    (resolve, reject) => {
      compiler.run((error, stats) => {
        compiler.close((closeError) => {
          if (error || closeError || !stats) {
            reject(error ?? closeError ?? new Error("Rspack did not return stats"));
            return;
          }
          resolve(stats);
        });
      });
    },
  );
}

void test("rspack: a .tsrx module loads as the native entry's output and renders", async (t) => {
  const root = project({
    "Counter.tsrx": counter,
    "entry.js": 'export { Counter } from "./Counter.tsrx";\n',
  });
  try {
    const stats = await compile(root);
    t.assert.equal(stats.hasErrors(), false, JSON.stringify(stats.toJson({ errors: true }).errors));
    const file = path.join(root, "Counter.tsrx");
    const native = compileTsrx(counter, { filename: file, target: "dom" });
    t.assert.deepEqual(native.errors, []);
    const loaded = Array.from(stats.compilation.modules).find(
      (module) => "resource" in module && module.resource === file,
    );
    t.assert.ok(loaded, "the .tsrx module is part of the compilation");
    const { Counter } = (await import(
      pathToFileURL(path.join(root, "dist", "bundle.mjs")).href
    )) as { Counter: object };
    const { createSSRApp } = (await import(pathToFileURL(vueRuntime).href)) as typeof import("vue");
    const { renderToString } = (await import(pathToFileURL(serverRenderer).href)) as {
      renderToString: (app: unknown) => Promise<string>;
    };
    t.assert.equal(
      await renderToString(createSSRApp(Counter, { title: "clicks", step: 1 })),
      '<section><h1>CLICKS</h1><span class="label">clicks: 0</span><button>add</button><p class="none">none</p></section>',
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

void test("rspack: a refused .tsrx module fails the build with every diagnostic", async (t) => {
  const root = project({
    "App.tsrx": "export const app = () => <p>hi</p>;\n",
    "entry.js": 'export { app } from "./App.tsrx";\n',
  });
  try {
    const stats = await compile(root);
    const errors = stats.toJson({ all: false, errors: true }).errors ?? [];
    t.assert.equal(errors.length, 1);
    t.assert.match(
      errors[0]?.message ?? "",
      /\[vize\] TSRX compilation failed in .*App\.tsrx:\n {2}- .*App\.tsrx:1:26 \[unsupported\] OutsideComponent is not emitted for the DOM target yet/,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
