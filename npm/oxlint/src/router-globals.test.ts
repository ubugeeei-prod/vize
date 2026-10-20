import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint/dist/index.mjs");
const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-vize-router-globals-"));
const oxlint = findOxlintBin();

try {
  const withRouter = createFixture("with-router");
  const routerPackage = path.join(withRouter, "node_modules/vue-router");
  fs.mkdirSync(routerPackage, { recursive: true });
  fs.writeFileSync(
    path.join(routerPackage, "package.json"),
    JSON.stringify({ name: "vue-router", version: "5.3.1", main: "index.js" }),
  );
  fs.writeFileSync(path.join(routerPackage, "index.js"), "module.exports = {};\n");

  const accepted = runOxlint(withRouter);
  assert.equal(
    accepted.exitCode,
    0,
    `installed vue-router components should be accepted:\n${accepted.output}`,
  );
  assert.doesNotMatch(accepted.output, /vize\(vue\/require-component-registration\)/u);

  const withoutRouter = createFixture("without-router");
  const rejected = runOxlint(withoutRouter);
  const diagnostics = JSON.parse(rejected.output) as {
    diagnostics: Array<{ code: string }>;
  };
  assert.equal(
    diagnostics.diagnostics.filter(
      (diagnostic) => diagnostic.code === "vize(vue/require-component-registration)",
    ).length,
    3,
  );
} finally {
  fs.rmSync(fixtureRoot, { recursive: true, force: true });
}

console.log("oxlint-plugin-vize vue-router global component tests passed!");

function createFixture(name: string): string {
  const fixture = path.join(fixtureRoot, name);
  fs.mkdirSync(fixture, { recursive: true });
  fs.writeFileSync(
    path.join(fixture, ".oxlintrc.json"),
    JSON.stringify(
      {
        jsPlugins: [pluginEntry],
        settings: { vize: { helpLevel: "none", preset: "opinionated" } },
        rules: {
          "vize/vue/require-component-registration": ["warn", { globals: ["MyGlobal"] }],
        },
      },
      null,
      2,
    ),
  );
  fs.writeFileSync(
    path.join(fixture, "AppNav.vue"),
    `<script setup lang="ts">
const home = "/";
</script>

<template>
  <nav>
    <RouterLink :to="home">home</RouterLink>
    <router-link :to="home">home</router-link>
    <RouterView />
    <MyGlobal />
  </nav>
</template>
`,
  );
  return fixture;
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

function runOxlint(fixture: string): { exitCode: number; output: string } {
  const env = { ...process.env };
  delete env.GITHUB_ACTIONS;
  const result = spawnSync(oxlint, ["-c", ".oxlintrc.json", "-f", "json", "AppNav.vue"], {
    cwd: fixture,
    encoding: "utf8",
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) throw result.error;
  return {
    exitCode: result.status ?? 1,
    output: `${result.stdout}${result.stderr}`
      .replace(/^WARNING: JS plugins are experimental and not subject to semver\.\n/gmu, "")
      .replace(
        /^Breaking changes are possible while JS plugins support is under development\.\n/gmu,
        "",
      )
      .trim(),
  };
}
