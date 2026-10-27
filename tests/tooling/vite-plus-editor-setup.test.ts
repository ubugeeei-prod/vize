import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../", import.meta.url));
const plugin = path.join(root, "npm/builder/vite");
const require = createRequire(path.join(plugin, "package.json"));
const { parse: parseJsonc } = require("jsonc-parser");
const vpPackage = require.resolve("vite-plus/package.json");
const vp = path.join(path.dirname(vpPackage), "bin/vp");

function run(cwd: string, args: string[], expected = 0) {
  const result = spawnSync(process.execPath, [vp, ...args], {
    cwd,
    encoding: "utf8",
    timeout: 120_000,
    env: { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" },
  });
  assert.equal(result.error, undefined);
  const output = result.stdout + result.stderr;
  assert.equal(result.status, expected, output);
  return output;
}

void test("packed editor:setup preserves project policy, explicit ownership and whole JSONC comments", () => {
  for (const directory of ["npm/cli", "npm/builder/unplugin", "npm/builder/vite"]) {
    run(path.join(root, directory), ["pack"]);
  }
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-editor-policy-10932-"));
  try {
    fs.mkdirSync(path.join(directory, "node_modules/@vizejs"), { recursive: true });
    for (const [name, source] of [
      ["vite-plus", path.dirname(vpPackage)],
      ["@vizejs/vite-plugin", plugin],
    ]) {
      fs.symlinkSync(
        source,
        path.join(directory, "node_modules", name),
        process.platform === "win32" ? "junction" : "dir",
      );
    }
    const write = (name: string, source: string) =>
      fs.writeFileSync(path.join(directory, name), source);
    write("package.json", '{"name":"vize-editor-policy","type":"module","private":true}\n');
    write(
      "vite.config.ts",
      'import { defineConfig } from "@vizejs/vite-plugin/vite-plus";\nexport default defineConfig({}, { plugin: false, conflicts: false });\n',
    );
    fs.mkdirSync(path.join(directory, ".vscode"));
    write(".vscode/settings.json", "{}\n");
    write(".vscode/extensions.json", "{}\n");
    checkEditorSetupPolicy(directory, write);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

function checkEditorSetupPolicy(directory: string, write: (name: string, value: string) => void) {
  const originalConfig = fs.readFileSync(path.join(directory, "vite.config.ts"), "utf8");
  const originalSettings = fs.readFileSync(path.join(directory, ".vscode/settings.json"), "utf8");
  const originalExtensions = fs.readFileSync(
    path.join(directory, ".vscode/extensions.json"),
    "utf8",
  );
  const features = [
    "enabled",
    "lint",
    "diagnostics",
    "typecheck",
    "editor",
    "ecosystem",
    "crossFile",
    "legacyVue2",
    "completion",
    "signatureHelp",
    "hover",
    "definition",
    "references",
    "documentSymbols",
    "workspaceSymbols",
    "formatting",
    "codeActions",
    "rename",
    "codeLens",
    "semanticTokens",
    "documentLinks",
    "foldingRanges",
    "inlayHints",
    "fileRename",
    "corsa",
    "tsgo",
  ];
  const allFeatures = Object.fromEntries(features.map((name) => [name, name === "enabled"]));
  const extensions = `// workspace extension policy
{
  "recommendations": [
    // keep the test explorer
    "Vitest.explorer", /* inline rationale */
    "Vue.volar", // preserve the companion language server
  ],
  "unwantedRecommendations": ["other.extension"],
}
`;
  const settings = '// workspace settings\n{ "editor.tabSize": 4, }\n';
  const explicit = {
    "editor.tabSize": 4,
    "vize.enable": false,
    ...Object.fromEntries(
      features
        .filter((name) => name !== "enabled")
        .map((name, index) => [`vize.${name}.enable`, index % 2 === 0]),
    ),
    "[vue]": { "editor.defaultFormatter": "Vue.volar", "editor.formatOnSave": false },
  };
  const cases = [
    { name: "default", vue: {}, formatting: true },
    {
      name: "Volar coexistence",
      vue: { languageServer: { lint: true, formatting: true, typecheck: false, editor: false } },
      formatting: true,
    },
    {
      name: "formatting disabled",
      vue: { languageServer: { formatting: false } },
      formatting: false,
    },
    {
      name: "server disabled",
      vue: { languageServer: { enabled: false, formatting: true } },
      formatting: false,
    },
    { name: "every feature", vue: { languageServer: allFeatures }, formatting: false },
    {
      name: "async withVue",
      vue: { languageServer: { formatting: false, typecheck: false, hover: false } },
      formatting: false,
      async: true,
    },
    {
      name: "explicit workspace",
      vue: { languageServer: allFeatures },
      formatting: false,
      explicit: true,
    },
  ];
  try {
    for (const entry of cases) {
      const base = entry.async ? "async () => { await Promise.resolve(); return {}; }" : "{}";
      const config = `import { defineConfig } from "@vizejs/vite-plugin/vite-plus";\nexport default defineConfig(${base}, { plugin: false, conflicts: false }).withVue(${JSON.stringify(entry.vue)});\n`;
      write("vite.config.ts", config);
      write(".vscode/extensions.json", extensions);
      const settingsSource = entry.explicit
        ? `// explicit workspace decisions\n${JSON.stringify(explicit, null, 2)}\n`
        : settings;
      write(".vscode/settings.json", settingsSource);
      run(directory, ["run", "editor:setup"]);
      const result = fs.readFileSync(path.join(directory, ".vscode/settings.json"), "utf8");
      const recommended = fs.readFileSync(path.join(directory, ".vscode/extensions.json"), "utf8");
      const expected = entry.explicit
        ? explicit
        : {
            "editor.tabSize": 4,
            ...(entry.formatting
              ? {
                  "[vue]": {
                    "editor.defaultFormatter": "ubugeeei.vize",
                    "editor.formatOnSave": true,
                  },
                }
              : {}),
          };
      assert.deepEqual(parseJsonc(result), expected, entry.name);
      assert.ok(
        result.includes(
          entry.explicit ? "// explicit workspace decisions" : "// workspace settings",
        ),
      );
      if (entry.explicit || !entry.formatting) assert.equal(result, settingsSource, entry.name);
      assert.deepEqual(
        parseJsonc(recommended),
        {
          recommendations: [
            "Vitest.explorer",
            "Vue.volar",
            "VoidZero.vite-plus-extension-pack",
            "ubugeeei.vize",
          ],
          unwantedRecommendations: ["other.extension"],
        },
        entry.name,
      );
      for (const comment of [
        "// workspace extension policy",
        "// keep the test explorer",
        "/* inline rationale */",
        "// preserve the companion language server",
      ]) {
        assert.ok(recommended.includes(comment), `${entry.name}: ${comment}`);
      }
      assert.equal(fs.readFileSync(path.join(directory, "vite.config.ts"), "utf8"), config);
      run(directory, ["run", "editor:setup"]);
      assert.equal(
        fs.readFileSync(path.join(directory, ".vscode/settings.json"), "utf8"),
        result,
        entry.name,
      );
      assert.equal(
        fs.readFileSync(path.join(directory, ".vscode/extensions.json"), "utf8"),
        recommended,
        entry.name,
      );
    }
    // `vue.lsp` is rejected by the canonical Vue config boundary today. Setup
    // must preserve both files when that public type/runtime mismatch is hit.
    write(
      "vite.config.ts",
      'import { defineConfig } from "@vizejs/vite-plugin/vite-plus";\nexport default defineConfig({}, { plugin: false, conflicts: false }).withVue({ lsp: { formatting: false } });\n',
    );
    write(".vscode/extensions.json", extensions);
    write(".vscode/settings.json", settings);
    assert.match(
      run(directory, ["run", "editor:setup"], 1),
      /TypeError: vue: unknown option "lsp"/,
    );
    assert.equal(
      fs.readFileSync(path.join(directory, ".vscode/extensions.json"), "utf8"),
      extensions,
    );
    assert.equal(fs.readFileSync(path.join(directory, ".vscode/settings.json"), "utf8"), settings);
  } finally {
    write("vite.config.ts", originalConfig);
    write(".vscode/settings.json", originalSettings);
    write(".vscode/extensions.json", originalExtensions);
  }
}
