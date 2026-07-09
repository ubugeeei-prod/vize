import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import { mkdtempSync, rmSync } from "node:fs";

import { repoRoot, runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

type CargoMetadata = {
  packages: CargoPackage[];
};

type CargoPackage = {
  name: string;
  dependencies: CargoDependency[];
  manifest_path: string;
  publish: string[] | null;
};

type CargoDependency = {
  kind: "dev" | "build" | null;
  name: string;
};

function getScriptCrateArray(variableName: string): string[] {
  const scriptPath = path.join(repoRoot, "tools", "moon", "cmd", "publish_crates", "main.mbt");
  const script = fs.readFileSync(scriptPath, "utf8");
  const arrayBody = script.match(
    new RegExp(
      `let ${variableName}\\s*:\\s*Array\\[String\\]\\s*=\\s*\\[(?<body>[\\s\\S]*?)\\n\\]`,
      "m",
    ),
  )?.groups?.body;

  assert.ok(arrayBody, `Failed to locate ${variableName} in publish_crates`);
  return Array.from(arrayBody.matchAll(/"([^"]+)"/g), ([, crateName]) => crateName);
}

function getPublishedCrates(): string[] {
  return getScriptCrateArray("published_crates");
}

function getPendingFirstPublishCrates(): string[] {
  return getScriptCrateArray("pending_first_publish_crates");
}

function getBlockedByPendingFirstPublishCrates(): string[] {
  return getScriptCrateArray("blocked_by_pending_first_publish_crates");
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

test("publish_crates includes every publishable crate package after first publish exclusions", () => {
  const publishedCrates = new Set(getPublishedCrates());
  const pendingFirstPublishCrates = new Set(getPendingFirstPublishCrates());
  const blockedByPendingFirstPublishCrates = new Set(getBlockedByPendingFirstPublishCrates());
  const missingCrates = getMetadata()
    .packages.filter((pkg) =>
      path.relative(repoRoot, pkg.manifest_path).startsWith(`crates${path.sep}`),
    )
    .filter((pkg) => pkg.publish === null || pkg.publish.length > 0)
    .map((pkg) => pkg.name)
    .filter((crateName) => !publishedCrates.has(crateName))
    .filter((crateName) => !pendingFirstPublishCrates.has(crateName))
    .filter((crateName) => !blockedByPendingFirstPublishCrates.has(crateName));

  assert.deepEqual(missingCrates, []);
});

test("publish_crates only defers crates that have not been created on crates.io", () => {
  assert.deepEqual(getPendingFirstPublishCrates(), ["vize_croquis_cf", "vize_atelier_jsx"]);
});

test("publish_crates only blocks crates that depend on first-publish exclusions", () => {
  assert.deepEqual(getBlockedByPendingFirstPublishCrates(), ["vize_canon", "vize_patina"]);
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
    assert.match(logLines[1] ?? "", /^info --registry crates-io vize_carton@/);
    assert.match(logLines.at(-2) ?? "", /^publish -p vize_fresco$/);
    assert.match(logLines.at(-1) ?? "", /^info --registry crates-io vize_fresco@/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_crates treats a non-zero cargo publish exit as success when the crate is already resolvable", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-crates-resolvable-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "fs.appendFileSync(process.env.CARGO_LOG, args.join(' ') + '\\n');",
        "if (args[0] === 'publish' && args[2] === 'vize_carton') process.exit(1);",
        "if (args[0] === 'publish' || args[0] === 'info') process.exit(0);",
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
    assert.match(result.stdout, /already resolvable despite a non-zero cargo publish exit/i);
    const logLines = fs.readFileSync(cargoLogPath, "utf8").trim().split("\n");
    assert.equal(logLines[0], "publish -p vize_carton");
    assert.match(logLines[1] ?? "", /^info --registry crates-io vize_carton@/);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
