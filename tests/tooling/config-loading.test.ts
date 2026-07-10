import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { loadConfig } from "../../npm/cli/src/config.ts";

async function withPath<T>(value: string, run: () => Promise<T>): Promise<T> {
  const previous = process.env.PATH;
  process.env.PATH = value;
  try {
    return await run();
  } finally {
    if (previous === undefined) {
      delete process.env.PATH;
    } else {
      process.env.PATH = previous;
    }
  }
}

function writeFakePkl(binDir: string, body: string): void {
  const pklPath = path.join(binDir, process.platform === "win32" ? "pkl.cmd" : "pkl");
  fs.writeFileSync(pklPath, body);
  fs.chmodSync(pklPath, 0o755);
}

test("vize.config.ts loads through the public config API", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-ts-"));

  try {
    fs.writeFileSync(
      path.join(tempDir, "vize.config.ts"),
      [
        "type Command = 'serve' | 'build';",
        "",
        "export default ({ mode, command }: { mode: string; command: Command }) => ({",
        "  compiler: {",
        "    sourceMap: mode === 'production',",
        "  },",
        "  formatter: {",
        "    lineWidth: command === 'build' ? 100 : 80,",
        "  },",
        "});",
        "",
      ].join("\n"),
    );

    const config = await loadConfig(tempDir, {
      env: { command: "build", mode: "production" },
      mode: "root",
    });

    assert.equal(config?.compiler?.sourceMap, true);
    assert.equal(config?.formatter?.lineWidth, 100);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("missing pkl runtime falls back to the next config format", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-pkl-missing-"));
  const binDir = path.join(tempDir, "bin");
  const warn = console.warn;

  try {
    fs.mkdirSync(binDir);
    fs.writeFileSync(path.join(tempDir, "vize.config.pkl"), "invalid pkl");
    fs.writeFileSync(
      path.join(tempDir, "vize.config.json"),
      '{ "formatter": { "lineWidth": 101 } }',
    );
    console.warn = () => {};

    const config = await withPath(binDir, () => loadConfig(tempDir, { mode: "root" }));

    assert.equal(config?.formatter?.lineWidth, 101);
  } finally {
    console.warn = warn;
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("pkl config can amend the bundled schema without a project-level vize dependency", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-pkl-schema-"));
  const binDir = path.join(tempDir, "bin");

  try {
    fs.mkdirSync(binDir);
    writeFakePkl(
      binDir,
      [
        "#!/usr/bin/env node",
        "const fs = require('node:fs');",
        "if (process.argv.includes('--version')) process.exit(0);",
        "const modulePath = process.argv[process.argv.length - 1];",
        "const source = fs.readFileSync(modulePath, 'utf-8');",
        "if (!source.includes('file://') || !source.includes('/pkl/vize.pkl')) {",
        "  process.stderr.write(source);",
        "  process.exit(1);",
        "}",
        "process.stdout.write(JSON.stringify({",
        "  compiler: { sourceMap: true },",
        "  vite: { scanPatterns: ['app/**/*.vue'] },",
        "}));",
        "",
      ].join("\n"),
    );
    fs.writeFileSync(
      path.join(tempDir, "vize.config.pkl"),
      [
        'amends "node_modules/vize/pkl/vize.pkl"',
        "",
        "compiler {",
        "  sourceMap = true",
        "}",
        "",
        "vite {",
        "  scanPatterns = new Listing {",
        '    "app/**/*.vue"',
        "  }",
        "}",
        "",
      ].join("\n"),
    );

    const config = await withPath(`${binDir}${path.delimiter}${process.env.PATH ?? ""}`, () =>
      loadConfig(tempDir, { mode: "root" }),
    );

    assert.equal(config?.compiler?.sourceMap, true);
    assert.deepEqual(config?.vite?.scanPatterns, ["app/**/*.vue"]);
    assert.equal(
      fs.existsSync(path.join(tempDir, "node_modules", "vize", "pkl", "vize.pkl")),
      false,
    );
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("pkl evaluation failures stop lower-priority config fallback", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-pkl-failure-"));
  const binDir = path.join(tempDir, "bin");

  try {
    fs.mkdirSync(binDir);
    writeFakePkl(
      binDir,
      [
        "#!/usr/bin/env node",
        "if (process.argv.includes('--version')) process.exit(0);",
        "process.stderr.write('synthetic pkl failure');",
        "process.exit(1);",
        "",
      ].join("\n"),
    );
    fs.writeFileSync(path.join(tempDir, "vize.config.pkl"), "invalid pkl");
    fs.writeFileSync(
      path.join(tempDir, "vize.config.json"),
      '{ "formatter": { "lineWidth": 101 } }',
    );

    await assert.rejects(
      withPath(`${binDir}${path.delimiter}${process.env.PATH ?? ""}`, () =>
        loadConfig(tempDir, { mode: "root" }),
      ),
      /Failed to evaluate vize PKL config/,
    );
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
