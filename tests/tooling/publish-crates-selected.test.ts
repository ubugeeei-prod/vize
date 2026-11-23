import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs, { mkdtempSync, rmSync } from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { repoRoot, runReleaseCommand } from "./support/release-command.ts";
import { assertPublishManifestIsSanitized } from "./support/fake-cargo-publish.ts";
import { getMetadata } from "./support/publish-crates-plan.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

function workspaceVersion(): string {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: repoRoot,
      encoding: "utf8",
    }),
  ) as { packages: Array<{ name: string; version: string }> };
  return metadata.packages.find((pkg) => pkg.name === "vize_atelier_jsx")?.version ?? "";
}

test("publish_crates can target the JSX and Patina handoff set", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-crates-selected-"));
  const binDir = path.join(tempDir, "bin");
  const cargoLogPath = path.join(tempDir, "cargo.log");
  const curlLogPath = path.join(tempDir, "curl.log");
  const selectedCrates = ["vize_atelier_jsx", "vize_patina"];
  const version = workspaceVersion();
  assert.ok(version);
  const manifestSnapshots = new Map(
    selectedCrates.map((crateName) => {
      const manifestPath = getMetadata().packages.find(
        (pkg) => pkg.name === crateName,
      )!.manifest_path;
      return [manifestPath, fs.readFileSync(manifestPath, "utf8")] as const;
    }),
  );

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeCommand(
      binDir,
      "cargo",
      readToolingProgram("commands/publish-crates/cargo-selected-crates.js.in", {
        CHECK_MANIFEST: assertPublishManifestIsSanitized,
      }),
    );
    writeFakeCommand(
      binDir,
      "curl",
      readToolingProgram("commands/publish-crates-selected/curl.js"),
    );

    const env = {
      PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
      CARGO_LOG: cargoLogPath,
      PUBLISH_RETRY_LIMIT: "1",
      PUBLISH_RETRY_DELAY: "1",
    };
    const selectedPublish = runReleaseCommand(
      "publish_crates",
      ["--crate", "vize_atelier_jsx", "--crate", "vize_patina"],
      { cwd: repoRoot, env },
    );
    assert.equal(selectedPublish.status, 0, selectedPublish.stderr);
    assert.match(
      selectedPublish.stdout,
      /Selected crate publish plan: vize_atelier_jsx, vize_patina/,
    );
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      "publish --allow-dirty --no-verify -p vize_atelier_jsx",
      `info --registry crates-io vize_atelier_jsx@${version}`,
      "publish --allow-dirty --no-verify -p vize_patina",
      `info --registry crates-io vize_patina@${version}`,
    ]);

    fs.writeFileSync(cargoLogPath, "");
    fs.writeFileSync(curlLogPath, "");
    const selectedDryRun = runReleaseCommand(
      "publish_crates",
      ["--dry-run", "--crate", "vize_atelier_jsx", "--crate", "vize_patina"],
      {
        cwd: repoRoot,
        env: {
          ...env,
          CURL_LOG: curlLogPath,
          TEST_PUBLISHED_CRATES: "vize_atelier_jsx",
        },
      },
    );
    assert.equal(selectedDryRun.status, 0, selectedDryRun.stderr);
    assert.deepEqual(fs.readFileSync(cargoLogPath, "utf8").trim().split("\n"), [
      `info --registry crates-io vize_atelier_jsx@${version}`,
      "publish --dry-run --allow-dirty --no-verify -p vize_patina",
    ]);
    for (const [manifestPath, original] of manifestSnapshots) {
      assert.equal(fs.readFileSync(manifestPath, "utf8"), original);
    }
    assert.match(selectedDryRun.stdout, /registry-resolvable frontier vize_patina/i);

    for (const invalidArgs of [
      ["--crate"],
      ["--crate", "unknown_crate"],
      ["--crate", "vize_patina", "--crate", "vize_patina"],
    ]) {
      fs.writeFileSync(cargoLogPath, "");
      const invalid = runReleaseCommand("publish_crates", invalidArgs, {
        cwd: repoRoot,
        env,
      });
      assert.notEqual(invalid.status, 0);
      assert.match(invalid.stderr, /Usage: .*publish_crates.*--crate <crate>/);
      assert.equal(fs.readFileSync(cargoLogPath, "utf8"), "");
    }
  } finally {
    for (const [manifestPath, original] of manifestSnapshots) {
      fs.writeFileSync(manifestPath, original);
    }
    rmSync(tempDir, { recursive: true, force: true });
  }
});
