import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { extractSfcBlocks, formatBlockLabel, getDiagnosticBlock } from "./sfc-blocks.ts";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint-plugin-vize/dist/index.mjs");
const cliEntry = path.join(workspaceRoot, "npm/oxlint-plugin-vize/dist/cli.mjs");
const fixtureDir = path.join(workspaceRoot, "__agent_only", "oxlint-plugin-vize-test");
const configPath = path.join(fixtureDir, ".oxlintrc.json");
const noHelpConfigPath = path.join(fixtureDir, ".oxlintrc.no-help.json");
const shortHelpConfigPath = path.join(fixtureDir, ".oxlintrc.short-help.json");
const generalRecommendedStyleConfigPath = path.join(
  fixtureDir,
  ".oxlintrc.general-recommended-style.json",
);
const essentialStyleConfigPath = path.join(fixtureDir, ".oxlintrc.essential-style.json");
const generalRecommendedScriptConfigPath = path.join(
  fixtureDir,
  ".oxlintrc.general-recommended-script.json",
);
const incrementalComboConfigPath = path.join(fixtureDir, ".oxlintrc.incremental-combo.json");
const opinionatedScriptConfigPath = path.join(fixtureDir, ".oxlintrc.opinionated-script.json");
const coreRulesConfigPath = path.join(fixtureDir, ".oxlintrc.core-rules.json");
const scriptlessConfigPath = path.join(fixtureDir, ".oxlintrc.scriptless.json");
const largeJsonConfigPath = path.join(fixtureDir, ".oxlintrc.large-json.json");
const vuePath = path.join(fixtureDir, "App.vue");
const scopedStyleVuePath = path.join(fixtureDir, "ScopedStyle.vue");
const incrementalComboVuePath = path.join(fixtureDir, "IncrementalCombo.vue");
const optionsApiVuePath = path.join(fixtureDir, "OptionsApi.vue");
const dualScriptVuePath = path.join(fixtureDir, "DualScript.vue");
const coreRulesVuePath = path.join(fixtureDir, "CoreRules.vue");
const scriptlessVuePath = path.join(fixtureDir, "Scriptless.vue");
const hugeJsonVuePath = path.join(fixtureDir, "HugeJson.vue");
const snapshotsDir = path.join(packageDir, "__snapshots__");
const ansiEscapePattern = new RegExp(String.raw`\u001B\[[0-9;]*m`, "gu");
const workspaceRootPattern = new RegExp(escapeRegExp(workspaceRoot), "gu");

function findOxlintBin() {
  const pnpmStoreDir = path.join(workspaceRoot, "node_modules", ".pnpm");
  const candidates = fs
    .readdirSync(pnpmStoreDir)
    .filter((entry) => entry.startsWith("oxlint@"))
    .sort((left, right) => right.localeCompare(left))
    .map((entry) => path.join(pnpmStoreDir, entry, "node_modules", "oxlint", "bin", "oxlint"))
    .filter((entry) => fs.existsSync(entry));

  const match = candidates[0];
  if (match == null) {
    throw new Error(`Unable to locate the oxlint binary in ${pnpmStoreDir}`);
  }

  return match;
}

const oxlintBin = findOxlintBin();

fs.rmSync(fixtureDir, { force: true, recursive: true });
fs.mkdirSync(fixtureDir, { recursive: true });

fs.writeFileSync(
  configPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      rules: {
        "no-unused-vars": "off",
        "vize/vue/require-v-for-key": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  noHelpConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          showHelp: false,
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/vue/require-v-for-key": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  shortHelpConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "short",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/vue/require-v-for-key": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  generalRecommendedStyleConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "general-recommended",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/vue/require-scoped-style": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  essentialStyleConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "essential",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/vue/require-scoped-style": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  generalRecommendedScriptConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "general-recommended",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/script/no-options-api": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  incrementalComboConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "incremental",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/script/no-options-api": "error",
        "vize/vue/require-scoped-style": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  opinionatedScriptConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "opinionated",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/script/no-options-api": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  coreRulesConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "general-recommended",
        },
      },
      rules: {
        "no-unused-vars": "off",
        eqeqeq: "error",
        "no-console": "warn",
        "vize/vue/require-v-for-key": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  scriptlessConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "incremental",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/vue/require-v-for-key": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  largeJsonConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "incremental",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/vue/component-name-in-template-casing": "warn",
        "vize/vue/no-multi-spaces": "warn",
        "vize/vue/require-component-registration": "warn",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  vuePath,
  `<script setup lang="ts">
const items = [1]
</script>
<template>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
`,
);

fs.writeFileSync(
  scopedStyleVuePath,
  `<script setup lang="ts">
const count = 1
</script>
<template>
  <div class="foo">{{ count }}</div>
</template>
<style>
.foo { color: red; }
</style>
`,
);

fs.writeFileSync(
  optionsApiVuePath,
  `<script lang="ts">
export default {
  data() {
    return {
      count: 1,
    };
  },
};
</script>
<template>
  <div>{{ count }}</div>
</template>
`,
);

fs.writeFileSync(
  incrementalComboVuePath,
  `<script lang="ts">
export default {
  data() {
    return {
      count: 1,
    };
  },
};
</script>
<template>
  <div class="foo">{{ count }}</div>
</template>
<style>
.foo { color: red; }
  </style>
`,
);

fs.writeFileSync(
  dualScriptVuePath,
  `<script lang="ts">
export default {
  data() {
    return {
      count: 1,
    };
  },
};
</script>
<script setup lang="ts">
const ready = true
</script>
<template>
  <div>{{ count }} {{ ready }}</div>
</template>
`,
);

fs.writeFileSync(
  coreRulesVuePath,
  `<script setup lang="ts">
const count = 1

if (count == 1) {
  console.log(count)
}
</script>
<template>
  <ul>
    <li v-for="item in [count]">{{ item }}</li>
  </ul>
</template>
`,
);

fs.writeFileSync(
  scriptlessVuePath,
  `<template>
  <ul>
    <li v-for="item in [1, 2]">{{ item }}</li>
  </ul>
</template>
`,
);

fs.writeFileSync(
  hugeJsonVuePath,
  `<template>
${Array.from({ length: 700 }, (_, index) => `  <demo-card  :title="'Item ${index}'" />`).join("\n")}
</template>
`,
);

function runOxlint(args: readonly string[]) {
  let output = "";
  let exitCode = 0;

  try {
    output = String(
      execFileSync(oxlintBin, args, {
        cwd: fixtureDir,
        encoding: "utf8",
        stdio: "pipe",
      }),
    );
  } catch (error) {
    const execError = error as {
      status?: number;
      stdout?: string | Buffer;
      stderr?: string | Buffer;
    };
    exitCode = execError.status ?? 1;
    output = String(execError.stdout ?? "") + String(execError.stderr ?? "");
  }

  return {
    exitCode,
    output: normalizeOutput(output),
  };
}

function runOxlintVize(args: readonly string[]) {
  let output = "";
  let exitCode = 0;

  try {
    output = String(
      execFileSync(process.execPath, [cliEntry, ...args], {
        cwd: fixtureDir,
        encoding: "utf8",
        stdio: "pipe",
      }),
    );
  } catch (error) {
    const execError = error as {
      status?: number;
      stdout?: string | Buffer;
      stderr?: string | Buffer;
    };
    exitCode = execError.status ?? 1;
    output = String(execError.stdout ?? "") + String(execError.stderr ?? "");
  }

  return {
    exitCode,
    output: normalizeOutput(output),
  };
}

function normalizeOutput(output: string): string {
  return output
    .replace(ansiEscapePattern, "")
    .replace(workspaceRootPattern, "<workspaceRoot>")
    .replace(/^WARNING: JS plugins are experimental and not subject to semver\.\n/gmu, "")
    .replace(
      /^Breaking changes are possible while JS plugins support is under development\.\n/gmu,
      "",
    )
    .replace(/^\s*\[tsgo\] Using: .*$/gmu, "")
    .replace(/"start_time": [0-9.]+/gu, '"start_time": 0')
    .replace(/^Finished in .*$/gmu, "")
    .trim();
}

function escapeRegExp(value: string): string {
  return value.replaceAll(/[.*+?^${}()|[\]\\]/gu, String.raw`\$&`);
}

function readSnapshot(name: string): string {
  return fs.readFileSync(path.join(snapshotsDir, name), "utf8").trim();
}

const defaultRun = runOxlint(["-c", ".oxlintrc.json", "App.vue"]);
assert.notEqual(defaultRun.exitCode, 0, "oxlint should fail when Patina reports an error");
assert.match(
  defaultRun.output,
  /vize\(vue\/require-v-for-key\)/,
  "Vize rule name should be surfaced",
);
assert.match(
  defaultRun.output,
  /Elements in iteration expect to have 'v-bind:key' directives\. \(at <template>:6:9\)/u,
  "Template diagnostics should expose the real Vue location inline in the summary",
);
assert.doesNotMatch(
  defaultRun.output,
  /^  \| Source:/mu,
  "The inline source snippet should be gone",
);
assert.match(
  defaultRun.output,
  /^  \|     Help:/mu,
  "Default output should still include help text",
);

const stylishRun = runOxlint(["-c", ".oxlintrc.no-help.json", "-f", "stylish", "App.vue"]);
assert.notEqual(stylishRun.exitCode, 0, "stylish formatter should still report Patina failures");
assert.match(
  stylishRun.output,
  /App\.vue[\s\S]*2:1[\s\S]*Elements in iteration expect to have 'v-bind:key' directives\. \(at <template>:6:9\)[\s\S]*^    Details:$[\s\S]*^      Element: <li>  vize\(vue\/require-v-for-key\)$/mu,
  "Stylish formatter should keep the output concise while surfacing the real Vue location",
);
assert.doesNotMatch(stylishRun.output, /^Help:/mu, "showHelp: false should omit the help section");
assert.doesNotMatch(
  stylishRun.output,
  /^    Location:/mu,
  "Stylish formatter should avoid a separate location block",
);
assert.equal(stylishRun.output, readSnapshot("stylish-no-help-output.txt"));

const shortHelpRun = runOxlint(["-c", ".oxlintrc.short-help.json", "-f", "stylish", "App.vue"]);
assert.notEqual(shortHelpRun.exitCode, 0, "short help should still report Patina failures");
assert.match(
  shortHelpRun.output,
  /^    Help:\n      Why:/mu,
  "short help should keep only the first help line",
);
assert.equal(shortHelpRun.output, readSnapshot("stylish-short-help-output.txt"));

const jsonRun = runOxlint(["-c", ".oxlintrc.no-help.json", "-f", "json", "App.vue"]);
assert.notEqual(jsonRun.exitCode, 0, "json formatter should still report Patina failures");
assert.equal(jsonRun.output, readSnapshot("json-no-help-output.txt"));

const scriptlessJsonRun = runOxlintVize([
  "-c",
  ".oxlintrc.scriptless.json",
  "-f",
  "json",
  "Scriptless.vue",
]);
assert.notEqual(
  scriptlessJsonRun.exitCode,
  0,
  "scriptless JSON output should still fail when Patina reports an error",
);
assert.match(
  scriptlessJsonRun.output,
  /"filename": ".*Scriptless\.vue"/u,
  "scriptless JSON output should rewrite temporary filenames back to the original SFC",
);
assert.doesNotMatch(
  scriptlessJsonRun.output,
  /__oxlint_plugin_vize_temp__/u,
  "scriptless JSON output should not leak temporary workaround paths",
);
assert.equal(scriptlessJsonRun.output, readSnapshot("json-scriptless-workaround-output.txt"));

const largeJsonRun = runOxlintVize([
  "-c",
  ".oxlintrc.large-json.json",
  "-f",
  "json",
  "HugeJson.vue",
]);
assert.equal(
  largeJsonRun.exitCode,
  0,
  "large JSON runs with warning-only diagnostics should still stay parseable",
);
const largeJsonPayload = JSON.parse(largeJsonRun.output) as {
  diagnostics: Array<{ code: string; filename: string }>;
  number_of_files: number;
};
assert.equal(largeJsonPayload.number_of_files, 1, "large JSON output should stay parseable");
assert.equal(
  largeJsonPayload.diagnostics.every((diagnostic) => diagnostic.filename === "HugeJson.vue"),
  true,
  "large JSON output should keep reporting the original scriptless filename",
);
assert.equal(
  largeJsonPayload.diagnostics.some(
    (diagnostic) => diagnostic.code === "vize(vue/require-component-registration)",
  ),
  true,
  "large JSON output should preserve diagnostics past the old spawnSync capture limit",
);
assert.equal(
  largeJsonPayload.diagnostics.length > 1000,
  true,
  "large JSON output should include all diagnostics instead of truncating mid-stream",
);

const coreRulesRun = runOxlint([
  "-c",
  ".oxlintrc.core-rules.json",
  "-f",
  "stylish",
  "CoreRules.vue",
]);
assert.notEqual(
  coreRulesRun.exitCode,
  0,
  "Oxlint core rules should still fail alongside Vize rules",
);
assert.match(
  coreRulesRun.output,
  /eqeqeq/u,
  "Oxlint core rules such as eqeqeq should still report through the same Vue run",
);
assert.match(
  coreRulesRun.output,
  /no-console/u,
  "Oxlint core rules such as no-console should still report through the same Vue run",
);
assert.match(
  coreRulesRun.output,
  /vize\(vue\/require-v-for-key\)/u,
  "Vize rules should keep reporting alongside Oxlint core rules",
);
assert.equal(coreRulesRun.output, readSnapshot("stylish-core-rules-and-vize-output.txt"));

const happyPathStyleRun = runOxlint([
  "-c",
  ".oxlintrc.general-recommended-style.json",
  "-f",
  "stylish",
  "ScopedStyle.vue",
]);
assert.notEqual(
  happyPathStyleRun.exitCode,
  0,
  "general-recommended should report require-scoped-style when the rule is enabled",
);
assert.match(
  happyPathStyleRun.output,
  /vize\(vue\/require-scoped-style\)/,
  "general-recommended should keep reporting rules that belong to the default preset",
);
assert.equal(
  happyPathStyleRun.output,
  readSnapshot("stylish-general-recommended-require-scoped-style-output.txt"),
);

const essentialStyleRun = runOxlint([
  "-c",
  ".oxlintrc.essential-style.json",
  "-f",
  "stylish",
  "ScopedStyle.vue",
]);
assert.equal(
  essentialStyleRun.exitCode,
  0,
  "essential preset should skip require-scoped-style even when the Oxlint rule is configured",
);
assert.doesNotMatch(
  essentialStyleRun.output,
  /vize\(vue\/require-scoped-style\)/,
  "essential preset should not surface general-recommended-only rules",
);

const generalRecommendedScriptRun = runOxlint([
  "-c",
  ".oxlintrc.general-recommended-script.json",
  "-f",
  "stylish",
  "OptionsApi.vue",
]);
assert.equal(
  generalRecommendedScriptRun.exitCode,
  0,
  "general-recommended should not enable opinionated script rules by default",
);
assert.doesNotMatch(
  generalRecommendedScriptRun.output,
  /vize\(script\/no-options-api\)/,
  "general-recommended should keep script/no-options-api opt-in",
);

const incrementalComboRun = runOxlint([
  "-c",
  ".oxlintrc.incremental-combo.json",
  "-f",
  "stylish",
  "IncrementalCombo.vue",
]);
assert.notEqual(
  incrementalComboRun.exitCode,
  0,
  "incremental preset should report explicitly configured rules",
);
assert.match(
  incrementalComboRun.output,
  /vize\(script\/no-options-api\)/,
  "incremental preset should allow explicitly configured script rules",
);
assert.match(
  incrementalComboRun.output,
  /vize\(vue\/require-scoped-style\)/,
  "incremental preset should allow explicitly configured template or style rules",
);
assert.equal(incrementalComboRun.output, readSnapshot("stylish-incremental-combo-output.txt"));

const opinionatedScriptRun = runOxlint([
  "-c",
  ".oxlintrc.opinionated-script.json",
  "-f",
  "stylish",
  "OptionsApi.vue",
]);
assert.notEqual(
  opinionatedScriptRun.exitCode,
  0,
  "opinionated preset should enable script/no-options-api",
);
assert.match(
  opinionatedScriptRun.output,
  /vize\(script\/no-options-api\)/,
  "opinionated preset should surface opinionated script diagnostics",
);
assert.equal(
  opinionatedScriptRun.output,
  readSnapshot("stylish-opinionated-no-options-api-output.txt"),
);

const dualScriptRun = runOxlint([
  "-c",
  ".oxlintrc.opinionated-script.json",
  "-f",
  "stylish",
  "DualScript.vue",
]);
assert.notEqual(
  dualScriptRun.exitCode,
  0,
  "dual-script SFCs should still report opinionated script diagnostics",
);
assert.equal(
  dualScriptRun.output.match(/vize\(script\/no-options-api\)/gu)?.length ?? 0,
  1,
  "dual-script SFCs should not duplicate the same rule diagnostic across <script> and <script setup>",
);
assert.equal(dualScriptRun.output, readSnapshot("stylish-dual-script-no-options-api-output.txt"));

const scriptlessRun = runOxlintVize([
  "-c",
  ".oxlintrc.scriptless.json",
  "-f",
  "stylish",
  "Scriptless.vue",
]);
assert.notEqual(
  scriptlessRun.exitCode,
  0,
  "oxlint-vize should lint scriptless SFCs through the temporary-file workaround",
);
assert.match(
  scriptlessRun.output,
  /Scriptless\.vue[\s\S]*3:9[\s\S]*Elements in iteration expect to have 'v-bind:key' directives\./u,
  "scriptless workaround should report the original file location instead of the temporary script block",
);
assert.doesNotMatch(
  scriptlessRun.output,
  /node_modules\/\.cache\/oxlint-plugin-vize/u,
  "scriptless workaround should not leak temporary cache paths to users",
);
assert.equal(scriptlessRun.output, readSnapshot("stylish-scriptless-workaround-output.txt"));

const sampleBlocks = extractSfcBlocks(
  `<script setup lang="ts">\nconst count = 1\n</script>\n<template>\n  <div>{{ count }}</div>\n</template>\n<style scoped>\n.foo {}\n</style>\n<i18n>\n{}\n</i18n>\n`,
);
assert.deepEqual(
  sampleBlocks.map((block) => formatBlockLabel(block)),
  ["<script setup>", "<template>", "<style>", "<i18n>"],
  "SFC block extraction should classify common Vue block types",
);
assert.equal(
  formatBlockLabel(
    getDiagnosticBlock(
      {
        rule: "vize/vue/mock",
        severity: "error",
        message: "Mock error. Detail: extra context",
        location: {
          start: { line: 5, column: 3, offset: 0 },
          end: { line: 5, column: 8, offset: 0 },
        },
        help: null,
      },
      sampleBlocks,
    ),
  ),
  "<template>",
  "Diagnostics should map back to their containing SFC block",
);

console.log("✅ oxlint-plugin-vize integration tests passed!");
