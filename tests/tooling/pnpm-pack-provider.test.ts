import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import {
  packWithPnpm,
  resolvePnpmPackProvider,
} from "../../tools/support/compat/package-managers/pnpm-pack-provider.ts";

function fixture() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-pnpm-provider-")));
  fs.writeFileSync(
    path.join(root, "package.json"),
    JSON.stringify({ packageManager: "pnpm@12.1.0" }),
  );
  const entry = path.join(root, "pnpm-native");
  fs.writeFileSync(entry, Buffer.from("7f454c4602010100", "hex"));
  return { root, entry, cleanup: () => fs.rmSync(root, { recursive: true, force: true }) };
}
const valid = () => ({ status: 0, signal: null, stdout: "12.1.0\n" });
void test("pack wrapper refuses a missing lifecycle owner before launching any child", () => {
  const f = fixture(),
    previous = process.env.npm_execpath;
  try {
    delete process.env.npm_execpath;
    assert.throws(
      () =>
        packWithPnpm(
          f.root,
          pathToFileURL(`${f.root}${path.sep}`),
          path.join(f.root, "pack"),
          120_000,
        ),
      /npm_execpath is missing/,
    );
    assert.deepEqual(fs.readdirSync(f.root).sort(), ["package.json", "pnpm-native"]);
  } finally {
    if (previous === undefined) delete process.env.npm_execpath;
    else process.env.npm_execpath = previous;
    f.cleanup();
  }
});
void test("active physical native PNPM bypasses hostile PATH without executing fixture images", () => {
  const f = fixture();
  const previousPath = process.env.PATH;
  try {
    const hostile = path.join(f.root, "hostile");
    fs.mkdirSync(hostile);
    fs.writeFileSync(path.join(hostile, "pnpm"), "#!/bin/sh\nexit 99\n");
    process.env.PATH = hostile;
    const calls: unknown[] = [];
    const result = resolvePnpmPackProvider(f.root, f.entry, (command, args, cwd) => {
      calls.push({ command, args, cwd });
      return valid();
    });
    assert.deepEqual(calls, [{ command: f.entry, args: ["--version"], cwd: f.root }]);
    assert.equal(result.command, f.entry);
    assert.deepEqual(result.prefix, []);
    assert.equal(result.identity.format, "ELF");
    assert.equal(result.identity.image.bytes, 8);
  } finally {
    process.env.PATH = previousPath;
    f.cleanup();
  }
});
void test("missing, relative, symlink, directory, foreign and unowned script providers fail closed", () => {
  const f = fixture();
  try {
    assert.throws(() => resolvePnpmPackProvider(f.root, undefined, valid), /is missing/);
    assert.throws(() => resolvePnpmPackProvider(f.root, "pnpm", valid), /absolute/);
    for (const entry of [undefined, "pnpm", path.join(f.root, "missing"), f.root])
      assert.throws(() => resolvePnpmPackProvider(f.root, entry, valid));
    const link = path.join(f.root, "link");
    fs.symlinkSync(f.entry, link);
    assert.throws(() => resolvePnpmPackProvider(f.root, link, valid), /regular file/);
    fs.writeFileSync(f.entry, '#!/bin/sh\nexec vp "$@"\n');
    assert.throws(() => resolvePnpmPackProvider(f.root, f.entry, valid), /belong to pnpm/);
  } finally {
    f.cleanup();
  }
});
void test("wrong version, signal, status and provider mutation are refused", () => {
  const f = fixture();
  try {
    for (const result of [
      { status: 0, stdout: "vp v1.4.0" },
      { status: 0, stdout: "12.0.0" },
      { status: 1, stdout: "12.1.0" },
      { status: null, signal: "SIGABRT", stdout: "" },
    ])
      assert.throws(() => resolvePnpmPackProvider(f.root, f.entry, () => result));
    assert.throws(
      () =>
        resolvePnpmPackProvider(f.root, f.entry, () => {
          fs.appendFileSync(f.entry, "changed");
          return valid();
        }),
      /changed during version probe/,
    );
  } finally {
    f.cleanup();
  }
});
void test("selection custody survives a failed version probe and native formats stay explicit", () => {
  const f = fixture();
  try {
    const observations: unknown[] = [];
    assert.throws(
      () =>
        resolvePnpmPackProvider(
          f.root,
          f.entry,
          () => ({ status: null, signal: "SIGABRT", stdout: "", pid: 123 }),
          (value) => observations.push(value),
        ),
      /version probe failed/,
    );
    assert.equal(observations.length, 2);
    assert.equal((observations[0] as { phase: string }).phase, "selected-before-version");
    assert.equal((observations[1] as { pid: number }).pid, 123);
    for (const [magic, format] of [
      ["4d5a0000", "PE"],
      ["cffaedfe", "Mach-O"],
      ["cafebabe", "Mach-O"],
    ]) {
      fs.writeFileSync(f.entry, Buffer.from(magic, "hex"));
      assert.equal(resolvePnpmPackProvider(f.root, f.entry, valid).identity.format, format);
    }
  } finally {
    f.cleanup();
  }
});
void test("JS requires the exact owning PNPM manifest entry and rejects escapes and mismatches", () => {
  const f = fixture();
  try {
    const owner = path.join(f.root, "pnpm"),
      entry = path.join(owner, "bin", "pnpm.cjs");
    fs.mkdirSync(path.dirname(entry), { recursive: true });
    fs.writeFileSync(entry, "#!/usr/bin/env node\n");
    const manifest = (version: string, bin: string) =>
      fs.writeFileSync(
        path.join(owner, "package.json"),
        JSON.stringify({ name: "pnpm", version, bin: { pnpm: bin } }),
      );
    manifest("12.1.0", "bin/pnpm.cjs");
    const selected = resolvePnpmPackProvider(f.root, entry, valid);
    assert.equal(selected.command, process.execPath);
    assert.deepEqual(selected.prefix, [entry]);
    assert.equal(selected.identity.format, "owned JavaScript");
    for (const [version, bin] of [
      ["12.0.0", "bin/pnpm.cjs"],
      ["12.1.0", "../escape.cjs"],
      ["12.1.0", "other.cjs"],
    ]) {
      manifest(version, bin);
      assert.throws(() => resolvePnpmPackProvider(f.root, entry, valid));
    }
  } finally {
    f.cleanup();
  }
});
