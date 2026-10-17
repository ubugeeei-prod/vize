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

test("canonical vite.config.ts loads through the public config API", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-ts-"));

  try {
    fs.writeFileSync(
      path.join(tempDir, "vite.config.ts"),
      [
        "type Command = 'serve' | 'build';",
        "",
        "export default ({ mode, command }: { mode: string; command: Command }) => ({",
        "  vue: { compiler: {",
        "    sourceMap: mode === 'production',",
        "  },",
        "  formatter: {",
        "    printWidth: command === 'build' ? 100 : 80,",
        "  },",
        "}});",
        "",
      ].join("\n"),
    );

    const config = await loadConfig(tempDir, {
      env: { command: "build", mode: "production" },
      mode: "root",
    });

    assert.equal(config?.compiler?.sourceMap, true);
    assert.equal(config?.formatter?.printWidth, 100);
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("missing explicit serialized PKL runtime returns null while Vite discovery remains available", async () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-config-pkl-missing-"));
  const binDir = path.join(tempDir, "bin");
  const warn = console.warn;

  try {
    fs.mkdirSync(binDir);
    fs.writeFileSync(path.join(tempDir, "serialized-config.pkl"), "invalid pkl");
    fs.writeFileSync(
      path.join(tempDir, "vite.config.mjs"),
      "export default {vue:{formatter:{printWidth:101}}};",
    );
    console.warn = () => {};

    const config = await withPath(binDir, () =>
      loadConfig(tempDir, { configFile: "serialized-config.pkl" }),
    );
    assert.equal(config, null, "an explicit missing PKL runtime cannot select the Vite sibling");
    assert.equal((await loadConfig(tempDir))?.formatter?.printWidth, 101);
  } finally {
    console.warn = warn;
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("explicit serialized PKL can amend the bundled schema without a project-level vize dependency", async () => {
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
      path.join(tempDir, "serialized-config.pkl"),
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
      loadConfig(tempDir, { configFile: "serialized-config.pkl" }),
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

test("explicit serialized PKL evaluation failures cannot fall through to a Vite sibling", async () => {
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
    fs.writeFileSync(path.join(tempDir, "serialized-config.pkl"), "invalid pkl");
    fs.writeFileSync(
      path.join(tempDir, "vite.config.mjs"),
      "export default {vue:{formatter:{printWidth:101}}};",
    );

    await assert.rejects(
      withPath(`${binDir}${path.delimiter}${process.env.PATH ?? ""}`, () =>
        loadConfig(tempDir, { configFile: "serialized-config.pkl" }),
      ),
      /Failed to evaluate vize PKL config/,
    );
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});
