import assert from "node:assert/strict";
import fs from "node:fs";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { runReleaseCommand } from "./support/release-command.ts";
import { writeFakeCommand } from "./support/fake-command.ts";
import { writeFakeNpmRegistry } from "./support/fake-npm-registry.ts";

test("publish_npm_package normalizes workspace and catalog dependency specs before publishing", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-normalize-"));
  const repoDir = path.join(tempDir, "repo");
  const packageDir = path.join(repoDir, "npm", "builder/vite");
  const binDir = path.join(tempDir, "bin");
  const statePath = path.join(tempDir, "vp-state.json");
  const manifestLogPath = path.join(tempDir, "manifest.json");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(path.join(repoDir, "npm", "native"), { recursive: true });
    fs.mkdirSync(path.join(repoDir, "npm", "cli"), { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeNpmRegistry(binDir);
    writeFileSync(
      path.join(repoDir, "pnpm-workspace.yaml"),
      [
        "packages:",
        '  - "npm/*"',
        "",
        "catalogs:",
        "  repo-tooling:",
        '    tinyglobby: "0.2.16"',
        "  vite-stack:",
        '    vite: "npm:@voidzero-dev/vite-plus-core@0.1.19"',
        "",
      ].join("\n"),
    );
    writeFileSync(
      path.join(repoDir, "npm", "native", "package.json"),
      `${JSON.stringify({ name: "@vizejs/native", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFileSync(
      path.join(repoDir, "npm", "cli", "package.json"),
      `${JSON.stringify({ name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFileSync(
      path.join(packageDir, "package.json"),
      `${JSON.stringify(
        {
          name: "@vizejs/vite-plugin",
          version: "0.57.0",
          dependencies: {
            "@vizejs/native": "workspace:*",
            tinyglobby: "catalog:repo-tooling",
            vize: "workspace:*",
          },
          peerDependencies: {
            vite: "catalog:vite-stack",
          },
        },
        null,
        2,
      )}\n`,
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
        "  fs.writeFileSync(",
        "    process.env.MANIFEST_LOG_PATH,",
        "    fs.readFileSync('package.json', 'utf8'),",
        "  );",
        "  state.published = true;",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "  process.exit(0);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'version') {",
        "  if (state.published) {",
        "    process.stdout.write(JSON.stringify('0.57.0'));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'pm' && args[1] === 'view' && args[3] === 'dist-tags') {",
        "  if (state.published) {",
        "    process.stdout.write(JSON.stringify({ latest: '0.57.0' }));",
        "    process.exit(0);",
        "  }",
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
        NPM_FAKE_VERSION: "0.57.0",
        MANIFEST_LOG_PATH: manifestLogPath,
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());

    const manifest = JSON.parse(fs.readFileSync(manifestLogPath, "utf8")) as {
      dependencies: Record<string, string>;
      peerDependencies: Record<string, string>;
    };
    assert.deepEqual(manifest.dependencies, {
      "@vizejs/native": "0.57.0",
      tinyglobby: "0.2.16",
      vize: "0.57.0",
    });
    assert.deepEqual(manifest.peerDependencies, {
      vite: "npm:@voidzero-dev/vite-plus-core@0.1.19",
    });
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("prepare_npm_publish_manifest pins native binary catalog deps to the release version", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-prepare-native-"));
  const repoDir = path.join(tempDir, "repo");
  const nativeDir = path.join(repoDir, "npm", "native");

  try {
    fs.mkdirSync(nativeDir, { recursive: true });
    writeFileSync(
      path.join(repoDir, "pnpm-workspace.yaml"),
      [
        "packages:",
        '  - "npm/*"',
        "",
        "catalogs:",
        "  native-binaries:",
        '    "@vizejs/native-linux-x64-gnu": "0.57.0"',
        '    "@vizejs/native-linux-x64-musl": "0.57.0"',
        "",
      ].join("\n"),
    );
    writeFileSync(
      path.join(nativeDir, "package.json"),
      `${JSON.stringify(
        {
          name: "@vizejs/native",
          version: "0.58.0",
          optionalDependencies: {
            "@vizejs/native-linux-x64-gnu": "catalog:native-binaries",
            "@vizejs/native-linux-x64-musl": "catalog:native-binaries",
          },
        },
        null,
        2,
      )}\n`,
    );

    const result = runReleaseCommand("prepare_npm_publish_manifest", [nativeDir]);
    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());

    const manifest = JSON.parse(fs.readFileSync(path.join(nativeDir, "package.json"), "utf8")) as {
      optionalDependencies: Record<string, string>;
    };
    assert.deepEqual(manifest.optionalDependencies, {
      "@vizejs/native-linux-x64-gnu": "0.58.0",
      "@vizejs/native-linux-x64-musl": "0.58.0",
    });
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_npm_package skips publish when the version is already visible in npm", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-skip-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");

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

    const result = runReleaseCommand("publish_npm_package", [packageDir, "--provenance"], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_ARGS_LOG: argsLogPath,
        NPM_FAKE_ALWAYS_VISIBLE: "1",
        NPM_FAKE_VERSION: "1.2.3-beta.1",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already published/i);
    assert.equal(fs.existsSync(argsLogPath), false);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_npm_package treats a non-zero publish exit as success when npm already has the version", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-recover-"));
  const packageDir = path.join(tempDir, "pkg");
  const binDir = path.join(tempDir, "bin");
  const statePath = path.join(tempDir, "vp-state.json");

  try {
    fs.mkdirSync(packageDir, { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    writeFakeNpmRegistry(binDir);
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

    const result = runReleaseCommand("publish_npm_package", [packageDir], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_STATE_PATH: statePath,
        NPM_FAKE_STATE_PATH: statePath,
        NPM_FAKE_VERSION: "1.2.3",
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
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-dirs-"));
  const baseDir = path.join(tempDir, "packages");
  const binDir = path.join(tempDir, "bin");
  const registryDir = path.join(tempDir, "registry");
  const publishLogPath = path.join(tempDir, "published.jsonl");

  try {
    fs.mkdirSync(path.join(baseDir, "pkg-a"), { recursive: true });
    fs.mkdirSync(path.join(baseDir, "skip-me"), { recursive: true });
    fs.mkdirSync(path.join(baseDir, "pkg-c"), { recursive: true });
    fs.mkdirSync(binDir, { recursive: true });
    fs.mkdirSync(registryDir);
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
        "if (args[0] !== 'pm' || args[1] !== 'publish') process.exit(1);",
        "const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));",
        "const published = path.join(process.env.MOCK_REGISTRY_DIR, encodeURIComponent(pkg.name));",
        "const pending = published + '.' + process.pid;",
        "fs.writeFileSync(pending, JSON.stringify({ name: pkg.name, version: pkg.version }));",
        "fs.renameSync(pending, published);",
        "fs.appendFileSync(process.env.MOCK_PUBLISH_LOG, JSON.stringify(pkg.name) + '\\n');",
      ].join("\n"),
    );
    // Version and tag requests run concurrently. Registry reads must not rewrite
    // the publisher's state or share a mutable JSON counter file.
    writeFakeCommand(
      binDir,
      "curl",
      [
        "const fs = require('node:fs');",
        "const path = require('node:path');",
        "const url = new URL(process.argv.at(-1));",
        "const [, encodedName, requestedVersion] = url.pathname.split('/');",
        "const published = path.join(process.env.MOCK_REGISTRY_DIR, encodeURIComponent(decodeURIComponent(encodedName)));",
        "if (!fs.existsSync(published)) { process.stdout.write('{}VIZE_HTTP_STATUS:404'); process.exit(0); }",
        "const pkg = JSON.parse(fs.readFileSync(published, 'utf8'));",
        "if (requestedVersion && requestedVersion !== pkg.version) { process.stdout.write('{}VIZE_HTTP_STATUS:404'); process.exit(0); }",
        "const body = requestedVersion ? pkg : { name: pkg.name, 'dist-tags': { latest: pkg.version } };",
        "process.stdout.write(JSON.stringify(body) + 'VIZE_HTTP_STATUS:200');",
      ].join("\n"),
    );

    const result = runReleaseCommand("publish_npm_package_dirs", [baseDir], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        MOCK_REGISTRY_DIR: registryDir,
        MOCK_PUBLISH_LOG: publishLogPath,
        PUBLISH_RETRY_LIMIT: "1",
        PUBLISH_RETRY_DELAY: "1",
        PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
        PUBLISH_RESOLUTION_RETRY_DELAY: "1",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    const publishedNames = fs
      .readFileSync(publishLogPath, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line) as string);
    assert.deepEqual(publishedNames, ["@vizejs/pkg-a", "@vizejs/pkg-c"]);
    assert.deepEqual(fs.readdirSync(registryDir).map(decodeURIComponent).sort(), publishedNames);
    for (const name of publishedNames) {
      const published = path.join(registryDir, encodeURIComponent(name));
      assert.deepEqual(JSON.parse(fs.readFileSync(published, "utf8")), { name, version: "1.0.0" });
    }
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_vscode_extension adds pre-release when NPM_TAG is not latest", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-vsix-"));
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const packageJsonPath = path.join(tempDir, "package.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFileSync(
      packageJsonPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "if (args[0] === 'dlx' && args[3] === 'vsce' && args[4] === 'show') { if (fs.existsSync(process.env.VP_ARGS_LOG)) { process.stdout.write(JSON.stringify({ versions: [{ version: '0.57.0' }] })); process.exit(0); } process.exit(1); }",
        "fs.writeFileSync(process.env.VP_ARGS_LOG, args.join('\\n'));",
        "process.exit(0);",
      ].join("\n"),
    );

    const result = runReleaseCommand("publish_vscode_extension", [vsixPath, packageJsonPath], {
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

test("publish_vscode_extension skips publish when the Marketplace already has the version", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-vsix-skip-"));
  const binDir = path.join(tempDir, "bin");
  const argsLogPath = path.join(tempDir, "vp-args.log");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const packageJsonPath = path.join(tempDir, "package.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFileSync(
      packageJsonPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "fs.appendFileSync(process.env.VP_ARGS_LOG, `${args.join(' ')}\\n`);",
        "if (args[0] === 'dlx' && args[3] === 'vsce' && args[4] === 'show') {",
        "  process.stdout.write(JSON.stringify({ versions: [{ version: '0.57.0' }] }));",
        "  process.exit(0);",
        "}",
        "if (args[0] === 'dlx' && args[3] === 'vsce' && args[4] === 'publish') process.exit(99);",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runReleaseCommand("publish_vscode_extension", [vsixPath, packageJsonPath], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_ARGS_LOG: argsLogPath,
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /already published/i);
    const loggedArgs = fs.readFileSync(argsLogPath, "utf8").trim().split("\n");
    assert.equal(
      loggedArgs.some((line) => line.includes("vsce publish")),
      false,
    );
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});

test("publish_vscode_extension waits for delayed Marketplace visibility after publish failure", () => {
  const tempDir = mkdtempSync(path.join(tmpdir(), "release-publish-vsix-delayed-"));
  const binDir = path.join(tempDir, "bin");
  const statePath = path.join(tempDir, "vp-state.json");
  const vsixPath = path.join(tempDir, "vize.vsix");
  const packageJsonPath = path.join(tempDir, "package.json");

  try {
    fs.mkdirSync(binDir, { recursive: true });
    writeFileSync(vsixPath, "placeholder");
    writeFileSync(
      packageJsonPath,
      `${JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }, null, 2)}\n`,
    );
    writeFakeCommand(
      binDir,
      "vp",
      [
        "const fs = require('node:fs');",
        "const args = process.argv.slice(2);",
        "const state = fs.existsSync(process.env.VP_STATE_PATH)",
        "  ? JSON.parse(fs.readFileSync(process.env.VP_STATE_PATH, 'utf8'))",
        "  : { publishCalls: 0, showCalls: 0 };",
        "if (args[0] === 'dlx' && args[3] === 'vsce' && args[4] === 'show') {",
        "  state.showCalls += 1;",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "  if (state.publishCalls > 0 && state.showCalls >= 3) {",
        "    process.stdout.write(JSON.stringify({ versions: [{ version: '0.57.0' }] }));",
        "    process.exit(0);",
        "  }",
        "  process.exit(1);",
        "}",
        "if (args[0] === 'dlx' && args[3] === 'vsce' && args[4] === 'publish') {",
        "  state.publishCalls += 1;",
        "  fs.writeFileSync(process.env.VP_STATE_PATH, JSON.stringify(state));",
        "  process.exit(1);",
        "}",
        "process.exit(1);",
      ].join("\n"),
    );

    const result = runReleaseCommand("publish_vscode_extension", [vsixPath, packageJsonPath], {
      env: {
        PATH: `${binDir}${path.delimiter}${process.env.PATH ?? ""}`,
        VP_STATE_PATH: statePath,
        PUBLISH_RESOLUTION_RETRY_LIMIT: "3",
        PUBLISH_RESOLUTION_RETRY_DELAY: "0",
      },
    });

    assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
    assert.match(result.stdout, /was published despite a non-zero vsce publish exit/i);
    const state = JSON.parse(fs.readFileSync(statePath, "utf8")) as {
      publishCalls: number;
      showCalls: number;
    };
    assert.equal(state.publishCalls, 1);
    assert.equal(state.showCalls, 3);
  } finally {
    rmSync(tempDir, { recursive: true, force: true });
  }
});
