// `vue.plugins` lint rules hosted inside Oxlint (#13074): one sidecar per
// Oxlint process, host-owned severities, and visible failures.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../../..");
const pluginEntry = path.join(workspaceRoot, "npm/linter/oxlint/dist/index.mjs");
const sdk = pathToFileURL(path.join(workspaceRoot, "npm/plugin-sdk/v2.js")).href;
// Oxlint reports physical paths (macOS tmpdir is a symlink).
const fixtureRoot = fs.realpathSync(
  fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vize-plugin-rules-")),
);
const oxlint = findOxlintBin();
// Oxlint visits a Vue file through its script, so the fixture has one.
const BUTTON =
  "<script setup>\nconst label = 'Go';\n</script>\n\n<template>\n  <button :color=\"'red'\" :size=\"'big'\">{{ label }}</button>\n</template>\n";

try {
  const project = createFixture("project", "");
  const result = runOxlint(project, ["Button.vue", "Other.vue"]);
  const diagnostics = report(result.output);
  assert.deepEqual(
    diagnostics.map(({ code, message, filename }) => [code, message.split("\n")[0], filename]),
    [
      [
        "vize(plugin-errors)",
        "[@acme/design/no-raw-color] Use a token instead of 'red'. (at <template>:6:11)",
        "Button.vue",
      ],
      [
        "vize(plugin-warnings)",
        "[@acme/design/no-raw-size] Avoid size 'big'. (at <template>:6:26)",
        "Button.vue",
      ],
      [
        "vize(plugin-errors)",
        "[@acme/design/no-raw-color] Use a token instead of 'red'. (at <template>:6:11)",
        "Other.vue",
      ],
      [
        "vize(plugin-warnings)",
        "[@acme/design/no-raw-size] Avoid size 'big'. (at <template>:6:26)",
        "Other.vue",
      ],
    ],
    result.output,
  );
  // The native session evaluated the config once and its sidecar once, for both files.
  assert.equal(fs.readFileSync(path.join(project, "evaluations.log"), "utf8"), "x\nx\n");

  const failing = createFixture("failing", 'throw new Error("rule-specificreason");');
  const failed = report(runOxlint(failing, ["Button.vue"]).output);
  assert.deepEqual(
    failed.map(({ code, message }) => [code, message]),
    [
      [
        "vize(plugin-errors)",
        `[plugin/rule-failed] @acme/design @acme/design/no-raw-color: ${path.join(failing, "Button.vue")}: rule-specificreason`,
      ],
    ],
  );
} finally {
  fs.rmSync(fixtureRoot, { recursive: true, force: true });
}

console.log("oxlint-plugin-vize plugin rule tests passed!");

function createFixture(name: string, failure: string): string {
  const fixture = path.join(fixtureRoot, name);
  fs.mkdirSync(fixture, { recursive: true });
  const log = JSON.stringify(path.join(fixture, "evaluations.log"));
  fs.writeFileSync(
    path.join(fixture, "vite.config.mjs"),
    `import { appendFileSync } from "node:fs";
import { defineVizePlugin } from ${JSON.stringify(sdk)};
appendFileSync(${log}, "x\\n");
setInterval(() => {}, 1000);
const design = defineVizePlugin({
  apiVersion: 2, stability: "experimental", name: "@acme/design", version: "1.0.0",
  setup(api) {
    api.lint.rule({
      id: "no-raw-color",
      meta: { messages: { raw: "Use a token instead of {value}." }, defaultLevel: "error" },
      visit: ["ui.bind"],
      run(context) {
        ${failure}
        for (const node of context.nodes)
          if (node.name === "color") context.report({ node, messageId: "raw", data: { value: node.value } });
      },
    });
    api.lint.rule({
      id: "no-raw-size",
      meta: { messages: { raw: "Avoid size {value}." }, defaultLevel: "warn" },
      visit: ["ui.bind"],
      run(context) {
        for (const node of context.nodes)
          if (node.name === "size") context.report({ node, messageId: "raw", data: { value: node.value } });
      },
    });
  },
});
export default { vue: { plugins: [design()] } };
`,
  );
  fs.writeFileSync(
    path.join(fixture, ".oxlintrc.json"),
    JSON.stringify(
      {
        jsPlugins: [pluginEntry],
        settings: { vize: { helpLevel: "none", preset: "incremental", pluginRules: true } },
        rules: { "vize/plugin-errors": "error", "vize/plugin-warnings": "warn" },
      },
      null,
      2,
    ),
  );
  fs.writeFileSync(path.join(fixture, "Button.vue"), BUTTON);
  fs.writeFileSync(path.join(fixture, "Other.vue"), BUTTON);
  return fixture;
}

function report(output: string): Array<{ code: string; message: string; filename: string }> {
  const parsed = JSON.parse(output) as {
    diagnostics: Array<{ code: string; message: string; filename: string }>;
  };
  return parsed.diagnostics.sort(
    (left, right) =>
      left.filename.localeCompare(right.filename) || left.code.localeCompare(right.code),
  );
}

function findOxlintBin(): string {
  const pnpmStoreDir = path.join(workspaceRoot, "node_modules", ".pnpm");
  const candidate = fs
    .readdirSync(pnpmStoreDir)
    .filter((entry) => entry.startsWith("oxlint@"))
    .sort((left, right) => right.localeCompare(left))
    .map((entry) => path.join(pnpmStoreDir, entry, "node_modules", "oxlint", "bin", "oxlint"))
    .find((entry) => fs.existsSync(entry));
  if (!candidate) throw new Error(`Unable to locate the oxlint binary in ${pnpmStoreDir}`);
  return candidate;
}

function runOxlint(fixture: string, files: string[]): { exitCode: number; output: string } {
  const env = { ...process.env };
  delete env.GITHUB_ACTIONS;
  const result = spawnSync(oxlint, ["-c", ".oxlintrc.json", "-f", "json", ...files], {
    cwd: fixture,
    encoding: "utf8",
    env,
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 60_000,
  });
  if (result.error) throw result.error;
  return { exitCode: result.status ?? 1, output: result.stdout.trim() };
}
