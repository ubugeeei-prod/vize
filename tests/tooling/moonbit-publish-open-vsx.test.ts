import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";
import { writeFakeCommand } from "./support/fake-command.ts";

test("publish_open_vsx_extension publishes a packaged VSIX with ovsx", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-open-vsx-publish-"));
  const binDir = path.join(tempDir, "bin");
  const manifestPath = path.join(tempDir, "package.json");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const statePath = path.join(tempDir, "state.json");
  const callsPath = path.join(tempDir, "calls.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(
      manifestPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFileSync(vsixPath, "vsix");
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const calls = fs.existsSync(process.env.CALLS_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.CALLS_PATH, 'utf8'))",
        "  : [];",
        "calls.push(args);",
        "fs.writeFileSync(process.env.CALLS_PATH, JSON.stringify(calls));",
        "const state = fs.existsSync(process.env.STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.STATE_PATH, 'utf8'))",
        "  : { published: false };",
        "if (args[0] === 'dlx' && args[2] === 'ovsx@^1.0.0' && args[3] === 'ovsx' && args[4] === 'get') {",
        "  if (state.published) {",
        "    process.stdout.write(JSON.stringify({ version: '0.57.0' }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'dlx' && args[2] === 'ovsx@^1.0.0' && args[3] === 'ovsx' && args[4] === 'create-namespace') {",
        "  process.exit(0);",
        "}",
        "if (args[0] === 'dlx' && args[2] === 'ovsx@^1.0.0' && args[3] === 'ovsx' && args[4] === 'publish') {",
        "  state.published = true;",
        "  fs.writeFileSync(process.env.STATE_PATH, JSON.stringify(state));",
        "  process.exit(0);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_open_vsx_extension", [vsixPath, manifestPath], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CALLS_PATH: callsPath,
        STATE_PATH: statePath,
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());

    const calls = JSON.parse(fs.readFileSync(callsPath, "utf8")) as string[][];
    assert.deepEqual(calls.at(-2), [
      "dlx",
      "-p",
      "ovsx@^1.0.0",
      "ovsx",
      "create-namespace",
      "ubugeeei",
    ]);
    assert.deepEqual(calls.at(-1), ["dlx", "-p", "ovsx@^1.0.0", "ovsx", "publish", vsixPath]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_open_vsx_extension skips an already visible Open VSX version", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-open-vsx-skip-"));
  const binDir = path.join(tempDir, "bin");
  const manifestPath = path.join(tempDir, "package.json");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const callsPath = path.join(tempDir, "calls.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(
      manifestPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFileSync(vsixPath, "vsix");
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const calls = fs.existsSync(process.env.CALLS_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.CALLS_PATH, 'utf8'))",
        "  : [];",
        "calls.push(args);",
        "fs.writeFileSync(process.env.CALLS_PATH, JSON.stringify(calls));",
        "if (args[0] === 'dlx' && args[2] === 'ovsx@^1.0.0' && args[3] === 'ovsx' && args[4] === 'get') {",
        "  process.stdout.write(JSON.stringify({ version: '0.57.0' }));",
        "  process.exit(0);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_open_vsx_extension", [vsixPath, manifestPath], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        CALLS_PATH: callsPath,
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already published/);

    const calls = JSON.parse(fs.readFileSync(callsPath, "utf8")) as string[][];
    assert.equal(calls.length, 1);
    assert.equal(calls[0]?.[4], "get");
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
