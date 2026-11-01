import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { writeTarGz } from "../../../tools/support/archive/tar.ts";
import { archiveComparison } from "../../../tools/support/release/public_install/archive.ts";
import { observedArchiveComparison } from "../../../tools/support/release/human_reports_11252_11253_public/package-inventory.ts";
import {
  object,
  sha256,
} from "../../../tools/support/release/human_reports_11252_11253_public/value.ts";

function fixture() {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "human-payload-inventory-")));
  const directory = join(root, "package");
  mkdirSync(join(directory, "dist"), { recursive: true });
  const manifest = Buffer.from('{"name":"synthetic-payload","version":"1.0.0"}\n');
  const binary = Buffer.from([0, 0xff, 0x80, 17, 19]);
  writeFileSync(join(directory, "package.json"), manifest);
  writeFileSync(join(directory, "dist/opaque.node"), binary);
  const payload = writeTarGz([
    { name: "package/package.json", data: manifest },
    { name: "package/dist", typeflag: "5" },
    { name: "package/dist/opaque.node", data: binary },
  ]);
  const integrity = "sha512-" + createHash("sha512").update(payload).digest("base64");
  const events: Array<{ kind: string; value: unknown }> = [];
  const observe = () =>
    observedArchiveComparison(directory, payload, integrity, (kind, value) =>
      events.push({ kind, value }),
    );
  const inventory = () => {
    assert.deepEqual(
      events.map((event) => event.kind),
      ["installed-package-file-inventory", "public-package-file-inventory"],
    );
    return { installed: object(events[0]!.value), archive: object(events[1]!.value) };
  };
  return { root, directory, manifest, binary, payload, integrity, events, observe, inventory };
}

test("inventory records all payload paths/kinds/sizes/hashes and returns the unchanged strict comparison", () => {
  const f = fixture();
  try {
    const expected = archiveComparison(f.directory, f.payload, f.integrity);
    assert.deepEqual(f.observe(), expected);
    const { installed, archive } = f.inventory();
    assert.equal(installed.complete, true);
    assert.deepEqual(object(archive.difference).extraInstalledRegularFiles, []);
    assert.deepEqual(object(archive.difference).missingInstalledRegularFiles, []);
    const physical = installed.physical as Array<Record<string, unknown>>;
    assert.deepEqual(
      physical.map((entry) => [entry.path, entry.kind]),
      [
        ["dist", "directory"],
        ["dist/opaque.node", "file"],
        ["package.json", "file"],
      ],
    );
    assert.equal(physical[1]!.bytes, f.binary.length);
    assert.equal(physical[1]!.sha256, sha256(f.binary));
    assert.equal(archive.archiveSha256, sha256(f.payload));
    assert.equal(archive.payloadOrExtraFilesWaived, false);
    assert(readFileSync(join(f.directory, "dist/opaque.node")).equals(f.binary));
    const inspect = (value: unknown): void => {
      assert(!Buffer.isBuffer(value));
      if (Array.isArray(value)) value.forEach(inspect);
      else if (value && typeof value === "object") Object.values(value).forEach(inspect);
    };
    inspect(f.events);
    assert(!JSON.stringify(f.events).includes(f.binary.toString("base64")));
  } finally {
    rmSync(f.root, { recursive: true, force: true });
  }
});

test("an unknown installed extra is retained before the original file-set refusal, never waived", () => {
  const f = fixture();
  try {
    const extra = Buffer.from("unclassified generated-or-foreign content; origin UNKNOWN");
    writeFileSync(join(f.directory, "unknown.txt"), extra);
    assert.throws(f.observe, /public archive and installed package file sets differ/u);
    const { installed, archive } = f.inventory();
    assert.deepEqual(object(archive.difference).extraInstalledRegularFiles, ["unknown.txt"]);
    assert.deepEqual(object(archive.difference).missingInstalledRegularFiles, []);
    const row = (installed.physical as Array<Record<string, unknown>>).find(
      (entry) => entry.path === "unknown.txt",
    );
    assert.deepEqual(row, {
      path: "unknown.txt",
      kind: "file",
      bytes: extra.length,
      sha256: sha256(extra),
      linkTarget: null,
    });
    assert.equal(archive.payloadOrExtraFilesWaived, false);
    assert(!JSON.stringify(f.events).includes(extra.toString()));
  } finally {
    rmSync(f.root, { recursive: true, force: true });
  }
});

test("missing or modified payload bytes retain diagnostics and keep their existing refusal", () => {
  for (const missing of [false, true]) {
    const f = fixture();
    try {
      const path = join(f.directory, "dist/opaque.node");
      if (missing) rmSync(path);
      else writeFileSync(path, "changed bytes");
      assert.throws(
        f.observe,
        missing ? /ENOENT/u : /installed bytes differ from actual public archive/u,
      );
      const { archive } = f.inventory();
      assert.deepEqual(
        object(archive.difference).missingInstalledRegularFiles,
        missing ? ["dist/opaque.node"] : [],
      );
      assert.equal(archive.payloadOrExtraFilesWaived, false);
    } finally {
      rmSync(f.root, { recursive: true, force: true });
    }
  }
});

test("a symlink is recorded without following its target and the unchanged shared guard refuses it", () => {
  const f = fixture();
  try {
    const outside = join(f.root, "outside.txt");
    writeFileSync(outside, "external body must not enter proof");
    symlinkSync(outside, join(f.directory, "redirect"));
    assert.throws(f.observe, /installed package contains a redirect\/special file/u);
    const { installed, archive } = f.inventory();
    const row = (installed.physical as Array<Record<string, unknown>>).find(
      (entry) => entry.path === "redirect",
    );
    assert.equal(row?.kind, "symlink");
    assert.equal(row.sha256, null);
    assert.equal(row.linkTarget, outside);
    assert.equal((archive.installedRegularFiles as string[]).includes("redirect"), false);
    assert(!JSON.stringify(f.events).includes("external body must not enter proof"));
  } finally {
    rmSync(f.root, { recursive: true, force: true });
  }
});
