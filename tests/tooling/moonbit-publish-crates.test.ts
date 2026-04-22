import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";

type CargoMetadata = {
  packages: CargoPackage[];
};

type CargoPackage = {
  name: string;
  dependencies: CargoDependency[];
};

type CargoDependency = {
  kind: "dev" | "build" | null;
  name: string;
};

function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, `#!/usr/bin/env node\n${body}`);
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(path.join(binDir, `${name}.cmd`), `@echo off\r\nnode "%~dp0\\${name}" %*\r\n`);
  }
}

function getPublishedCrates(): string[] {
  const scriptPath = path.join(repoRoot, "tools", "moon", "scripts", "publish_crates.mbtx");
  const script = fs.readFileSync(scriptPath, "utf8");
  const arrayBody = script.match(
    /let published_crates\s*:\s*Array\[String\]\s*=\s*\[(?<body>[\s\S]*?)\n\]/m,
  )?.groups?.body;

  assert.ok(arrayBody, "Failed to locate publishedCrates in publish_crates.mbtx");
  return Array.from(arrayBody.matchAll(/"([^"]+)"/g), ([, crateName]) => crateName);
}

function getMetadata(): CargoMetadata {
  return JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as CargoMetadata;
}

test("publish_crates script keeps publishable workspace dependencies ordered", () => {
  const publishedCrates = getPublishedCrates();
  const publishOrder = new Map(publishedCrates.map((crateName, index) => [crateName, index]));
  const packages = new Map(getMetadata().packages.map((pkg) => [pkg.name, pkg]));

  for (const crateName of publishedCrates) {
    const pkg = packages.get(crateName);
    assert.ok(pkg, `Missing package metadata for ${crateName}`);

    for (const dependency of pkg.dependencies) {
      if (dependency.kind === "dev") {
        continue;
      }

      const dependencyOrder = publishOrder.get(dependency.name);
      if (dependencyOrder == null) {
        continue;
      }

      const crateOrder = publishOrder.get(crateName);
      assert.ok(crateOrder != null, `Missing publish order for ${crateName}`);
      assert.ok(
        dependencyOrder < crateOrder,
        `${crateName} must be published after ${dependency.name}`,
      );
    }
  }
});

test("publish_crates runs as a native MoonBit script", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-crates-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const fs = require('node:fs');",
        "fs.appendFileSync(process.env.CARGO_LOG, process.argv.slice(2).join(' ') + '\\n');",
        "const [command] = process.argv.slice(2);",
        "if (command === 'publish' || command === 'info') process.exit(0);",
        "process.exit(1);",
      ].join("\n"),
    );
    writeFakeCommand(
      binDir,
      "curl",
      ["process.stdout.write(JSON.stringify({ versions: [] }));", "process.exit(0);"].join("\n"),
    );

    const result = runMoonScript("publish_crates", [], {
      cwd: repoRoot,
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CARGO_LOG: cargoLogPath,
        PUBLISH_RETRY_LIMIT: "1",
        PUBLISH_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    const logLines = fs.readFileSync(cargoLogPath, "utf8").trim().split("\n");
    assert.match(logLines[0] ?? "", /^publish -p vize_carton$/);
    assert.match(logLines[1] ?? "", /^info vize_carton@/);
    assert.match(logLines.at(-2) ?? "", /^publish -p vize_fresco$/);
    assert.match(logLines.at(-1) ?? "", /^info vize_fresco@/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
