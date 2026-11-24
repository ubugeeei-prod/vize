/**
 * The native compile calls real Vite module graphs make (#13559, #13565), at a
 * small size: a build with a client, an SSR and a component-free environment,
 * and a dev server's first load in the client and SSR environments.
 */

// Must stay the first import: it wraps the native compile functions the plugin captures.
import {
  labelledComponent,
  scratchProject,
  takeNativeCompileCalls,
  type NativeCompileCall,
} from "../test/native-compile-calls.ts";

import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";
import { createBuilder, createServer, type Plugin } from "vite";

import { linkWorkspaceVue } from "../test/startup-scan-project.ts";
import { vize } from "./index.ts";

const require = createRequire(import.meta.url);
const imported = 6;
const unused = 4;
const components = Array.from({ length: imported }, (_, i) => `src/components/Item${i}.vue`);

/** The reporter's project shape: every environment's entry, and components nothing imports. */
function reproduction() {
  const files: Record<string, string> = {
    "src/app.ts": `${components
      .map((file, i) => `import Item${i} from "./${file.slice("src/".length)}";`)
      .join("\n")}\n\nexport const items = [${components.map((_, i) => `Item${i}`).join(", ")}];\n`,
    "src/main.ts": `import { items } from "./app";\n\nconsole.log(items);\n`,
    "src/entry-server.ts": `import { items } from "./app";\n\nexport const render = () => items;\n`,
    // The third environment imports no component, as Nuxt's Nitro pass imports none.
    "src/worker.ts": `export const handler = (): string => "ok";\n`,
  };
  for (const file of components) files[file] = labelledComponent(file);
  for (let i = 0; i < unused; i++) {
    files[`src/components/Unused${i}.vue`] = labelledComponent(`unused ${i}`);
  }
  return scratchProject(files);
}

void test("a build compiles what each environment's graph imports, in one batch", async () => {
  const project = reproduction();
  try {
    linkWorkspaceVue(project.root);
    const calls: Record<string, NativeCompileCall[]> = {};
    const outputs: Record<string, string> = {};
    // Environments build one after another, so the calls between two closes belong to one.
    const observe: Plugin = {
      name: "test:compile-calls-by-environment",
      generateBundle(_, bundle) {
        outputs[this.environment.name] = Object.values(bundle)
          .map((chunk) => (chunk.type === "chunk" ? chunk.code : ""))
          .join("\n");
      },
      closeBundle() {
        // The bundler resolves a module's imports concurrently, so the order
        // inside a batch is its own; the batches and their files are not.
        calls[this.environment.name] = takeNativeCompileCalls(project.root).map((call) => ({
          ...call,
          files: [...call.files].sort(),
        }));
      },
    };
    const entry = (input: string) => ({ build: { rollupOptions: { input } } });
    const builder = await createBuilder({
      root: project.root,
      configFile: false,
      logLevel: "silent",
      plugins: [...vize({ configMode: false }), observe],
      build: { write: false, minify: false },
      builder: {},
      environments: {
        client: entry("src/main.ts"),
        ssr: entry("src/entry-server.ts"),
        worker: { consumer: "server", ...entry("src/worker.ts") },
      },
    });
    await builder.buildApp();

    assert.deepEqual(calls, {
      client: [{ api: "batch", ssr: false, files: components }],
      ssr: [{ api: "batch", ssr: true, files: components }],
      worker: [],
    });
    for (const environment of ["client", "ssr"]) {
      assert.deepEqual(
        components.map((file) => outputs[environment].includes(`"${file}"`)),
        components.map(() => true),
        `${environment} renders every imported component`,
      );
      assert.equal(outputs[environment].includes("unused"), false);
    }
  } finally {
    project.cleanup();
  }
});

void test("a dev server compiles nothing at start, and a first load in one batch per environment", async () => {
  const project = reproduction();
  const server = await createServer({
    root: project.root,
    configFile: false,
    logLevel: "silent",
    plugins: vize({ configMode: false }),
    resolve: {
      alias: [
        { find: /^vue$/u, replacement: require.resolve("vue") },
        { find: /^vue\/server-renderer$/u, replacement: require.resolve("vue/server-renderer") },
      ],
    },
    optimizeDeps: { noDiscovery: true, include: [] },
    server: { middlewareMode: true, watch: null, ws: false },
  });
  try {
    const urls = components.map((file) => `/${file}`);
    const load = async (name: "client" | "ssr", entry: string) => {
      const environment = server.environments[name];
      // As a page load does: the entry, then the module that imports every component.
      await environment.transformRequest(entry);
      await environment.transformRequest("/src/app.ts");
      return Promise.all(urls.map((url) => environment.transformRequest(url)));
    };
    assert.deepEqual(takeNativeCompileCalls(project.root), [], "the server starts uncompiled");

    const client = await load("client", "/src/main.ts");
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: false, files: components },
    ]);
    const ssr = await load("ssr", "/src/entry-server.ts");
    assert.deepEqual(takeNativeCompileCalls(project.root), [
      { api: "batch", ssr: true, files: components },
    ]);
    for (const results of [client, ssr]) {
      assert.deepEqual(
        results.map((result, i) => result?.code.includes(`"${components[i]}"`) ?? false),
        components.map(() => true),
      );
    }
    assert.notEqual(client[0]?.code, ssr[0]?.code);
  } finally {
    await server.close();
    project.cleanup();
  }
});
