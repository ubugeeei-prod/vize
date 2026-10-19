import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  asciiLowercase,
  moonTrim,
  parseMoonInt,
  readDir,
  replaceEvery,
  replaceFirst,
} from "../../tools/support/release/command_runtime.ts";
import { ioErrorMessage, spawnErrorText } from "../../tools/support/release/command_io.ts";
import { bumpVersion } from "../../tools/support/release/release/versions.ts";
import { runReleaseCommand } from "./support/release-command.ts";

test("integers follow the original sign, prefix, underscore and 32-bit grammar", () => {
  const cases: [string, number | null][] = [
    ["5", 5],
    ["+5", 5],
    ["-5", -5],
    ["05", 5],
    ["0x1f", 31],
    ["0X1F", 31],
    ["0o17", 15],
    ["0b101", 5],
    ["0x_1", 1],
    ["1_000", 1000],
    ["2147483647", 2147483647],
    ["-2147483648", -2147483648],
    ["2147483648", null],
    ["-2147483649", null],
    ["99999999999999999999999", null],
    ["", null],
    [" 5", null],
    ["5 ", null],
    ["_1", null],
    ["1_", null],
    ["1__0", null],
    ["0b2", null],
    ["0x", null],
    ["1e3", null],
    ["1.0", null],
    ["١", null],
  ];
  for (const [input, expected] of cases) assert.equal(parseMoonInt(input), expected, input);
});

test("trimming, replacement and lowercasing keep the original narrow semantics", () => {
  assert.equal(moonTrim("\t\r\n value \n"), "value");
  assert.equal(moonTrim("\u000bvalue "), "\u000bvalue ");
  assert.equal(replaceFirst("a$&a", "a", "$&$1"), "$&$1$&a");
  assert.equal(replaceFirst("abc", "", "x"), "xabc");
  assert.equal(replaceEvery("a.a.a", ".", "$'"), "a$'a$'a");
  assert.equal(replaceEvery("a😀", "", "-"), "-a-😀-");
  assert.equal(asciiLowercase("ALREADY EXİSTS"), "already exİsts");
});

test("version bumps keep the original integer parsing and 32-bit wrapping", () => {
  const cases: [string, string, string | null][] = [
    ["1.2.3-alpha", "alpha", "1.2.3-alpha.2"],
    ["1.2.3-alpha.7", "alpha", "1.2.3-alpha.8"],
    ["1.2.3-beta.1", "alpha", "1.2.3-alpha"],
    ["1.2.3-alpha.01", "alpha", "1.2.3-alpha.2"],
    ["01.2.3", "patch", "1.2.4"],
    ["+1.2.3", "minor", "1.3.0"],
    ["0x1.0b1.0o7", "patch", "1.1.8"],
    ["1_0.2.3", "major", "11.0.0"],
    ["1.2.2147483647", "patch", "1.2.-2147483648"],
    ["2147483647.0.0", "major", "-2147483648.0.0"],
    ["1.2.3-alpha.2147483647", "alpha", "1.2.3-alpha.-2147483648"],
    ["2147483648.0.0", "patch", null],
    ["1.2.3-", "patch", null],
    ["1.2.3-a.b.c", "patch", null],
    ["1.2.3-alpha-beta", "patch", null],
    ["1.2.3-alpha.-1", "patch", null],
    [" 1.2.3", "patch", null],
    ["1.2", "patch", null],
    ["1.2.3", "other", null],
  ];
  for (const [version, bump, expected] of cases) {
    assert.equal(bumpVersion(version, bump), expected, `${version} ${bump}`);
  }
});

test("IO and spawn failures carry the original C error texts", () => {
  const failure = (code: string) => Object.assign(new Error(`${code}: x, open 'y'`), { code });
  assert.equal(ioErrorMessage(failure("ENOENT")), "No such file or directory");
  assert.equal(ioErrorMessage(failure("EISDIR")), "Is a directory");
  assert.equal(ioErrorMessage(failure("EWHATEVER")), "X");
  assert.equal(
    spawnErrorText(
      Object.assign(new Error("spawn x ENOENT"), { code: "ENOENT", syscall: "spawn x" }),
    ),
    'OSError("@process.run(): No such file or directory")',
  );
});

test("directory reads keep raw entries and report opendir failures like the original", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-runtime-"));
  try {
    fs.writeFileSync(path.join(root, "b"), "");
    fs.mkdirSync(path.join(root, "a"));
    assert.deepEqual(readDir(root).sort(), ["a", "b"]);
    const runtime = path.join(
      import.meta.dirname,
      "../../tools/support/release/command_runtime.ts",
    );
    const missing = spawnSync(
      process.execPath,
      [
        "--input-type=module",
        "-e",
        `import { readDir } from ${JSON.stringify(runtime)}; console.log(JSON.stringify(readDir("missing")));`,
      ],
      { cwd: root, encoding: "utf8" },
    );
    assert.equal(missing.status, 0);
    assert.equal(missing.stderr, "opendir: No such file or directory\n");
    assert.equal(missing.stdout, "[]\n");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("an uncaught failed child start reports the original runtime text and status", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "release-runtime-spawn-"));
  try {
    fs.writeFileSync(path.join(root, "Cargo.toml"), '[workspace.package]\nversion = "1.2.3"\n');
    const result = runReleaseCommand("publish_crates", ["--dry-run"], {
      cwd: root,
      env: { PATH: path.join(root, "missing-bin") },
    });
    assert.equal(result.status, 1);
    assert.equal(
      result.stdout,
      'Expected version: 1.2.3\nOSError("@process.run(): No such file or directory")\n',
    );
    assert.equal(result.stderr, "");
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
