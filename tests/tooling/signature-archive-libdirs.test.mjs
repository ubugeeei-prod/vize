import assert from "node:assert/strict";
import { test } from "node:test";
import {
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  archivedLibdirs,
  libraryPaths,
  loaderDependencyPaths,
  requireDependencyCustody,
} from "../../tools/support/compat/github/signature-archive-libdirs.mjs";
import { sha256 } from "../../tools/support/compat/github/signature-archive-capture.mjs";

const workspace = "/home/runner/_work/vize/vize";
const fixture = (targets = 1) => {
  const original = {
    frozen: { complete: ["all", "other", "metadata"] },
    "rust-build-meta": {
      "target-directory": `${workspace}/target`,
      platforms: {
        host: {
          platform: "x86_64-unknown-linux-gnu",
          libdir: {
            status: "available",
            path: "/original/toolchain/libdir",
          },
        },
        targets: Array.from({ length: targets }, () => ({
          platform: "x86_64-unknown-linux-gnu",
          libdir: { status: "available", path: "/original/target/libdir" },
        })),
      },
    },
  };
  const archiveList = structuredClone(original);
  archiveList["rust-build-meta"].platforms.host.libdir.path =
    `${workspace}/target/nextest/libdirs/host`;
  for (const target of archiveList["rust-build-meta"].platforms.targets) {
    target.libdir.path = `${workspace}/target/nextest/libdirs/target/0`;
  }
  return { original, archiveList };
};

void test("archive mapping changes only exact host/target libdirs with full inverse", () => {
  const { original, archiveList } = fixture();
  const frozen = structuredClone(original);
  const { derived, edits } = archivedLibdirs(original, archiveList, workspace);
  assert.deepEqual(derived, archiveList);
  assert.deepEqual(original, frozen);
  assert.deepEqual(
    edits.map((edit) => edit.selector),
    ["host", "target"],
  );
  assert.deepEqual(derived.frozen, frozen.frozen);
});

void test("zero targets remains zero and actual missing archive libdir remains unavailable", () => {
  const { original, archiveList } = fixture(0);
  archiveList["rust-build-meta"].platforms.host.libdir = {
    status: "unavailable",
    reason: "not-in-archive",
  };
  const { derived, edits } = archivedLibdirs(original, archiveList, workspace);
  assert.equal(derived["rust-build-meta"].platforms.targets.length, 0);
  assert.equal(edits.length, 1);
  assert.deepEqual(derived["rust-build-meta"].platforms.host.libdir, {
    status: "unavailable",
    reason: "not-in-archive",
  });
});

void test("foreign libdir, changed platform and unsupported old/multiple-target schemas refuse", () => {
  for (const mutate of [
    (value) => {
      value.archiveList["rust-build-meta"].platforms.host.libdir.path = "/foreign/libdir";
    },
    (value) => {
      value.archiveList["rust-build-meta"].platforms.host.platform = "foreign";
    },
    (value) => {
      value.archiveList["rust-build-meta"].platforms.targets = [];
    },
    (value) => {
      delete value.original["rust-build-meta"].platforms;
    },
    (value) => {
      value.archiveList["rust-build-meta"].platforms.host.libdir = {
        status: "unavailable",
        reason: "unknown",
      };
    },
  ]) {
    const value = fixture();
    mutate(value);
    assert.throws(() => archivedLibdirs(value.original, value.archiveList, workspace));
  }
  const value = fixture(2);
  assert.throws(() => archivedLibdirs(value.original, value.archiveList, workspace));
});

void test("library search retains exact sorted existence, artifact/deps and host/target order", () => {
  const meta = {
    "target-directory": "/target",
    "build-directory": "/build",
    "linked-paths": ["z", "a", "absent"],
    "base-output-directories": ["release", "ci"],
    platforms: {
      host: { libdir: { status: "available", path: "/host/libdir" } },
      targets: [{ libdir: { status: "available", path: "/target/libdir" } }],
    },
  };
  const value = libraryPaths(meta, (path) => path !== "/build/absent");
  assert.deepEqual(value.ordered, [
    "/build/a",
    "/build/z",
    "/target/ci",
    "/build/ci/deps",
    "/target/release",
    "/build/release/deps",
    "/host/libdir",
    "/target/libdir",
  ]);
  assert.deepEqual(value.linked, [
    { path: "/build/a", exists: true },
    { path: "/build/absent", exists: false },
    { path: "/build/z", exists: true },
  ]);
  assert.notDeepEqual(
    libraryPaths(meta, () => true),
    value,
  );
  assert.deepEqual(
    libraryPaths(meta, (path) => path !== "/build/absent", ["/toolchain/lib", "/host/libdir"])
      .ordered,
    [...value.ordered, "/toolchain/lib", "/host/libdir"],
  );
});

void test("complete GNU loader report preserves aliases and refuses any unsupported line", () => {
  const output =
    "linux-vdso.so.1 (0x0001)\nlibc.so.6 => /lib/libc.so.6 (0x0002)\n/lib/ld-linux-x86-64.so.2 (0x0003)\n";
  assert.deepEqual(loaderDependencyPaths(output, "/lib/ld-linux-x86-64.so.2"), [
    "/lib/ld-linux-x86-64.so.2",
    "/lib/libc.so.6",
  ]);
  assert.throws(() => loaderDependencyPaths("", "/lib/loader"), /unresolved\/unsupported/);
  for (const line of ["libstd.so => not found", "linux-vdso.so.2 (0x0001)", "unsupported line"])
    assert.throws(
      () => loaderDependencyPaths(output + `${line}\n`, "/lib/loader"),
      /unresolved\/unsupported/,
    );
});

void test("dependency custody refuses alias retarget while original complete bytes stay unchanged", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-loader-alias-law-"));
  try {
    const original = join(directory, "original.so");
    const changed = join(directory, "changed.so");
    const alias = join(directory, "selected.so");
    writeFileSync(original, "whole original dependency");
    writeFileSync(changed, "whole different dependency");
    symlinkSync(original, alias, "file");
    const custody = [
      { loaderReportedPath: alias, path: realpathSync(alias), sha256: sha256(readFileSync(alias)) },
    ];
    requireDependencyCustody(custody);
    unlinkSync(alias);
    symlinkSync(changed, alias, "file");
    assert.equal(sha256(readFileSync(original)), custody[0].sha256);
    assert.throws(() => requireDependencyCustody(custody), /alias retargeted/);
    unlinkSync(alias);
    symlinkSync(original, alias, "file");
    requireDependencyCustody(custody);
    writeFileSync(original, "changed complete original bytes");
    assert.throws(() => requireDependencyCustody(custody));
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
