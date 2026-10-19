import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { writeFakeCommand } from "./support/fake-command.ts";
import { runReleaseCommand } from "./support/release-command.ts";

function temporary(t: { after: (callback: () => void) => void }) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-diagnostics-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}

const usage = (name: string, rest: string) =>
  `Usage: node tools/support/release/${name}/main.ts ${rest}\n`;

test("argument errors keep their exact diagnostics and statuses", (t) => {
  const root = temporary(t);
  const cases: [string, string[], number, string, string][] = [
    [
      "prepare_npm_publish_manifest",
      [],
      1,
      "",
      usage("prepare_npm_publish_manifest", "<dir-or-package-json> [--print]"),
    ],
    [
      "prepare_npm_publish_manifest",
      ["--print"],
      1,
      "",
      "Missing package directory or package.json path\n",
    ],
    [
      "publish_npm_package",
      [],
      1,
      "",
      usage(
        "publish_npm_package",
        "<dir> [--provenance] [--no-wait] [--prepared] [--ignore-failure] [--retries N] [--delay N]",
      ),
    ],
    ["publish_npm_package", [root, "--retries"], 1, "", "Unknown argument: --retries\n"],
    [
      "publish_npm_package_dirs",
      [],
      1,
      "",
      "Usage: node tools/support/release/publish_npm_package_dirs/main.ts <dir>... [--concurrency 1..16] [publish_npm_package flags]\n       node tools/support/release/publish_npm_package_dirs/main.ts --names <dir>...\n",
    ],
    [
      "publish_npm_package_dirs",
      [root, "--concurrency", "17"],
      1,
      "",
      "Usage: node tools/support/release/publish_npm_package_dirs/main.ts <dir>... [--concurrency 1..16] [publish_npm_package flags]\n       node tools/support/release/publish_npm_package_dirs/main.ts --names <dir>...\n",
    ],
    ["publish_npm_package_dirs", [root], 1, "", `No package directories in: ${root}\n`],
    [
      "publish_vscode_extension",
      ["a", "b", "c"],
      1,
      usage("publish_vscode_extension", "<vsix-path> [package-json-path]"),
      "",
    ],
    [
      "publish_open_vsx_extension",
      [],
      1,
      usage("publish_open_vsx_extension", "<vsix-path> [package-json-path]"),
      "",
    ],
  ];
  for (const [name, args, status, stdout, stderr] of cases) {
    const result = runReleaseCommand(name, args, { cwd: root });
    assert.deepEqual(
      [result.status, result.stdout, result.stderr],
      [status, stdout, stderr],
      `${name} ${args.join(" ")}`,
    );
  }
});

test("package metadata failures stop before any registry or publish command", (t) => {
  const root = temporary(t);
  const bin = path.join(root, "bin");
  const log = path.join(root, "calls.log");
  fs.mkdirSync(bin);
  for (const name of ["curl", "vp"]) {
    writeFakeCommand(
      bin,
      name,
      `require('node:fs').appendFileSync(${JSON.stringify(log)}, '${name}\\n');`,
    );
  }
  const env = { PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`, VIZE_REPO_ROOT: root };
  const write = (manifest: string) => fs.writeFileSync(path.join(root, "package.json"), manifest);
  const json = path.join(root, "package.json");
  for (const [manifest, message] of [
    ["{", `Failed to parse ${json}\n`],
    ['{"name":"x"}', `Failed to read version from ${json}\n`],
    ['{"name":"","version":"1.0.0"}', `Failed to read name from ${json}\n`],
    [
      JSON.stringify({
        name: "x",
        version: "1.0.0",
        main: " ./dist/index.js ",
        types: ["./dist/index.d.ts", "https://example.com/a", "#internal"],
        bin: { a: "./bin/a.js", b: "./bin/a.js", c: "" },
        exports: { ".": { types: "./dist/index.d.ts" }, "./present": "./package.json" },
      }),
      `Refusing to publish x@1.0.0 because package.json points at files that are missing from ${root}:\n  - ./dist/index.js\n  - ./dist/index.d.ts\n  - ./bin/a.js\n`,
    ],
  ]) {
    write(manifest);
    const result = runReleaseCommand("publish_npm_package", [root], { env });
    assert.equal(result.status, 1);
    assert.equal(result.stderr, message);
  }
  for (const [name, manifest, message] of [
    ["publish_vscode_extension", "{", `Failed to read ${json}\n`],
    [
      "publish_open_vsx_extension",
      '{"publisher":"p","name":1,"version":"1"}',
      `Failed to read extension metadata from ${json}\n`,
    ],
  ]) {
    write(manifest);
    const result = runReleaseCommand(name, ["x.vsix", json], { env });
    assert.equal(result.status, 1);
    assert.equal(result.stderr, message);
  }
  assert.equal(fs.existsSync(log), false);
});

test("Open VSX namespace failures report trimmed diagnostics and stop before publish", (t) => {
  const root = temporary(t);
  const bin = path.join(root, "bin");
  const log = path.join(root, "vp.log");
  fs.mkdirSync(bin);
  const manifest = path.join(root, "package.json");
  fs.writeFileSync(
    manifest,
    JSON.stringify({ publisher: "ubugeeei", name: "vize", version: "0.57.0" }),
  );
  writeFakeCommand(bin, "curl", "process.exit(22);");
  writeFakeCommand(
    bin,
    "vp",
    `
    const fs = require('node:fs');
    const args = process.argv.slice(2);
    fs.appendFileSync(${JSON.stringify(log)}, args.join(' ') + '\\n');
    if (args.includes('create-namespace')) {
      if (process.env.NAMESPACE === 'exists') { console.error('  Namespace ubugeeei ALREADY Exists  '); process.exit(1); }
      console.log(' out '); console.error('denied\\n'); process.exit(3);
    }
    process.exit(4);
  `,
  );
  const env = {
    PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
    PUBLISH_RESOLUTION_RETRY_LIMIT: "1",
    PUBLISH_RESOLUTION_RETRY_DELAY: "0",
  };
  const denied = runReleaseCommand("publish_open_vsx_extension", ["x.vsix", manifest], {
    cwd: root,
    env,
  });
  assert.equal(denied.status, 1);
  assert.equal(
    denied.stderr,
    "Failed to ensure Open VSX namespace ubugeeei (exit 3).\nout\ndenied\n",
  );
  assert.doesNotMatch(fs.readFileSync(log, "utf8"), / publish /);
  const exists = runReleaseCommand("publish_open_vsx_extension", ["x.vsix", manifest], {
    cwd: root,
    env: { ...env, NAMESPACE: "exists" },
  });
  assert.equal(exists.status, 4, exists.stderr);
  assert.match(
    exists.stdout,
    /^Open VSX namespace ubugeeei already exists; continuing to publish\.\n/,
  );
  assert.match(fs.readFileSync(log, "utf8"), /dlx -p ovsx@\^1\.0\.0 ovsx publish /);
});
