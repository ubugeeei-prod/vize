import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { writeFakeNpmRegistry } from "./support/fake-npm-registry.ts";

// Run the actual registry peer unchanged, but admit one complete transaction
// before the other. Both orders are legal for Promise.all visibility requests.
function orderRegistryProbes(binDir: string, first: "version" | "metadata"): void {
  const original = path.join(binDir, "original-curl");
  fs.renameSync(path.join(binDir, "curl"), original);
  writeFakeCommand(
    binDir,
    "curl",
    [
      "const fs = require('node:fs');",
      "const path = require('node:path');",
      "const { spawnSync } = require('node:child_process');",
      "const url = process.argv.at(-1);",
      "const probe = Number(new URL(url).searchParams.get('vize-publish-check').replace('staged-order-1-', ''));",
      "const role = url.includes('/1.2.3?') ? 'version' : 'metadata';",
      `const first = ${JSON.stringify(first)};`,
      "const marker = path.join(process.env.STAGED_ORDER_ROOT, probe + '-' + first);",
      "if (probe > 0 && role !== first && !fs.existsSync(marker)) {",
      "  await new Promise((resolve, reject) => {",
      "    const watcher = fs.watch(process.env.STAGED_ORDER_ROOT, () => { if (fs.existsSync(marker)) done(); });",
      "    const timer = setTimeout(() => { watcher.close(); reject(Error('registry peer did not finish')); }, 10000);",
      "    function done() { watcher.close(); clearTimeout(timer); resolve(); }",
      "    if (fs.existsSync(marker)) done();",
      "  });",
      "}",
      `const result = spawnSync(process.execPath, [${JSON.stringify(original)}, ...process.argv.slice(2)], { encoding: 'utf8' });`,
      "process.stdout.write(result.stdout ?? '');",
      "process.stderr.write(result.stderr ?? '');",
      "if (probe > 0 && role === first) fs.writeFileSync(marker, 'complete');",
      "process.exitCode = result.status ?? 1;",
    ].join("\n"),
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
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const state = fs.existsSync(process.env.VP_STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'))",
        "  : { publishCalls: 0, tagChecks: 0, versionChecks: 0 };",
        "function save() {",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'version') {",
        "  state.versionChecks += 1;",
        "  save();",
        "  if (state.publishCalls === 1 && state.versionChecks >= 3) {",
        "    process.stdout.write(JSON.stringify('1.2.3'));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'dist-tags') {",
        "  state.tagChecks += 1;",
        "  save();",
        "  if (state.publishCalls === 1) {",
        "    process.stdout.write(JSON.stringify({ latest: '1.2.3' }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'publish') {",
        "  state.publishCalls += 1;",
        "  save();",
        "  fs.appendFileSync(process.env.NPM_FAKE_URL_LOG, 'staged publish\\n');",
        "  process.stderr.write(",
        "    'Cannot publish over previously staged version \"1.2.3\".',",
        "  );",
        "  process.exit(1);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

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
