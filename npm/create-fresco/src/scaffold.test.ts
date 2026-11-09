import assert from "node:assert/strict";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, test } from "node:test";

import { createFrescoProject } from "./scaffold.js";
import { CREATE_FRESCO_VERSION } from "./version.js";

const temporaryDirectories: string[] = [];

function temporaryDirectory(): string {
  const directory = mkdtempSync(join(tmpdir(), "create-fresco-test-"));
  temporaryDirectories.push(directory);
  return directory;
}

afterEach(() => {
  for (const directory of temporaryDirectories.splice(0)) {
    rmSync(directory, { recursive: true, force: true });
  }
});

void test("creates a deterministic SFC, Vite, and typecheck project", () => {
  const cwd = temporaryDirectory();
  const result = createFrescoProject({ directory: "My Fresco App", cwd, packageManager: "pnpm" });

  assert.equal(result.packageName, "my-fresco-app");
  assert.deepEqual(result.files, [
    ".gitignore",
    "README.md",
    "package.json",
    "tsconfig.json",
    "vite.config.ts",
    "src/App.vue",
    "src/env.d.ts",
    "src/main.ts",
  ]);
  for (const file of result.files)
    assert.equal(existsSync(join(result.targetDirectory, file)), true);

  const manifest = JSON.parse(
    readFileSync(join(result.targetDirectory, "package.json"), "utf8"),
  ) as {
    scripts: Record<string, string>;
    dependencies: Record<string, string>;
  };
  assert.deepEqual(manifest.scripts, {
    dev: "vite-node --watch src/main.ts",
    build: "vite build",
    start: "node dist/main.mjs",
    check: "vue-tsc --noEmit -p tsconfig.json",
  });
  assert.equal(manifest.dependencies["@vizejs/fresco"], `^${CREATE_FRESCO_VERSION}`);
  assert.doesNotMatch(JSON.stringify(manifest), /workspace:|catalog:/u);
  const appSource = readFileSync(join(result.targetDirectory, "src/App.vue"), "utf8");
  assert.match(appSource, /useApp/u);
  assert.match(appSource, /useInput/u);
  assert.match(
    readFileSync(join(result.targetDirectory, "src/main.ts"), "utf8"),
    /import\.meta\.hot\.dispose/u,
  );
  assert.match(
    readFileSync(join(result.targetDirectory, "vite.config.ts"), "utf8"),
    /ssr: "src\/main.ts"/u,
  );
});

void test("refuses non-empty targets unless force is explicit", () => {
  const cwd = temporaryDirectory();
  const target = join(cwd, "existing");
  createFrescoProject({ directory: "existing", cwd });
  writeFileSync(join(target, "user-owned.txt"), "keep", "utf8");

  assert.throws(
    () => createFrescoProject({ directory: "existing", cwd }),
    /CREATE_FRESCO_TARGET_NOT_EMPTY/u,
  );
  createFrescoProject({ directory: "existing", cwd, force: true });
  assert.equal(readFileSync(join(target, "user-owned.txt"), "utf8"), "keep");
});

void test("refuses filesystem roots even with force", () => {
  const root = process.platform === "win32" ? "C:\\" : "/";
  assert.throws(
    () => createFrescoProject({ directory: root, force: true }),
    /CREATE_FRESCO_UNSAFE_TARGET/u,
  );
});
