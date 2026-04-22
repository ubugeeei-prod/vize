import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { test } from "node:test";

import { runMoonScript } from "./_helpers/moonbit.ts";

function writeFakeCommand(binDir: string, name: string, body: string): void {
  const unixPath = path.join(binDir, name);
  writeFileSync(unixPath, `#!/usr/bin/env node\n${body}`);
  fs.chmodSync(unixPath, 0o755);

  if (process.platform === "win32") {
    writeFileSync(path.join(binDir, `${name}.cmd`), `@echo off\r\nnode "%~dp0\\${name}" %*\r\n`);
  }
}

test("npm_tag script maps prerelease versions to npm dist-tags", () => {
  const cases = [
    ["1.2.3-alpha.1", "alpha"],
    ["1.2.3-beta.1", "beta"],
    ["1.2.3-rc.1", "rc"],
    ["1.2.3", "latest"],
  ] as const;

  for (const [version, expected] of cases) {
    const result = runMoonScript("npm_tag", [version]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(result.stdout.trim(), expected);
  }
});

test("inject_native_optional_deps updates only native optional dependency pins", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-inject-native-"));
  const targetPath = path.join(tempDir, "package.json");
  const versionPath = path.join(tempDir, "version-package.json");

  try {
    writeFileSync(
      targetPath,
      `${JSON.stringify(
        {
          name: "@vizejs/example",
          version: "0.0.1",
          optionalDependencies: {
            "@vizejs/native-linux-x64-gnu": "0.0.1",
            "@vizejs/native-darwin-arm64": "0.0.1",
            fsevents: "^2.3.3",
          },
        },
        null,
        2,
      )}\n`,
    );
    writeFileSync(versionPath, `${JSON.stringify({ version: "1.2.3-beta.1" }, null, 2)}\n`);

    const result = runMoonScript("inject_native_optional_deps", [targetPath, versionPath]);
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());

    const updated = JSON.parse(fs.readFileSync(targetPath, "utf8")) as {
      optionalDependencies: Record<string, string>;
    };
    assert.deepEqual(updated.optionalDependencies, {
      "@vizejs/native-linux-x64-gnu": "1.2.3-beta.1",
      "@vizejs/native-darwin-arm64": "1.2.3-beta.1",
      fsevents: "^2.3.3",
    });
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_npm_package computes the tag and forwards provenance to vp", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-npm-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");
  const cwdLogPath = path.join(tempDir, "vp-cwd.log");
  const statePath = path.join(tempDir, "vp-state.json");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
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
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_npm_package skips publish when the version is already visible in npm", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-skip-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
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
        "fs.appendFileSync(process.env.VP_ARGS_LOG, `${args.join(' ')}\\n`);",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'version') {",
        "  process.stdout.write(JSON.stringify('1.2.3-beta.1'));",
        "  process.exit(0);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'dist-tags') {",
        "  process.stdout.write(JSON.stringify({ beta: '1.2.3-beta.1' }));",
        "  process.exit(0);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'publish') process.exit(99);",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_npm_package", [packageDir, "--provenance"], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_ARGS_LOG: argsLogPath,
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already published/i);
    const loggedArgs = fs.readFileSync(argsLogPath, "utf8").trim().split("\n");
    assert.equal(
      loggedArgs.some((line) => line.includes("pm publish")),
      false,
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_npm_package treats a non-zero publish exit as success when npm already has the version", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-recover-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const statePath = path.join(tempDir, "vp-state.json");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(
      path.join(packageDir, "package.json"),
      `${JSON.stringify({ name: "@vizejs/example", version: "1.2.3" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const state = fs.existsSync(process.env.VP_STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'))",
        "  : { publishCalls: 0, versionVisible: false };",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'version') {",
        "  if (state.versionVisible) {",
        "    process.stdout.write(JSON.stringify('1.2.3'));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'dist-tags') {",
        "  if (state.versionVisible) {",
        "    process.stdout.write(JSON.stringify({ latest: '1.2.3' }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'publish') {",
        "  state.publishCalls += 1;",
        "  state.versionVisible = true;",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "  process.exit(1);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_npm_package", [packageDir], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_STATE_PATH: statePath,
        PUBLISH_RETRY_LIMIT: "1",
        PUBLISH_RETRY_DELAY: "1",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "2",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /appears in npm despite a non-zero publish exit/i);
    const state = JSON.parse(fs.readFileSync(statePath, "utf8")) as {
      publishCalls: number;
      versionVisible: boolean;
    };
    assert.equal(state.publishCalls, 1);
    assert.equal(state.versionVisible, true);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_npm_package_dirs publishes only subdirectories that contain package.json", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-dirs-"));
  const baseDir = path.join(tempDir, "packages");
  const binDir = path.join(tempDir, "bin");
  const statePath = path.join(tempDir, "vp-state.json");

  try {
    fs.mkdirSync(path.join(baseDir, "pkg-a"), { recursive: true });
    fs.mkdirSync(path.join(baseDir, "skip-me"), { recursive: true });
    fs.mkdirSync(path.join(baseDir, "pkg-c"), { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(
      path.join(baseDir, "pkg-a", "package.json"),
      `${JSON.stringify({ name: "@vizejs/pkg-a", version: "1.0.0" }, null, 2)}\n`,
    );
    writeFileSync(
      path.join(baseDir, "pkg-c", "package.json"),
      `${JSON.stringify({ name: "@vizejs/pkg-c", version: "1.0.0" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const path = require('node:path');",
        "const args = process.argv.slice(2);",
        "const state = fs.existsSync(process.env.VP_STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'))",
        "  : { published: {}, order: [] };",
        "const packageJson = path.join(process.cwd(), 'package.json');",
        "if (args[0] === 'pm' && args[1] === 'publish') {",
        "  const pkg = JSON.parse(fs.readFileSync(packageJson, 'utf8'));",
        "  state.published[pkg.name] = pkg.version;",
        "  state.order.push(pkg.name);",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "  process.exit(0);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'version') {",
        "  const spec = args[2];",
        "  const splitAt = spec.lastIndexOf('@');",
        "  const name = spec.slice(0, splitAt);",
        "  const version = state.published[name];",
        "  if (version) {",
        "    process.stdout.write(JSON.stringify(version));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'dist-tags') {",
        "  const name = args[2];",
        "  const version = state.published[name];",
        "  if (version) {",
        "    process.stdout.write(JSON.stringify({ latest: version }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_npm_package_dirs", [baseDir], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_STATE_PATH: statePath,
        PUBLISH_RETRY_LIMIT: "1",
        PUBLISH_RETRY_DELAY: "1",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    const state = JSON.parse(fs.readFileSync(statePath, "utf8")) as {
      order: string[];
      published: Record<string, string>;
    };
    assert.deepEqual(state.order, ["@vizejs/pkg-a", "@vizejs/pkg-c"]);
    assert.deepEqual(Object.keys(state.published).sort(), ["@vizejs/pkg-a", "@vizejs/pkg-c"]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_vscode_extension adds pre-release when NPM_TAG is not latest", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "moonbit-publish-vsix-"));
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");
  const vsixPath = path.join(tempDir, "vize.vsix");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "fs.writeFileSync(process.env.VP_ARGS_LOG, process.argv.slice(2).join('\\n'));",
        "process.exit(0);",
      ].join("\n"),
    );

    const result = runMoonScript("publish_vscode_extension", [vsixPath], {
      env: {
        NPM_TAG: "rc",
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_ARGS_LOG: argsLogPath,
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.deepEqual(fs.readFileSync(argsLogPath, "utf8").trim().split("\n"), [
      "dlx",
      "-p",
      "@vscode/vsce@^3.3.2",
      "vsce",
      "publish",
      "--no-dependencies",
      "--packagePath",
      vsixPath,
      "--pre-release",
    ]);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
