import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const smokeScript = path.join(root, "tools/npm/smoke-release-install.mjs");

test("release install smoke packs and installs local package tarballs", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  try {
    const dependencyDir = writePackage(tempDir, "dependency", {
      name: "@vizejs/smoke-dependency",
      version: "1.0.0",
    });
    const packageDir = writePackage(tempDir, "package", {
      bin: {
        "smoke-bin": "cli.js",
      },
      dependencies: {
        "@vizejs/smoke-dependency": "1.0.0",
      },
      name: "@vizejs/smoke-package",
      version: "1.0.0",
    });
    fs.writeFileSync(path.join(packageDir, "cli.js"), "#!/usr/bin/env node\n");
    fs.chmodSync(path.join(packageDir, "cli.js"), 0o755);

    const result = runSmoke(packageDir, dependencyDir);

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /smoked 2\/2 package tarballs/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("release install smoke rejects unresolved workspace protocols", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-release-smoke-test-"));
  try {
    const packageDir = writePackage(tempDir, "package", {
      dependencies: {
        "@vizejs/smoke-dependency": "workspace:*",
      },
      name: "@vizejs/smoke-package",
      version: "1.0.0",
    });

    const result = runSmoke(packageDir);

    assert.notEqual(result.status, 0);
    assert.match(`${result.stderr}\n${result.stdout}`, /workspace:\*/);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

function writePackage(tempDir: string, dirName: string, manifest: Record<string, unknown>): string {
  const packageDir = path.join(tempDir, dirName);
  fs.mkdirSync(packageDir, { recursive: true });
  fs.writeFileSync(path.join(packageDir, "index.js"), "export const ok = true;\n");
  fs.writeFileSync(path.join(packageDir, "index.d.ts"), "export declare const ok: true;\n");
  fs.writeFileSync(
    path.join(packageDir, "package.json"),
    JSON.stringify(
      {
        exports: {
          ".": {
            import: "./index.js",
            types: "./index.d.ts",
          },
        },
        files: ["index.js", "index.d.ts", "cli.js"],
        main: "./index.js",
        types: "./index.d.ts",
        ...manifest,
      },
      null,
      2,
    ),
  );
  return packageDir;
}

function runSmoke(...packageDirs: string[]): {
  status: number | null;
  stderr: string;
  stdout: string;
} {
  const result = spawnSync(process.execPath, [smokeScript, ...packageDirs], {
    cwd: root,
    encoding: "utf8",
  });

  if (result.error != null) {
    throw result.error;
  }

  return {
    status: result.status,
    stderr: result.stderr,
    stdout: result.stdout,
  };
}
