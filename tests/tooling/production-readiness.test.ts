import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("CLI typechecker reports mapped TypeScript diagnostics", () => {
  const checker = resolveCheckerPath();
  const fixtureDir = path.join(root, "tests/_fixtures/_projects/typecheck-errors");
  const result = runVize(
    ["check", "src/TypeMismatch.vue", "--format", "json", "--quiet", "--corsa-path", checker],
    fixtureDir,
  );

  assert.equal(result.status, 1, result.stderr);
  const parsed = JSON.parse(result.stdout) as {
    errorCount: number;
    fileCount: number;
    files: Array<{ file: string; diagnostics: string[] }>;
  };

  assert.equal(parsed.fileCount, 1);
  assert.equal(parsed.errorCount, 1);
  assert.equal(parsed.files[0]?.file, "src/TypeMismatch.vue");
  assert.match(parsed.files[0]?.diagnostics.join("\n") ?? "", /TS2322/);
});

test("CLI linter reports production JSON diagnostics", () => {
  const smokeDir = path.join(root, "__agent_only", "production-readiness");
  const fixturePath = path.join(smokeDir, "Lint.vue");

  fs.rmSync(smokeDir, { force: true, recursive: true });
  fs.mkdirSync(smokeDir, { recursive: true });

  try {
    fs.writeFileSync(
      fixturePath,
      `<script setup lang="ts">
const items = [1]
</script>

<template>
  <div v-for="item in items">{{ item }}</div>
</template>
`,
      "utf8",
    );

    const result = runVize(["lint", fixturePath, "--format", "json", "--quiet"], root);
    assert.equal(result.status, 1, result.stderr);

    const parsed = JSON.parse(result.stdout) as Array<{
      file: string;
      errorCount: number;
      warningCount: number;
      messages: Array<{ ruleId: string; line: number; column: number }>;
    }>;

    assert.equal(parsed.length, 1);
    assert.equal(parsed[0]?.file, fixturePath);
    assert.equal(parsed[0]?.errorCount, 1);
    assert.equal(parsed[0]?.warningCount, 0);
    assert.equal(parsed[0]?.messages[0]?.ruleId, "vue/require-v-for-key");
    assert.equal(parsed[0]?.messages[0]?.line, 6);
    assert.equal(parsed[0]?.messages[0]?.column, 8);
  } finally {
    fs.rmSync(smokeDir, { force: true, recursive: true });
  }
});

function runVize(
  args: string[],
  cwd: string,
): { status: number | null; stdout: string; stderr: string } {
  const [command, ...prefixArgs] = resolveVizeCommand();
  const result = spawnSync(command, [...prefixArgs, ...args], {
    cwd,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });

  if (result.error != null) {
    throw result.error;
  }

  return {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

function resolveVizeCommand(): string[] {
  const candidates = [
    [path.join(root, "target/release/vize")],
    [path.join(root, "target/debug/vize")],
    ["vize"],
  ];

  for (const candidate of candidates) {
    const result = spawnSync(candidate[0], ["--version"], {
      cwd: root,
      encoding: "utf8",
    });
    if (result.status === 0) {
      return candidate;
    }
  }

  return ["cargo", "run", "-q", "-p", "vize", "--"];
}

function resolveCheckerPath(): string {
  const candidates = [
    path.join(root, "node_modules/.bin/corsa"),
    path.join(root, "node_modules/.bin/tsgo"),
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  throw new Error("Unable to find corsa or tsgo in node_modules/.bin");
}
