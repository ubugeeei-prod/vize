import assert from "node:assert/strict";
import { test } from "node:test";
import {
  assertOutput,
  expectedOutput,
  assertInventory,
  expectedCells,
  assertNativeObservation,
  type NativeObservation,
} from "../../tools/support/compat/content-mapper/direct-child-props-runtime-11809.ts";

void test("numeric prop diagnostic requires every authored diagnostic byte", () => {
  const expected = expectedOutput("wrong");
  assertOutput(2, Buffer.from(expected), Buffer.alloc(0), expected);
  for (const actual of [
    "",
    expected + expected,
    expected.replace("(4,24)", "(4,25)"),
    expected.replace("TS2322", "TS2339"),
  ]) {
    assert.throws(() => assertOutput(2, Buffer.from(actual), Buffer.alloc(0), expected));
  }
  assert.throws(() => assertOutput(0, Buffer.alloc(0), Buffer.alloc(0), expected));
  assert.throws(() => assertOutput(null, Buffer.from(expected), Buffer.alloc(0), expected));
  assert.throws(() =>
    assertOutput(2, Buffer.from(expected), Buffer.from("mapper failed"), expected),
  );
});
void test("valid number and genuine missing-property control have independent contracts", () => {
  assertOutput(0, Buffer.alloc(0), Buffer.alloc(0), expectedOutput("valid"));
  const missing = expectedOutput("missing");
  assertOutput(2, Buffer.from(missing), Buffer.alloc(0), missing);
  assert.throws(() => assertOutput(0, Buffer.from(missing), Buffer.alloc(0), ""));
  assert.throws(() => assertOutput(2, Buffer.alloc(0), Buffer.alloc(0), missing));
});
void test("all 36 cells are mandatory without missing, duplicate or unknown evidence", () => {
  assertInventory(expectedCells);
  for (const rows of [
    [],
    expectedCells.slice(1),
    [...expectedCells, "official-bench-count-false-wrong"],
    [...expectedCells.slice(1), "unknown"],
  ])
    assert.throws(() => assertInventory(rows));
});
void test("actual mapper load cannot borrow producer runtime, image, token or CLI identity", () => {
  const executable = { path: "/node22/bin/node", bytes: 1024, sha256: "a".repeat(64) };
  const sha = "b".repeat(64),
    token = "c".repeat(32),
    cli = "/install/node_modules/vize/bin/vize";
  const record: NativeObservation = {
    runtime: { version: "v22.23.3", platform: "linux", arch: "x64", executable },
    image: { path: "/image.node", bytes: 2048, sha256: sha },
    token,
    argv: [executable.path, cli, "content-mapper"],
    cwd: "/install/node_modules/vize",
    pid: 123,
  };
  assertNativeObservation(record, executable, sha, token, cli);
  for (const changed of [
    { ...record, runtime: { ...record.runtime, version: "v24.15.0" } },
    { ...record, runtime: { ...record.runtime, arch: "arm64" } },
    {
      ...record,
      runtime: { ...record.runtime, executable: { ...executable, sha256: "d".repeat(64) } },
    },
    { ...record, image: { ...record.image, sha256: "d".repeat(64) } },
    { ...record, token: "d".repeat(32) },
    { ...record, argv: [executable.path, cli, "check"] },
    { ...record, argv: [executable.path, "/other/vize", "content-mapper"] },
  ])
    assert.throws(() => assertNativeObservation(changed, executable, sha, token, cli));
});

void test("archive custody rejects changed bytes and package locks require the exact version", async () => {
  const fs = await import("node:fs");
  const { tmpdir } = await import("node:os");
  const path = await import("node:path");
  const { createHash } = await import("node:crypto");
  const { archiveIdentity, lockedPackage } =
    await import("../../tools/support/compat/content-mapper/direct-child-props-runtime-11809.ts");
  const directory = fs.mkdtempSync(path.join(tmpdir(), "child-props-sri-"));
  try {
    const file = path.join(directory, "package.tgz");
    fs.writeFileSync(file, "original archive bytes");
    const integrity = `sha512-${createHash("sha512").update(fs.readFileSync(file)).digest("base64")}`;
    assert.equal(archiveIdentity(file, integrity).integrity, integrity);
    fs.writeFileSync(file, "changed archive bytes");
    assert.throws(() => archiveIdentity(file, integrity));
    const key = "node_modules/node-linux-x64";
    const valid = {
      version: "22.23.3",
      resolved: "https://registry.npmjs.org/node-linux-x64/-/node-linux-x64-22.23.3.tgz",
      integrity,
    };
    assert.equal(lockedPackage({ packages: { [key]: valid } }, key, "22.23.3").version, "22.23.3");
    for (const value of [
      {},
      { version: "24.15.0", integrity },
      { ...valid, integrity: "" },
      { ...valid, resolved: undefined },
    ]) {
      assert.throws(() => lockedPackage({ packages: { [key]: value } }, key, "22.23.3"));
    }
    assert.throws(() => lockedPackage({ packages: {} }, key, "22.23.3"));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

for (const [name, version, executable] of [
  ["@typescript/typescript-linux-x64", "7.1.0-beta", "lib/tsc"],
  ["@tsrs/linux-x64-gnu", "0.21.0-ts7.1.0-beta", "tsrs"],
] as const) {
  void test(`physical CLI is selectable when ${name} exports only its manifest`, async () => {
    const fs = await import("node:fs");
    const path = await import("node:path");
    const { tmpdir } = await import("node:os");
    const { createRequire } = await import("node:module");
    const { physicalHost } =
      await import("../../tools/support/compat/content-mapper/direct-child-props-runtime-11809.ts");
    const project = fs.mkdtempSync(path.join(tmpdir(), "child-props-exports-"));
    try {
      const root = path.join(project, "node_modules", name);
      const binary = path.join(root, executable);
      fs.mkdirSync(path.dirname(binary), { recursive: true });
      fs.writeFileSync(
        path.join(root, "package.json"),
        JSON.stringify({ name, version, exports: { "./package.json": "./package.json" } }),
      );
      fs.writeFileSync(binary, "pinned physical executable bytes");
      const require = createRequire(path.join(project, "package.json"));
      assert.throws(() => require.resolve(`${name}/${executable}`), {
        code: "ERR_PACKAGE_PATH_NOT_EXPORTED",
      });
      const selected = physicalHost(require, name, version, executable);
      assert.equal(selected.executable.path, fs.realpathSync(binary));
      assert.equal(selected.executable.bytes, fs.statSync(binary).size);
      assert.equal(selected.manifest.path, fs.realpathSync(path.join(root, "package.json")));
      assert.throws(() => physicalHost(require, name, "wrong-version", executable));
      fs.rmSync(binary);
      assert.throws(() => physicalHost(require, name, version, executable));
    } finally {
      fs.rmSync(project, { recursive: true, force: true });
    }
  });
}
