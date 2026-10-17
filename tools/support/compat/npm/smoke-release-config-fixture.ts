import assert from "node:assert/strict";

/** Whole inputs and authored output oracles for the installed #8371 workspace cell. */
export const CONFIG_PACKAGE_NAMES = ["app", "control"] as const;

export const CONFIG_SOURCES = {
  "BuildPanel.vue":
    "<template>\n  <div>\n    <p>  two  spaces  </p>\n    <my-widget />\n  </div>\n</template>\n",
  "ImagePanel.vue": '<template>\n  <img src="fixture.png" />\n</template>\n',
  "QuotesPanel.vue":
    '<script setup>const message="hello";</script>\n<template><p>{{ message }}</p></template>\n',
  "TypedPanel.vue":
    '<script setup lang="ts">\nconst value: string = null;\n</script>\n<template><p>{{ value }}</p></template>\n',
};

// Both dependencies still exist; these are syntax errors, not missing-file aliases.
export const MALFORMED_CONFIG_DEPENDENCIES = {
  "vue-settings.ts": "export default { formatter: { singleQuote: true },\n",
  "tsconfig.base.json": '{"compilerOptions":{"strictNullChecks":\n',
} as const;

// Whole authored inputs/packets from editors/vim/test/config_project_spec.vim.
export const REFERENCE_SOURCE = '<template>\n<div :title.sync="title" />\n</template>\n';
export const REFERENCE_DIRTY_SOURCE = REFERENCE_SOURCE.replace('"title"', '"draft"');
export const REFERENCE_POLICY =
  'export default { languageServer: { lint:true, typecheck:false, ecosystem:false, hover:false, formatting:false }, linter:{ preset:"incremental", rules:{ "vue/no-deprecated-v-bind-sync":"error" } } };';
export const REFERENCE_DIAGNOSTICS = [
  {
    code: "vue/no-deprecated-v-bind-sync",
    codeDescription: { href: "https://eslint.vuejs.org/rules/no-deprecated-v-bind-sync.html" },
    message:
      'the .sync modifier on v-bind was removed in Vue 3; use v-model:<prop> instead\n\nHelp: Vue 3 removed the .sync modifier and folded its two-way binding into argumented v-model. Replace it with v-model:<prop> (e.g. :title.sync="title" -> v-model:title="title").',
    range: { start: { line: 1, character: 5 }, end: { line: 1, character: 24 } },
    severity: 1,
    source: "vize/lint",
  },
];
export const REFERENCE_FORMATTED = [
  {
    newText: '<template>\n  <div :title.sync="draft" />\n</template>\n',
    range: { start: { line: 0, character: 0 }, end: { line: 3, character: 0 } },
  },
];

export function referenceWorkspaceFiles(): Record<string, string> {
  return {
    "vite.config.ts": 'import vue from "./vue-policy.mjs"; export default { vue };\n',
    "vue-policy.mjs": `${REFERENCE_POLICY}\n`,
    "tsconfig.json":
      '{"files":[],"references":[{"path":"./tsconfig.legacy.json"},{"path":"./tsconfig.modern.json"}]}\n',
    "tsconfig.legacy.json":
      '{"include":["src/legacy/**/*.vue"],"vueCompilerOptions":{"target":2.7}}\n',
    "tsconfig.modern.json":
      '{"include":["src/modern/**/*.vue"],"vueCompilerOptions":{"target":3.5}}\n',
    "src/legacy/App.vue": REFERENCE_SOURCE,
    "src/modern/App.vue": REFERENCE_SOURCE,
  };
}

export const SINGLE_FORMATTED =
  "<script setup>\nconst message = 'hello';\n</script>\n\n<template><p>{{ message }}</p></template>\n";
export const DOUBLE_FORMATTED =
  '<script setup>\nconst message = "hello";\n</script>\n\n<template>\n  <p>{{ message }}</p>\n</template>\n';

// Complete historical compiler outputs for the identical BuildPanel source,
// retained from project_config_cli__project_settings_build_{vite,defaults}.snap.
export const PRESERVED_BUILD = `import { createElementVNode as _createElementVNode, openBlock as _openBlock, createElementBlock as _createElementBlock, createTextVNode as _createTextVNode } from "vue"

export function render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("div", null, [
    _createElementVNode("p", null, "  two  spaces  "),
    _createTextVNode(),
    _createElementVNode("my-widget")
  ]))
}
`;
export const DEFAULT_BUILD = `import { resolveComponent as _resolveComponent, createElementVNode as _createElementVNode, createVNode as _createVNode, openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"

export function render(_ctx, _cache) {
  const _component_my_widget = _resolveComponent("my-widget")
  
  return (_openBlock(), _createElementBlock("div", null, [
    _createElementVNode("p", null, " two spaces "),
    _createVNode(_component_my_widget)
  ]))
}
`;

// insta 1.49.0 snapshot.rs::Snapshot::serialize_snapshot adds one framing LF
// after the displayed text. Preserve those complete snapshot bodies above and
// declare only these two file counterparts; never normalize observed CLI files.
export const BUILD_FILE_OUTPUTS = Object.freeze({
  configured: PRESERVED_BUILD.slice(0, -1),
  defaults: DEFAULT_BUILD.slice(0, -1),
});

export function assertBuildFileOutput(actual: string, configured: boolean): void {
  assert.equal(
    actual,
    BUILD_FILE_OUTPUTS[configured ? "configured" : "defaults"],
    "complete configured compiler output",
  );
}

export function vueHelper(configured: boolean): string {
  return `export default ${JSON.stringify({
    compiler: {
      whitespace: configured ? "preserve" : "condense",
      customElements: configured ? ["my-widget"] : [],
    },
    formatter: { singleQuote: configured },
    linter: {
      preset: "essential",
      rules: {
        "vue/multi-word-component-names": "off",
        "a11y/alt-text": configured ? "error" : "off",
      },
    },
    languageServer: { typecheck: true, lint: false, formatting: true },
  })};\n`;
}

export function tsconfigBase(strictNullChecks: boolean): string {
  return `${JSON.stringify({ compilerOptions: { strictNullChecks } }, null, 2)}\n`;
}

export const CANONICAL_VITE = [
  'import { defineConfig } from "vite";',
  'import vize from "@vizejs/vite-plugin";',
  'import vue from "./vue-settings";',
  "export default defineConfig({ plugins: [vize()], vue });",
  "",
].join("\n");

export function typeDiagnostics(broken: boolean) {
  return broken
    ? [
        {
          range: { start: { line: 1, character: 6 }, end: { line: 1, character: 11 } },
          severity: 1,
          code: 2322,
          source: "vize/types",
          message: "Type 'null' is not assignable to type 'string'.",
        },
      ]
    : [];
}

export function lintReport(file: string, configured: boolean) {
  return [
    {
      file,
      messages: configured
        ? [
            {
              ruleId: "a11y/alt-text",
              ruleDocsPath: "docs/content/rules/accessibility.md",
              severity: 2,
              message: "[vize:a11y/alt-text] <img> elements must have an alt attribute",
              help: 'Add an alt attribute: <img alt="Description"> or <img alt=""> for decorative images',
              line: 2,
              column: 3,
              endLine: 2,
              endColumn: 28,
            },
          ]
        : [],
      errorCount: configured ? 1 : 0,
      warningCount: 0,
    },
  ];
}

export interface Message {
  jsonrpc?: string;
  method?: string;
  params?: unknown;
  result?: unknown;
  error?: unknown;
}

export function assertLatestPublication(messages: Message[], expected: Message): void {
  const expectedUri = (expected.params as { uri: string }).uri;
  const latest = messages.findLast(
    (message) =>
      message.method === "textDocument/publishDiagnostics" &&
      (message.params as { uri?: string } | undefined)?.uri === expectedUri,
  );
  assert.deepEqual(latest, expected, "latest complete publication after the request barrier");
}

/** Exact config_reload.rs::reload_error contract; preserve sent URI spelling. */
export function expectedConfigReloadError(rootUri: string, changedUris: readonly string[]) {
  const changed = [...new Set(changedUris)].sort().join(", ");
  return {
    jsonrpc: "2.0",
    method: "window/showMessage",
    params: {
      type: 1,
      message: `Vize could not reload project configuration for ${rootUri} after changes to ${changed}. Keeping the last good configuration.`,
    },
  };
}
