import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const command = fileURLToPath(
  new URL("../../../tools/support/release/release/main.ts", import.meta.url),
);
const stamp = new URL("../../../tools/support/release/release/stamp.ts", import.meta.url);
const invalid = [
  "",
  " ",
  "\n",
  "0.2",
  "0.2.0-",
  "0.2.0-alpha.",
  "0.2.0-alpha..1",
  "0.2.0-alpha.01",
  "0.2.0-01",
  "01.2.0",
  "+1.2.0",
  "0x1.2.0",
  "v0.2.0",
  "0.2.0\n",
  "0.2.0\r\n",
  '0.2.0"\nversion = "',
  "0.2.0+",
  "0.2.0+build..7",
  "0.2.0-pre_release",
];

function fixture(): string {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-stamp-guard-"));
  const write = (file: string, content: string) => {
    fs.mkdirSync(path.dirname(path.join(directory, file)), { recursive: true });
    fs.writeFileSync(path.join(directory, file), content);
  };
  write(
    "Cargo.toml",
    '[workspace.package]\nversion = "0.1.0"\n\n[workspace.dependencies]\nvize_carton = { path = "c", version = "=0.1.0" }\n',
  );
  write(
    "Cargo.lock",
    '[[package]]\nname = "vize"\nversion = "0.1.0"\n\n[[package]]\nname = "serde"\nversion = "0.1.0"\n',
  );
  for (const file of [
    "npm/cli/package.json",
    "npm/builder/vite/package.json",
    "editors/vscode/package.json",
  ])
    write(file, '{\n  "name": "vize-fixture",\n  "version": "0.1.0"\n}\n');
  write(
    "README.md",
    "npm i vize@0.1.0\n<!-- benchmark:readme:start -->\nmeasured 0.1.0\n<!-- benchmark:readme:end -->\n",
  );
  write(
    "pnpm-workspace.yaml",
    'catalogs:\n  native-binaries:\n    "@vizejs/native-darwin-arm64": "0.1.0"\n',
  );
  write("unrelated/retained.txt", "untouched\n");
  execFileSync("git", ["init", "--quiet"], { cwd: directory });
  return directory;
}

function snapshot(directory: string): unknown[] {
  const result: unknown[] = [];
  const visit = (relative: string) => {
    const absolute = path.join(directory, relative),
      info = fs.lstatSync(absolute);
    result.push([
      relative,
      info.mode,
      info.isFile() ? fs.readFileSync(absolute).toString("base64") : null,
    ]);
    if (info.isDirectory())
      for (const entry of fs.readdirSync(absolute).sort()) visit(path.join(relative, entry));
  };
  visit("");
  return result;
}

test("invalid stamp targets leave every checkout file, directory and mode unchanged", () => {
  const directory = fixture();
  try {
    const original = snapshot(directory);
    for (const version of invalid) {
      const result = spawnSync(process.execPath, [command, "stamp", version], {
        cwd: directory,
        encoding: "utf8",
      });
      assert.equal(result.status, 1, JSON.stringify(version));
      assert.match(result.stderr, /Invalid release target version:/);
      assert.doesNotMatch(result.stdout, /stamped/);
      assert.deepEqual(
        snapshot(directory),
        original,
        `whole checkout unchanged: ${JSON.stringify(version)}`,
      );
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("invalid targets refuse before any stamp target filesystem access", () => {
  const result = spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `
    import assert from "node:assert/strict";
    import fs from "node:fs";
    import {stampVersion} from ${JSON.stringify(stamp.href)};
    let calls = 0;
    for (const name of ["opendirSync", "statSync", "lstatSync", "existsSync", "accessSync", "openSync", "readFileSync", "writeFileSync", "appendFileSync", "mkdirSync"])
      fs[name] = () => {calls++; throw new Error("stamp target IO forbidden");};
    for (const version of ${JSON.stringify(invalid)}) assert.equal(await stampVersion("0.1.0", version), false);
    assert.equal(calls, 0);
    process.stdout.write(JSON.stringify({cases: ${invalid.length}, targetIoCalls: calls}));
  `,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), { cases: invalid.length, targetIoCalls: 0 });
});

test("valid stable and prerelease targets still stamp manifests while retaining unrelated pins and measurements", () => {
  for (const version of [
    "0.2.0",
    "0.2.0-alpha",
    "0.2.0-beta.1",
    "0.2.0-rc.0",
    "0.2.0-preview-2026.1+build.7",
  ]) {
    const directory = fixture();
    try {
      const result = spawnSync(process.execPath, [command, "stamp", version], {
        cwd: directory,
        encoding: "utf8",
      });
      assert.equal(result.status, 0, result.stderr);
      for (const file of [
        "Cargo.toml",
        "npm/cli/package.json",
        "npm/builder/vite/package.json",
        "editors/vscode/package.json",
      ])
        assert.ok(
          fs.readFileSync(path.join(directory, file), "utf8").includes(version),
          `${version}: ${file}`,
        );
      const lock = fs.readFileSync(path.join(directory, "Cargo.lock"), "utf8");
      assert.ok(lock.includes(`name = "vize"\nversion = "${version}"`));
      assert.ok(lock.includes('name = "serde"\nversion = "0.1.0"'));
      assert.match(
        fs.readFileSync(path.join(directory, "pnpm-workspace.yaml"), "utf8"),
        /"0\.1\.0"/,
      );
      assert.equal(
        fs.readFileSync(path.join(directory, "unrelated/retained.txt"), "utf8"),
        "untouched\n",
      );
      assert.equal(
        fs.readFileSync(path.join(directory, "README.md"), "utf8"),
        `npm i vize@${version}\n<!-- benchmark:readme:start -->\nmeasured 0.1.0\n<!-- benchmark:readme:end -->\n`,
      );
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
});
