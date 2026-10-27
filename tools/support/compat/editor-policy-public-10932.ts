import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
const [directory, evidenceFile] = process.argv.slice(2);
assert.ok(directory && evidenceFile, "consumer directory and evidence file required");
const rootRequire = createRequire(path.join(directory, "package.json"));
const pluginManifest = fs.realpathSync(path.join(directory, "node_modules/@vizejs/vite-plugin/package.json"));
const pluginRequire = createRequire(pluginManifest);
const { parse: parseJsonc } = pluginRequire("jsonc-parser");
const vpPackage = rootRequire.resolve("vite-plus/package.json");
const vp = path.join(path.dirname(vpPackage), "bin/vp");
const commands: unknown[] = [];
const observations: unknown[] = [];
function run(cwd: string, args: string[], expected = 0) {
  const result = spawnSync(process.execPath, [vp, ...args], {
    cwd, encoding: "utf8", timeout: 120_000,
    env: { ...process.env, NO_COLOR: "1", FORCE_COLOR: "0" },
  });
  const output = result.stdout + result.stderr;
  commands.push({ args, expected, actual: result.status, output, error: result.error?.message });
  assert.equal(result.error, undefined);
  assert.equal(result.status, expected, output);
  return output;
}
const evidence: Record<string, unknown> = {
  issue: 10932, pr: 10947, consumer: directory,
  node: process.version, executable: process.execPath,
  startedAt: new Date().toISOString(), commands, observations,
  sourceRegressionHead: "cd63de50f9774a3b7a3ef8dc28da926cb046fcac",
  mergeSha: "d756cde149df2ba447bad11440f26a8e949a6c66",
  lockfileSha256: createHash("sha256").update(fs.readFileSync(path.join(directory, "pnpm-lock.yaml"))).digest("hex"),
  packages: Object.fromEntries(["@vizejs/vite-plugin", "vite-plus", "vize", "@vizejs/native", "@vizejs/unplugin"].map(name => {
    let packageRoot = name === "@vizejs/vite-plugin" ? path.dirname(pluginManifest) : name === "vite-plus" ? path.dirname(vpPackage) : path.dirname(pluginRequire.resolve(name));
    while (!fs.existsSync(path.join(packageRoot, "package.json"))) {
      const parent = path.dirname(packageRoot);
      assert.notEqual(parent, packageRoot, `package manifest missing: ${name}`);
      packageRoot = parent;
    }
    const real = fs.realpathSync(path.join(packageRoot, "package.json"));
    assert.ok(!real.includes("/Source/"), `workspace package leaked: ${name}: ${real}`);
    assert.ok(real.startsWith(fs.realpathSync(directory) + path.sep), `package escaped fresh consumer: ${name}: ${real}`);
    const pkg = JSON.parse(fs.readFileSync(real, "utf8"));
    assert.equal(pkg.name, name);
    return [name, { version: pkg.version, manifest: real }];
  })),
};
fs.mkdirSync(path.join(directory, ".vscode"), { recursive: true });
const write = (name: string, value: string) => fs.writeFileSync(path.join(directory, name), value);
write("vite.config.ts", 'import { defineConfig } from "@vizejs/vite-plugin/vite-plus";\nexport default defineConfig({}, { plugin: false, conflicts: false });\n');
write(".vscode/settings.json", "{}\n");
write(".vscode/extensions.json", "{}\n");
try {
  checkEditorSetupPolicy(directory, write);
  evidence.status = "passed";
  evidence.positiveCases = 7;
  evidence.idempotentRuns = 7;
  evidence.unsupportedAliasNoWrite = true;
} catch (error) {
  evidence.status = "failed";
  evidence.error = error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
  process.exitCode = 1;
} finally {
  evidence.finishedAt = new Date().toISOString();
  fs.writeFileSync(evidenceFile, JSON.stringify(evidence, null, 2) + "\n");
  console.log(JSON.stringify({ status: evidence.status, evidenceFile, packages: evidence.packages, invocations: commands.length }));
}

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
      observations.push({ name: entry.name, policy: entry.vue, formatterExpected: entry.formatting, settings: result, extensions: recommended, projectConfig: fs.readFileSync(path.join(directory, "vite.config.ts"), "utf8") });
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
