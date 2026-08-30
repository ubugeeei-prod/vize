import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { resetFixtureDir } from "./test-support/fixture-dir.ts";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint/dist/index.mjs");
const fixtureDir = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "oxlint-plugin-vize-type-aware-test",
);
const ansiEscapePattern = new RegExp(String.raw`\u001B\[[0-9;]*m`, "gu");
const oxlintEnv = { ...process.env };
delete oxlintEnv.GITHUB_ACTIONS;

function findOxlintBin() {
  const pnpmStoreDir = path.join(workspaceRoot, "node_modules", ".pnpm");
  const match = fs
    .readdirSync(pnpmStoreDir)
    .filter((entry) => entry.startsWith("oxlint@"))
    .sort((left, right) => right.localeCompare(left))
    .map((entry) => path.join(pnpmStoreDir, entry, "node_modules", "oxlint", "bin", "oxlint"))
    .find((entry) => fs.existsSync(entry));

  if (match == null) {
    throw new Error(`Unable to locate the oxlint binary in ${pnpmStoreDir}`);
  }

  return match;
}

function runOxlint(args: readonly string[]) {
  let output = "";
  let exitCode = 0;

  try {
    output = String(
      execFileSync(findOxlintBin(), args, {
        cwd: fixtureDir,
        encoding: "utf8",
        env: oxlintEnv,
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
    output: output.replace(ansiEscapePattern, "").trim(),
  };
}

resetFixtureDir(fixtureDir);
fs.writeFileSync(
  path.join(fixtureDir, ".oxlintrc.json"),
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "opinionated",
          corsaPath: path.join(fixtureDir, "__missing_tsgo__"),
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/type/no-floating-promises": "error",
      },
    },
    null,
    2,
  ),
);
fs.writeFileSync(
  path.join(fixtureDir, "TypeAwarePromise.vue"),
  `<script setup lang="ts">
async function loadData(): Promise<number> {
  return 1
}

loadData()
</script>
`,
);

const typeAwareMissingCorsaRun = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "."]);
assert.notEqual(
  typeAwareMissingCorsaRun.exitCode,
  0,
  "type-aware rules should surface Corsa runtime failures through Oxlint",
);
assert.match(
  typeAwareMissingCorsaRun.output,
  /vize\(type\/no-floating-promises\)/u,
  "type-aware runtime failures should be attached to the configured type-aware rule",
);
assert.match(
  typeAwareMissingCorsaRun.output,
  /Configured Corsa executable does not exist[\s\S]*__missing_tsgo__/u,
  "settings.vize.corsaPath should be passed to the native type-aware lint runtime",
);
