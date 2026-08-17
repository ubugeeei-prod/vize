import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("vize package exposes an executable TypeScript content mapper", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf-8"),
  ) as {
    bin?: Record<string, string>;
    files?: string[];
    tsContentMapper?: unknown;
    typescript?: {
      contentMapper?: {
        compilerOptions?: string[];
        exec?: string[];
      };
    };
  };
  const binPath = path.join(root, "npm/cli/bin/vize");

  assert.deepEqual(packageJson.typescript?.contentMapper, {
    exec: ["node", "./bin/vize", "content-mapper"],
    compilerOptions: ["noUnusedLocals"],
  });
  assert.equal(
    packageJson.tsContentMapper,
    undefined,
    "the retired tsContentMapper key must not ship alongside typescript.contentMapper",
  );
  assert.equal(packageJson.bin?.vize, "bin/vize");
  assert.ok(packageJson.files?.includes("bin"));
  assert.notEqual(
    fs.statSync(binPath).mode & 0o111,
    0,
    "content mapper entrypoint must be executable",
  );
  assert.match(fs.readFileSync(binPath, "utf-8"), /^#!\/usr\/bin\/env node\n/);
});

test("VS Code discovers Vize through the trusted native preview command", () => {
  const discovery = fs.readFileSync(
    path.join(root, "editors/vscode/src/content-mapper-discovery.ts"),
    "utf-8",
  );

  assert.match(discovery, /extensions\.getExtension\("TypeScriptTeam\.native-preview"\)/);
  assert.match(
    discovery,
    /commands\.executeCommand\(\s*"typescript\.native-preview\.discoverContentMappers"/,
  );
  assert.match(discovery, /workspace\.isTrusted/);
  assert.match(discovery, /extensions: \["\.vue"\]/);
});
