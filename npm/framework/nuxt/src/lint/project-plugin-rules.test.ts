// `vue.plugins` lint rules in the Nuxt Oxlint integration (#13074): hosted by
// oxlint-plugin-vize, never projected as Oxlint rules, and fail closed for
// capabilities lint, or another product consuming them, does not implement.
import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { TestContext } from "node:test";

import { renderNuxtOxlintConfig } from "./emitter.ts";
import { readProjectLintRules, readProjectLintRuleSets } from "./project-rules.ts";

async function temporaryRoot(t: TestContext, prefix: string): Promise<string> {
  const root = await mkdtemp(path.join(os.tmpdir(), prefix));
  t.after(() => rm(root, { force: true, recursive: true }));
  return root;
}

async function unexpectedPreset(): Promise<never> {
  throw new Error("no preset expected");
}

const PLUGIN_SDK = JSON.stringify(new URL("../../../../plugin-sdk/v2.js", import.meta.url).href);
const DESIGN_PLUGIN = `import { defineVizePlugin } from ${PLUGIN_SDK};
const design = defineVizePlugin({
  apiVersion: 2, stability: "experimental", name: "@acme/design", version: "1.0.0",
  setup(api) {
    api.lint.rule({ id: "no-raw-color", meta: { messages: { raw: "raw" } }, visit: ["ui.bind"], run() {} });
  },
});`;

void test("plugin lint rules run in oxlint-plugin-vize instead of as Oxlint rules", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-plugin-rules-");
  await writeFile(
    path.join(root, "vite.config.mjs"),
    `${DESIGN_PLUGIN}
export default { vue: { plugins: [design()], linter: { rules: { "@acme/design/no-raw-color": "error", "vue/no-v-html": "warn" } } } };`,
  );
  assert.deepEqual(await readProjectLintRuleSets(root, unexpectedPreset), {
    pluginRules: true,
    vize: { "vue/no-v-html": "warn" },
  });

  const scoped = await temporaryRoot(t, "vize-nuxt-project-plugin-entries-");
  await writeFile(
    path.join(scoped, "vite.config.mjs"),
    `${DESIGN_PLUGIN}
export default { vue: { plugins: [design()], entries: [{ files: ["src/**"], linter: { rules: { "@acme/design/no-raw-color": "off" } } }] } };`,
  );
  await assert.rejects(readProjectLintRuleSets(scoped, unexpectedPreset), {
    message:
      "[plugin/plan-invalid] vue.entries[0].linter: plugin rule `@acme/design/no-raw-color` is configured in vue.linter.rules only",
  });

  const block = await temporaryRoot(t, "vize-nuxt-project-plugin-block-");
  await writeFile(
    path.join(block, "vite.config.mjs"),
    `import { defineVizePlugin } from ${PLUGIN_SDK};
const routes = defineVizePlugin({
  apiVersion: 2, stability: "experimental", name: "@acme/routes", version: "1.0.0",
  setup(api) { api.sfc.block("route", { lang: "json" }); },
});
export default { vue: { plugins: [routes()] } };`,
  );
  await assert.rejects(readProjectLintRuleSets(block, unexpectedPreset), {
    message:
      "[plugin/capability-unadmitted] @acme/routes: lint does not implement sfc.custom-block `route:json` yet",
  });

  // Lint checks directive contracts, but the compiler and type checker do not yet.
  const directive = await temporaryRoot(t, "vize-nuxt-project-plugin-directive-");
  await writeFile(
    path.join(directive, "vite.config.mjs"),
    `import { defineVizePlugin } from ${PLUGIN_SDK};
const track = defineVizePlugin({
  apiVersion: 2, stability: "experimental", name: "@acme/track", version: "1.0.0",
  setup(api) { api.vue.directive("track"); },
});
export default { vue: { plugins: [track()] } };`,
  );
  await assert.rejects(readProjectLintRuleSets(directive, unexpectedPreset), {
    message:
      "[plugin/capability-unadmitted] @acme/track: vue.directive `track` is not implemented by compiler, typecheck yet; run `vize plugin inspect` for the plan",
  });

  const skipped = await temporaryRoot(t, "vize-nuxt-project-plugin-skipped-");
  await writeFile(
    path.join(skipped, "vite.config.mjs"),
    `export default { vue: { plugins: [false, null], linter: { rules: { "vue/no-v-html": "error" } } } };`,
  );
  assert.deepEqual(await readProjectLintRules(skipped, unexpectedPreset), {
    "vue/no-v-html": "error",
  });
});

void test("the generated config enables both plugin rule hosts", () => {
  const config = JSON.parse(
    renderNuxtOxlintConfig(
      [
        {
          name: "project/plugins",
          nativeRules: { "vize/plugin-errors": "error", "vize/plugin-warnings": "warn" },
        },
      ],
      "oxlint-plugin-vize",
      undefined,
      [],
      true,
    ),
  ) as { rules: Record<string, string>; settings: { vize: Record<string, unknown> } };
  assert.deepEqual(config.settings.vize, { preset: "incremental", pluginRules: true });
  assert.deepEqual(config.rules, { "vize/plugin-errors": "error", "vize/plugin-warnings": "warn" });
});
