// Execute unchanged CLI-produced JS modules in the real Vite/Vue consumer graph.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { asArray, asRecord, asString, at } from "../../json-value.ts";
import {
  importChecked,
  importDefault,
  loadChromium,
  previewPort,
  viteRuntimeFunctions,
} from "./host-types.ts";
import type { PreviewServer, ViteRuntimeHost } from "./host-types.ts";
import { browserOutput } from "./javascript-workspace-runtime.ts";
import { save, sha } from "./javascript-workspace-project.ts";
import type { WorkspaceContext } from "./javascript-workspace-project.ts";

export async function cliViteRuntime(root: string, context: WorkspaceContext, compiled: unknown) {
  assert.equal(context.cohort.id, "vite");
  const application = path.join(context.project, "apps/vite");
  const api = await importChecked<ViteRuntimeHost>(
    path.join(application, "workspace-runtime.mjs"),
    viteRuntimeFunctions,
  );
  const browser = await loadChromium(root).launch();
  const rows: Record<string, unknown>[] = [];
  try {
    for (const value of asArray(compiled, "compiled packets")) {
      const entry = asRecord(value, "compiled entry");
      const backend = asString(entry.backend, "entry.backend");
      const directory = path.join(context.artifacts, `cli-runtime-${backend}`);
      fs.mkdirSync(directory);
      const modules: Record<string, string> = {};
      for (const item of asArray(entry.packets, "entry.packets")) {
        const packet = asRecord(item, "packet");
        const filename = asString(packet.filename, "packet.filename");
        const code = asString(packet.code, "packet.code");
        const file = path.join(directory, filename + ".mjs");
        fs.writeFileSync(file, code);
        assert.equal(sha(fs.readFileSync(file)), sha(Buffer.from(code)));
        modules[filename] = file;
      }
      const app = modules["App.vue"];
      const badge = modules["BadgeCard.vue"];
      assert.ok(app && badge, "both CLI packets were compiled");
      fs.symlinkSync(
        path.join(context.project, "node_modules"),
        path.join(directory, "node_modules"),
      );
      const host = path.join(directory, "entry.mjs");
      fs.writeFileSync(
        host,
        backend === "ssr"
          ? `export { default } from ${JSON.stringify(app)};\n`
          : `import { createApp } from "vue";\nimport App from ${JSON.stringify(app)};\ncreateApp(App).mount("#app");\n`,
      );
      const common = {
        root: directory,
        configFile: false,
        logLevel: "warn",
        resolve: { alias: { "@workspace/ui/BadgeCard.vue": badge } },
      };
      let server: PreviewServer | undefined;
      try {
        if (backend === "ssr") {
          const result = await api.build({
            ...common,
            build: {
              minify: false,
              outDir: path.join(directory, "dist"),
              ssr: host,
              rollupOptions: {
                external: ["vue", "vue/server-renderer", "@vue/server-renderer"],
                output: { entryFileNames: "entry.mjs", chunkFileNames: "[name]-[hash].mjs" },
              },
            },
          });
          save(directory, "bundle.json", at(result, "output"));
          fs.symlinkSync(
            path.join(context.project, "node_modules"),
            path.join(directory, "dist/node_modules"),
          );
          const component = await importDefault(path.join(directory, "dist/entry.mjs"));
          const html = await api.renderToString(api.createSSRApp(component));
          fs.writeFileSync(path.join(directory, "rendered.html"), html);
          assert.equal(html, context.corpus.expected.Vite.initial);
          rows.push({ backend, html });
        } else {
          fs.writeFileSync(
            path.join(directory, "index.html"),
            '<!doctype html>\n<div id="app"></div>\n<script type="module" src="./entry.mjs"></script>\n',
          );
          const result = await api.build({
            ...common,
            build: { minify: false, outDir: path.join(directory, "dist") },
          });
          save(directory, "bundle.json", at(result, "output"));
          server = await api.preview({
            ...common,
            build: { outDir: path.join(directory, "dist") },
            preview: { host: "127.0.0.1", port: 0 },
          });
          rows.push({
            backend,
            ...(await browserOutput(
              browser,
              `http://127.0.0.1:${previewPort(server)}/`,
              context.corpus.expected.Vite,
              directory,
            )),
          });
        }
      } finally {
        await server?.close();
        for (const filename of [
          path.join(directory, "node_modules"),
          path.join(directory, "dist/node_modules"),
        ])
          if (fs.existsSync(filename)) fs.unlinkSync(filename);
      }
    }
  } finally {
    await browser.close();
  }
  save(context.artifacts, "cli-executable-runtime.json", rows);
  assert.equal(rows.length, 2);
}
