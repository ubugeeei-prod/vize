// Execute unchanged CLI-produced JS modules in the real Vite/Vue consumer graph.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";
import { browserOutput } from "./javascript-workspace-runtime.mjs";
import { save, sha } from "./javascript-workspace-project.mjs";

export async function cliViteRuntime(root, context, compiled) {
  assert.equal(context.cohort.id, "vite");
  const application = path.join(context.project, "apps/vite");
  const api = await import(pathToFileURL(path.join(application, "workspace-runtime.mjs")).href);
  const { chromium } = createRequire(path.join(root, "tests/package.json"))("@playwright/test");
  const browser = await chromium.launch();
  const rows = [];
  try {
    for (const entry of compiled) {
      const directory = path.join(context.artifacts, `cli-runtime-${entry.backend}`);
      fs.mkdirSync(directory);
      const modules = {};
      for (const packet of entry.packets) {
        const file = path.join(directory, packet.filename + ".mjs");
        fs.writeFileSync(file, packet.code);
        assert.equal(sha(fs.readFileSync(file)), sha(Buffer.from(packet.code)));
        modules[packet.filename] = file;
      }
      fs.symlinkSync(
        path.join(context.project, "node_modules"),
        path.join(directory, "node_modules"),
      );
      const host = path.join(directory, "entry.mjs");
      fs.writeFileSync(
        host,
        entry.backend === "ssr"
          ? `export { default } from ${JSON.stringify(modules["App.vue"])};\n`
          : `import { createApp } from "vue";\nimport App from ${JSON.stringify(modules["App.vue"])};\ncreateApp(App).mount("#app");\n`,
      );
      const common = {
        root: directory,
        configFile: false,
        logLevel: "warn",
        resolve: { alias: { "@workspace/ui/BadgeCard.vue": modules["BadgeCard.vue"] } },
      };
      let server;
      try {
        if (entry.backend === "ssr") {
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
          save(directory, "bundle.json", result.output);
          fs.symlinkSync(
            path.join(context.project, "node_modules"),
            path.join(directory, "dist/node_modules"),
          );
          const { default: component } = await import(
            pathToFileURL(path.join(directory, "dist/entry.mjs")).href
          );
          const html = await api.renderToString(api.createSSRApp(component));
          fs.writeFileSync(path.join(directory, "rendered.html"), html);
          assert.equal(html, context.corpus.expected.Vite.initial);
          rows.push({ backend: entry.backend, html });
        } else {
          fs.writeFileSync(
            path.join(directory, "index.html"),
            '<!doctype html>\n<div id="app"></div>\n<script type="module" src="./entry.mjs"></script>\n',
          );
          const result = await api.build({
            ...common,
            build: { minify: false, outDir: path.join(directory, "dist") },
          });
          save(directory, "bundle.json", result.output);
          server = await api.preview({
            ...common,
            build: { outDir: path.join(directory, "dist") },
            preview: { host: "127.0.0.1", port: 0 },
          });
          const address = server.httpServer.address();
          rows.push({
            backend: entry.backend,
            ...(await browserOutput(
              browser,
              `http://127.0.0.1:${address.port}/`,
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
