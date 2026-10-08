import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vite-plus/test";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));

function withInstalledConfig(run: (directory: string, loads: string) => void): void {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-cold-native-"));
  const modules = path.join(directory, "node_modules");
  const installed = path.join(modules, "vize");
  const loads = path.join(directory, "binding-loads.txt");
  try {
    fs.mkdirSync(installed, { recursive: true });
    fs.copyFileSync(path.join(packageRoot, "package.json"), path.join(installed, "package.json"));
    fs.cpSync(path.join(packageRoot, "dist"), path.join(installed, "dist"), { recursive: true });
    for (const [name, type, entry] of [
      ["@vizejs/native", "commonjs", "index.cjs"],
      ["oxc-transform", "module", "index.mjs"],
    ]) {
      const dependency = path.join(modules, name);
      fs.mkdirSync(dependency, { recursive: true });
      fs.writeFileSync(
        path.join(dependency, "package.json"),
        JSON.stringify({ name, type, exports: `./${entry}` }),
      );
      const record = `appendFileSync(${JSON.stringify(loads)}, ${JSON.stringify(name + "\n")});`;
      fs.writeFileSync(
        path.join(dependency, entry),
        `${type === "module" ? 'import { appendFileSync } from "node:fs";' : 'const { appendFileSync } = require("node:fs");'}
${record}
throw new Error(${JSON.stringify(`${name}: deliberately unavailable binding`)});
`,
      );
    }
    run(directory, loads);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function execute(directory: string, script: string): void {
  const result = spawnSync(process.execPath, ["--input-type=module", "-e", script], {
    cwd: directory,
    encoding: "utf8",
    timeout: 15_000,
    maxBuffer: 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.stderr);
  assert.equal(result.signal, null, result.stderr);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "");
  assert.equal(result.stderr, "");
}

test("installed defineConfig preserves imported CLI settings without loading native bindings", () => {
  withInstalledConfig((directory, loads) => {
    const shared = {
      linter: {
        enabled: true,
        preset: "incremental",
        rules: { "vue/attribute-hyphenation": "warn", "vue/require-v-for-key": "error" },
        ruleOptions: {
          "vue/attribute-hyphenation": "always",
          "vue/component-name-in-template-casing": { casing: "PascalCase" },
          "vue/sfc-element-order": { order: ["script", "template", "style"] },
        },
      },
      entries: [
        {
          files: ["src/Fragment.vue"],
          linter: { rules: { "vue/no-multiple-template-root": "off" } },
        },
      ],
    };
    fs.writeFileSync(
      path.join(directory, "shared.mjs"),
      `export const shared = ${JSON.stringify(shared)};\n`,
    );
    execute(
      directory,
      `import assert from "node:assert/strict";
import { defineConfig, normalizeGlobalTypes } from "vize/config";
import { shared } from "./shared.mjs";
assert.equal(defineConfig(shared), shared);
const array = [shared];
assert.equal(defineConfig(array), array);
const factory = () => shared;
assert.equal(defineConfig(factory), factory);
const asyncFactory = async () => shared;
assert.equal(defineConfig(asyncFactory), asyncFactory);
assert.deepEqual(normalizeGlobalTypes({ types: { UserId: "string" } }), { UserId: { type: "string" } });
`,
    );
    assert.equal(fs.existsSync(loads), false, "helper import must not load either native binding");
  });
});

test("installed resolveConfigExport still requires native structural normalization", () => {
  withInstalledConfig((directory, loads) => {
    execute(
      directory,
      `import assert from "node:assert/strict";
import { resolveConfigExport } from "vize/config";
await assert.rejects(resolveConfigExport({ linter: { enabled: true } }), {
  message: "@vizejs/native: deliberately unavailable binding",
});
`,
    );
    assert.equal(fs.readFileSync(loads, "utf8"), "@vizejs/native\n");
  });
});

test("installed JSON config loading keeps native normalization failures visible", () => {
  withInstalledConfig((directory, loads) => {
    fs.writeFileSync(path.join(directory, "vize.config.json"), '{"linter":{"enabled":true}}');
    execute(
      directory,
      `import assert from "node:assert/strict";
import { loadConfig } from "vize/config";
await assert.rejects(loadConfig(process.cwd()), (error) => error.message.endsWith("@vizejs/native: deliberately unavailable binding"));
`,
    );
    assert.equal(fs.readFileSync(loads, "utf8"), "@vizejs/native\n");
  });
});

test("installed TypeScript config loading still requires its native transformer", () => {
  withInstalledConfig((directory, loads) => {
    fs.writeFileSync(
      path.join(directory, "vize.config.ts"),
      "export default { linter: { enabled: true } };\n",
    );
    execute(
      directory,
      `import assert from "node:assert/strict";
import { loadConfig } from "vize/config";
await assert.rejects(loadConfig(process.cwd()), {
  message: "oxc-transform: deliberately unavailable binding",
});
`,
    );
    assert.equal(fs.readFileSync(loads, "utf8"), "oxc-transform\n");
  });
});
