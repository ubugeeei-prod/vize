import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { root } from "./support/lsp/paths.ts";
import { checkVersions, core, renderer } from "./support/native-sfc-ssr-reference.ts";
import { loadSetupModule, stockSetupSfc } from "./support/native-sfc-setup-ssr-loader.ts";

const directory = path.join(root, "tests/_fixtures/differential/linter/ssr-script-setup-7982");
const original = fs.readFileSync(path.join(directory, "WidthLabel.vue.txt"), "utf8");
const sources = [
  { id: "whole-original-issue", source: original, expectedWindowErrors: 1 },
  {
    id: "server-prefetch",
    source:
      '<script setup lang="ts">\nimport { onServerPrefetch } from "vue";\nonServerPrefetch(() => { window.innerWidth; });\n</script>\n<template><p>server</p></template>\n',
    expectedWindowErrors: 1,
  },
  {
    id: "mounted-and-event-deferred",
    source:
      '<script setup lang="ts">\nimport { ref, onMounted } from "vue";\nconst width = ref(0);\nonMounted(() => { width.value = window.innerWidth; });\nconst click = () => window.alert("clicked");\n</script>\n<template><p>{{ width }}</p><button @click="click">Go</button></template>\n',
    expectedWindowErrors: 0,
    expectedHtml: "<!--[--><p>0</p><button>Go</button><!--]-->",
  },
];

await test("actual pinned Vue SSR executes setup/server prefetch and defers mounted/event callbacks", async () => {
  checkVersions();
  assert.equal(Object.hasOwn(globalThis, "window"), false, "this is a real server environment");
  const rows: Array<Record<string, unknown>> = [];
  const record = {
    schema: "vize.ssr.script-execution-7982",
    provider: { vue: "3.5.35", plugin: "6.0.7" },
    rows,
    status: "PENDING",
    error: null as string | null,
    productLintCredit: false,
    nativeCompilerCredit: false,
  };
  const output = path.join(root, "target/differential/lint-ssr-script-runtime-7982.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const save = () => fs.writeFileSync(output, JSON.stringify(record, null, 2) + "\n");
  try {
    for (const row of sources) {
      const filename = `/native-ssr/${row.id}.vue`;
      const graph = await stockSetupSfc(row.source, filename, true);
      const component = await loadSetupModule(graph);
      const errors: Array<{ name: string; message: string; info: string }> = [];
      const warnings: string[] = [];
      const context: { modules?: Set<string> } = {};
      const app = core.createSSRApp(component);
      app.config.errorHandler = (error: unknown, _instance: unknown, info: string) => {
        assert(error instanceof Error);
        errors.push({ name: error.name, message: error.message, info });
      };
      app.config.warnHandler = (warning: string) => warnings.push(warning);
      let html: string | null = null,
        thrown: { name: string; message: string } | null = null;
      try {
        html = await renderer.renderToString(app, context);
      } catch (error) {
        assert(error instanceof Error);
        thrown = { name: error.name, message: error.message };
      }
      rows.push({
        ...row,
        filename,
        graph,
        errors,
        warnings,
        html,
        thrown,
        modules: [...(context.modules ?? [])],
      });
      save();
      assert.equal(
        errors.filter(
          (error) => error.name === "ReferenceError" && error.message === "window is not defined",
        ).length,
        row.expectedWindowErrors,
        row.id,
      );
      if (row.expectedWindowErrors === 0) {
        assert.deepEqual(errors, []);
        assert.deepEqual(warnings, []);
        assert.equal(thrown, null);
        assert.equal(html, row.expectedHtml);
      }
    }
    record.status = "PASS";
  } catch (error) {
    record.status = "FAIL";
    record.error = error instanceof Error ? error.message : String(error);
    throw error;
  } finally {
    save();
  }
});
