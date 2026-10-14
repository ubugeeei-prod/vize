import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { buildScriptShim, requestsTypeAware, typeAwareOnlyConfig } from "./type-aware-vue.ts";

const workspaceRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const cliEntry = path.join(workspaceRoot, "npm/oxlint/dist/cli.mjs");

const BODY = [
  "async function work(): Promise<number> { return 1; }",
  "function run() {",
  "  work();",
  "  const n: number = 1;",
  "  return `${n as number}`;",
  "}",
  "void run;",
].join("\n");

void test("script shims keep every script offset and blank the rest", () => {
  const source = `<template>\n  <p>{{ msg }}</p>\n</template>\n<script setup lang="ts">\n${BODY}\n</script>\n<style>\np { color: red }\n</style>\n`;
  const shim = buildScriptShim(source);
  assert.ok(shim);
  assert.equal(shim.extension, ".ts");
  assert.equal(shim.text.length, source.length);
  assert.equal(shim.text.split("\n").length, source.split("\n").length);
  const start = source.indexOf(BODY);
  assert.equal(shim.text.slice(start, start + BODY.length), BODY);
  assert.equal(shim.text.slice(0, start).trim(), "");
  assert.equal(shim.text.slice(start + BODY.length).trim(), "");
});

void test("script shims merge both script blocks and prefer the TypeScript language", () => {
  const source =
    '<script>\nexport const plain = 1;\n</script>\n<script setup lang="tsx">\nconst a: number = plain;\n</script>\n';
  const shim = buildScriptShim(source);
  assert.ok(shim);
  assert.equal(shim.extension, ".tsx");
  assert.match(shim.text, /export const plain = 1;/u);
  assert.match(shim.text, /const a: number = plain;/u);
  assert.equal(buildScriptShim("<script>\nconst x = 1;\n</script>\n")?.extension, ".js");
});

void test("templates, external scripts and empty SFCs produce no shim", () => {
  assert.equal(buildScriptShim("<template><p /></template>\n"), null);
  assert.equal(buildScriptShim('<script src="./x.ts"></script>\n'), null);
});

void test("only enabled type-aware rules survive in the pass configuration", () => {
  const config = typeAwareOnlyConfig(
    {
      plugins: ["typescript", "vue"],
      rules: {
        "no-debugger": "deny",
        "typescript/no-floating-promises": "deny",
        "typescript/await-thenable": "allow",
        "typescript/no-explicit-any": "warn",
      },
      settings: { vize: {} },
    },
    new Set(["typescript/no-floating-promises", "typescript/await-thenable"]),
  );
  assert.ok(config);
  assert.deepEqual(config.rules, { "typescript/no-floating-promises": "deny" });
  assert.deepEqual(config.plugins, ["typescript", "vue"]);
  assert.equal((config.categories as Record<string, string>).correctness, "off");
  assert.equal(
    typeAwareOnlyConfig({ rules: { "no-debugger": "deny" } }, new Set(["typescript/x"])),
    null,
  );
});

void test("type-aware linting is requested by the flag or by options.typeAware", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-type-aware-request-"));
  try {
    assert.equal(requestsTypeAware(["--type-aware", "src"], root), true);
    assert.equal(requestsTypeAware(["src"], root), false);
    fs.writeFileSync(
      path.join(root, ".oxlintrc.json"),
      '{\n  // n8n style\n  "options": { "typeAware": true }\n}\n',
    );
    assert.equal(requestsTypeAware(["src"], root), true);
    assert.equal(requestsTypeAware(["--", "--type-aware"], path.join(root, "missing")), false);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

void test("oxlint-vize reports type-aware rules inside Vue script blocks at authored positions", () => {
  const tsgolint = findTsgolint();
  assert.ok(tsgolint, "oxlint-tsgolint must be installed in the workspace for this test");
  // A temp root outside the repository: the repository ignores target/, and
  // oxlint refuses to lint anything below an ignored working directory.
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vize-type-aware-"));
  try {
    fs.symlinkSync(path.join(workspaceRoot, "node_modules"), path.join(root, "node_modules"));
    fs.mkdirSync(path.join(root, "src"));
    fs.writeFileSync(
      path.join(root, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: { strict: true, target: "ES2022", module: "ESNext", noEmit: true },
        include: ["src/**/*.ts", "src/**/*.vue"],
      }),
    );
    fs.writeFileSync(
      path.join(root, ".oxlintrc.json"),
      JSON.stringify({
        plugins: ["typescript", "vue"],
        categories: { correctness: "off" },
        rules: {
          "typescript/no-floating-promises": "error",
          "typescript/no-unnecessary-type-assertion": "error",
        },
      }),
    );
    fs.writeFileSync(path.join(root, "src/a.ts"), `${BODY}\n`);
    fs.writeFileSync(
      path.join(root, "src/B.vue"),
      `<template><p>{{ 1 }}</p></template>\n<script setup lang="ts">\n${BODY}\n</script>\n`,
    );
    const result = spawnSync(
      process.execPath,
      [cliEntry, "--type-aware", "--format", "unix", "src/a.ts", "src/B.vue"],
      {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, OXLINT_TSGOLINT_PATH: tsgolint, GITHUB_ACTIONS: "" },
      },
    );
    const output = `${result.stdout}${result.stderr}`;
    for (const [file, line] of [
      ["src/a.ts", 3],
      ["src/B.vue", 5],
    ] as const) {
      assert.match(output, new RegExp(`${file}:${line}:3: .*no-floating-promises`, "u"), output);
      assert.match(
        output,
        new RegExp(`${file}:${line + 2}:13: .*no-unnecessary-type-assertion`, "u"),
        output,
      );
    }
    assert.doesNotMatch(output, /B\.vue\.ts/u, "shim paths must be rewritten to the SFC");
    assert.equal(fs.existsSync(path.join(root, "src/B.vue.ts")), false, "shims are removed");
    assert.equal(result.status, 1);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

function findTsgolint(): string | undefined {
  if (process.env.OXLINT_TSGOLINT_PATH) return process.env.OXLINT_TSGOLINT_PATH;
  const store = path.join(workspaceRoot, "node_modules", ".pnpm");
  const platform = `${process.platform}-${process.arch}`;
  return fs
    .readdirSync(store)
    .filter((entry) => entry.startsWith(`@oxlint-tsgolint+${platform}@`))
    .sort((left, right) => right.localeCompare(left, undefined, { numeric: true }))
    .map((entry) =>
      path.join(store, entry, "node_modules", "@oxlint-tsgolint", platform, "tsgolint"),
    )
    .find((candidate) => fs.existsSync(candidate));
}
