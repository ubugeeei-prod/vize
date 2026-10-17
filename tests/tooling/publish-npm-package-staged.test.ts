import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { test } from "node:test";

import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { writeFakeNpmRegistry } from "./support/fake-npm-registry.ts";

test("publish_npm_package waits instead of retrying a hidden staged npm publish", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-staged-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const statePath = path.join(tempDir, "vp-state.json");
  const registryUrlLogPath = path.join(tempDir, "registry-urls.log");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeNpmRegistry(binDir);
    writeFileSync(
      path.join(packageDir, "package.json"),
      `${JSON.stringify({ name: "@vizejs/example", version: "1.2.3" }, null, 2)}\n`,
    );
    writeFileSync(statePath, JSON.stringify({ publishCalls: 0 }));
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const state = fs.existsSync(process.env.VP_STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'))",
        "  : { publishCalls: 0 };",
        "function save() {",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "}",
        "if (args[0] === 'pm' && args[1] === 'publish') {",
        "  state.publishCalls += 1;",
        "  save();",
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
        NPM_FAKE_URL_LOG: registryUrlLogPath,
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
    };
    assert.deepEqual(state, { publishCalls: 1 }, "registry probes never rewrite publisher state");
    const requests = fs
      .readFileSync(registryUrlLogPath, "utf8")
      .trim()
      .split("\n")
      .map((url) => new URL(url).pathname);
    assert.equal(requests.filter((url) => url === "/%40vizejs%2Fexample").length, 1);
    assert.equal(requests.filter((url) => url === "/%40vizejs%2Fexample/1.2.3").length, 3);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
