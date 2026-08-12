import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { assertInstalledMapperContract } from "../../tools/npm/smoke-release-runtime.mjs";

function withInstalledMapper(run: (installDir: string, packageRoot: string) => void): void {
  const installDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-mapper-contract-"));
  const packageRoot = path.join(installDir, "node_modules", "vize");
  const mapperPath = path.join(packageRoot, "bin", "vize");
  try {
    fs.mkdirSync(path.dirname(mapperPath), { recursive: true });
    fs.writeFileSync(
      path.join(packageRoot, "package.json"),
      JSON.stringify({
        name: "vize",
        tsContentMapper: {
          exec: ["node", "./bin/vize", "content-mapper"],
          extensions: { ".vue": ".tsx" },
          compilerOptions: ["noUnusedLocals"],
        },
      }),
    );
    fs.writeFileSync(mapperPath, "#!/usr/bin/env node\n");
    fs.chmodSync(mapperPath, 0o755);
    run(installDir, packageRoot);
  } finally {
    fs.rmSync(installDir, { force: true, recursive: true });
  }
}

test("installed Content Mapper contract accepts the production package shape", () => {
  withInstalledMapper((installDir) => assertInstalledMapperContract(installDir));
});

test("installed Content Mapper contract rejects a corrupted exec entry", () => {
  withInstalledMapper((installDir, packageRoot) => {
    const manifestPath = path.join(packageRoot, "package.json");
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    manifest.tsContentMapper.exec = ["node", "./bin/missing", "content-mapper"];
    fs.writeFileSync(manifestPath, JSON.stringify(manifest));

    assert.throws(
      () => assertInstalledMapperContract(installDir),
      /must expose the production tsContentMapper contract/,
    );
  });
});

test("installed Content Mapper contract rejects a missing virtual extension", () => {
  withInstalledMapper((installDir, packageRoot) => {
    const manifestPath = path.join(packageRoot, "package.json");
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));
    delete manifest.tsContentMapper.extensions;
    fs.writeFileSync(manifestPath, JSON.stringify(manifest));

    assert.throws(
      () => assertInstalledMapperContract(installDir),
      /must expose the production tsContentMapper contract/,
    );
  });
});

test("installed Content Mapper contract rejects a missing executable", () => {
  withInstalledMapper((installDir, packageRoot) => {
    fs.rmSync(path.join(packageRoot, "bin", "vize"));
    assert.throws(() => assertInstalledMapperContract(installDir), /bin[/\\]vize must exist/);
  });
});
