import assert from "node:assert/strict";
import fs from "node:fs";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { writeFakeNpmRegistry } from "./support/fake-npm-registry.ts";

test("publish_npm_package computes the tag and forwards provenance to vp", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-npm-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");
  const cwdLogPath = path.join(tempDir, "vp-cwd.log");
  const registryUrlLogPath = path.join(tempDir, "registry-urls.log");
  const statePath = path.join(tempDir, "vp-state.json");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeNpmRegistry(binDir);
    writeFileSync(
      path.join(packageDir, "package.json"),
      `${JSON.stringify({ name: "@vizejs/example", version: "1.2.3-beta.1" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const state = fs.existsSync(process.env.VP_STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'))",
        "  : { published: false };",
        "if (args[0] === 'pm' && args[1] === 'publish') {",
        "  fs.writeFileSync(process.env.VP_ARGS_LOG, args.join('\\n'));",
        "  fs.writeFileSync(process.env.VP_CWD_LOG, process.cwd());",
        "  state.published = true;",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "  process.exit(0);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'version') {",
        "  if (state.published) {",
        "    process.stdout.write(JSON.stringify('1.2.3-beta.1'));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'dist-tags') {",
        "  if (state.published) {",
        "    process.stdout.write(JSON.stringify({ beta: '1.2.3-beta.1' }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_npm_package", [packageDir, "--provenance"], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_ARGS_LOG: argsLogPath,
        VP_CWD_LOG: cwdLogPath,
        VP_STATE_PATH: statePath,
        NPM_FAKE_STATE_PATH: statePath,
        NPM_FAKE_VERSION: "1.2.3-beta.1",
        NPM_FAKE_URL_LOG: registryUrlLogPath,
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.deepEqual(fs.readFileSync(argsLogPath, "utf8").trim().split("\n"), [
      "pm",
      "publish",
      "--access",
      "public",
      "--no-git-checks",
      "--tag",
      "beta",
      "--",
      "--provenance",
    ]);
    assert.equal(fs.realpathSync(fs.readFileSync(cwdLogPath, "utf8")), fs.realpathSync(packageDir));
    const registryUrls = fs
      .readFileSync(registryUrlLogPath, "utf8")
      .trim()
      .split("\n")
      .map((url) => new URL(url));
    assert.deepEqual(
      registryUrls.map((url) => url.pathname),
      [
        "/%40vizejs%2Fexample/1.2.3-beta.1",
        "/%40vizejs%2Fexample/1.2.3-beta.1",
        "/%40vizejs%2Fexample",
      ],
    );
    assert.notEqual(
      registryUrls[0].searchParams.get("vize-publish-check"),
      registryUrls[1].searchParams.get("vize-publish-check"),
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
