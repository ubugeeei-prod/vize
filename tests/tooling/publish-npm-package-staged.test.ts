import { readToolingProgram } from "./support/program-path.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { fakeRegistryArchivePath, writeFakeNpmRegistry } from "./support/fake-npm-registry.ts";

// Run the actual registry peer unchanged, but admit one complete transaction
// before the other. Both orders are legal for Promise.all visibility requests.
function orderRegistryProbes(binDir: string, first: "version" | "metadata"): void {
  const original = path.join(binDir, "original-curl");
  fs.renameSync(path.join(binDir, "curl"), original);
  writeFakeCommand(
    binDir,
    "curl",
    readToolingProgram("commands/publish-npm-package-staged/curl.js.in", {
      ARCHIVE_MODULE: JSON.stringify(fakeRegistryArchivePath),
      FIRST: JSON.stringify(first),
      ORIGINAL: JSON.stringify(original),
    }),
  );
}

function assertStagedPublish(first: "version" | "metadata"): void {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-staged-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const statePath = path.join(tempDir, "vp-state.json");
  const tracePath = path.join(tempDir, "registry-trace.txt");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeNpmRegistry(binDir);
    orderRegistryProbes(binDir, first);
    writeFileSync(
      path.join(packageDir, "package.json"),
      `${JSON.stringify({ name: "@vizejs/example", version: "1.2.3" }, null, 2)}\n`,
    );
    writeFileSync(statePath, JSON.stringify({ publishCalls: 0, tagChecks: 0, versionChecks: 0 }));
    writeFakeCommand(binDir, "vp", readToolingProgram("commands/publish-npm-package-staged/vp.js"));

    const result = runReleaseCommand("publish_npm_package", [packageDir], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_STATE_PATH: statePath,
        NPM_FAKE_STATE_PATH: statePath,
        NPM_FAKE_VERSION: "1.2.3",
        NPM_FAKE_URL_LOG: tracePath,
        STAGED_ORDER_ROOT: tempDir,
        GITHUB_RUN_ID: "staged-order",
        GITHUB_RUN_ATTEMPT: "1",
        PUBLISH_RETRY_LIMIT: "6",
        PUBLISH_RETRY_DELAY: "1",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "3",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /staged in npm but not yet visible/i);
    assert.match(result.stdout, /visible in npm with dist-tag latest/i);
    const state = JSON.parse(fs.readFileSync(statePath, "utf8")) as {
      publishCalls: number;
      tagChecks: number;
      versionChecks: number;
    };
    assert.equal(state.publishCalls, 1);
    assert.equal(state.tagChecks, 1);
    assert.equal(state.versionChecks, 3);
    const versionUrl = "https://registry.npmjs.org/%40vizejs%2Fexample/1.2.3";
    const metadataUrl = "https://registry.npmjs.org/%40vizejs%2Fexample";
    const query = "?vize-publish-check=staged-order-1-";
    const versionProbe = `${versionUrl}${query}1`;
    const metadataProbe = `${metadataUrl}${query}1`;
    assert.deepEqual(fs.readFileSync(tracePath, "utf8").trim().split("\n"), [
      `${versionUrl}${query}0`,
      "staged publish",
      `${versionUrl}${query}-1`,
      ...(first === "version" ? [versionProbe, metadataProbe] : [metadataProbe, versionProbe]),
      "https://registry.npmjs.org/@vizejs/example/-/fixture-1.2.3.tgz",
    ]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
}

test("publish_npm_package waits instead of retrying a hidden staged npm publish", () => {
  assertStagedPublish("version");
});

test("publish_npm_package preserves staged visibility when metadata responds first", () => {
  assertStagedPublish("metadata");
});
