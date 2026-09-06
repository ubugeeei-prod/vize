import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const syncCommand = path.join(
  root,
  "tools",
  "commands",
  "editors",
  "vscode",
  "sync-typescript-plugin.rs",
);
const pluginFiles = [
  "component-contracts.cjs",
  "import-resolution.cjs",
  "index.cjs",
  "module-resolution.cjs",
  "package.json",
  "virtual-modules.cjs",
] as const;

test("TypeScript Vue plugin staging refuses partial source packages without touching target", () => {
  const fixture = createFixture();
  const source = path.join(fixture.root, "editors/vscode/typescript-vue-plugin");
  const target = path.join(
    fixture.root,
    "editors/vscode/node_modules/@vizejs/typescript-vue-plugin",
  );
  fs.mkdirSync(source, { recursive: true });
  fs.mkdirSync(target, { recursive: true });
  for (const file of pluginFiles.filter((file) => file !== "virtual-modules.cjs")) {
    fs.writeFileSync(path.join(source, file), `new ${file}\n`);
  }
  for (const file of pluginFiles) {
    fs.writeFileSync(path.join(target, file), `previous ${file}\n`);
  }

  try {
    const result = runStage(fixture.root);

    assert.equal(result.status, 1);
    assert.match(result.stderr, /virtual-modules\.cjs/);
    for (const file of pluginFiles) {
      assert.equal(fs.readFileSync(path.join(target, file), "utf8"), `previous ${file}\n`);
    }
  } finally {
    fs.rmSync(fixture.root, { force: true, recursive: true });
  }
});

test("TypeScript Vue plugin staging atomically replaces the target package", () => {
  const fixture = createFixture();
  const source = path.join(fixture.root, "editors/vscode/typescript-vue-plugin");
  const target = path.join(
    fixture.root,
    "editors/vscode/node_modules/@vizejs/typescript-vue-plugin",
  );
  fs.mkdirSync(source, { recursive: true });
  fs.mkdirSync(target, { recursive: true });
  for (const file of pluginFiles) {
    fs.writeFileSync(path.join(source, file), `new ${file}\n`);
    fs.writeFileSync(path.join(target, file), `previous ${file}\n`);
  }

  try {
    const result = runStage(fixture.root);

    assert.equal(result.status, 0, result.stderr);
    for (const file of pluginFiles) {
      assert.equal(fs.readFileSync(path.join(target, file), "utf8"), `new ${file}\n`);
    }
    assert.deepEqual(
      fs.readdirSync(path.dirname(target)).filter((entry) => entry.startsWith(".")),
      [],
    );
  } finally {
    fs.rmSync(fixture.root, { force: true, recursive: true });
  }
});

function createFixture(): { root: string } {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-typescript-plugin-stage-"));
  fs.writeFileSync(path.join(root, "Cargo.toml"), "[workspace]\n");
  fs.writeFileSync(path.join(root, "pnpm-workspace.yaml"), "packages: []\n");
  return { root };
}

function runStage(repoRoot: string): ReturnType<typeof spawnSync> {
  return spawnSync("rust-script", [syncCommand, "stage"], {
    cwd: repoRoot,
    encoding: "utf8",
    env: { ...process.env, VIZE_REPO_ROOT: repoRoot },
  });
}
