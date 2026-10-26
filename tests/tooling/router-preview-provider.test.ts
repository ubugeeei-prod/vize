import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { type TarEntry, writeTarGz } from "../../tools/support/archive/tar.ts";
import { packageInventory } from "./support/router-preview/archive-inventory.ts";
import { buildProvider } from "./support/router-preview/build-provider.ts";
import { fixtureDirectory, readAuthority } from "./support/router-preview/authority.ts";
import { readProjectFixture } from "./support/router-preview/project.ts";
import { nativeOracle, readOracles, verifyControl } from "./support/router-preview/oracles.ts";

const digest = (value: string) => createHash("sha256").update(value).digest("hex");

test("preview identity pins real source rather than reusing the published version number", () => {
  const { authority } = readAuthority();
  assert.equal(authority.sourceInputs.length, 9);
  assert.deepEqual(authority.qualification, {
    state: "frozen",
    sourceBuiltPackageIsNotPublishedNpmArchive: true,
    automaticProjection: "not-qualified",
  });
  const project = readProjectFixture();
  assert.deepEqual(
    [...project.files.keys()],
    [
      "generate.mjs",
      "package-lock.json",
      "package.json",
      "src/pages/index.vue",
      "src/pages/products/[id=int].vue",
      "src/params/date.ts",
      "src/router/index.ts",
      "tsconfig.json",
    ],
  );
  assert.equal(project.authority.strictOracles, "frozen-independent-controls");
  const oracles = readOracles();
  assert.equal(oracles.controls.length, 20);
  assert.equal(oracles.independent.size, 40);
});

test("frozen independent output drift is refused before any provider installation", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preview-oracle-control-"));
  try {
    fs.cpSync(fixtureDirectory, root, { recursive: true });
    fs.appendFileSync(path.join(root, "frozen/independent-processes.json"), "\n");
    assert.throws(() => readOracles(root), /frozen\/independent-processes\.json/u);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("whole packet qualification changes only the proven rootDir protocol field", () => {
  const oracles = readOracles();
  const id = "lf/enabled/wrong-id";
  const control = oracles.controls.find((entry) => entry.id === id)!;
  const frozen = oracles.independent.get(`${id}/public-control`)!;
  const root = `/tmp/longer-日本語-🍣-'quoted'-preview-root`;
  const native = { ...frozen, ...nativeOracle(frozen, root), cwd: root };
  const actual = {
    reference: structuredClone(oracles.independent.get(`${id}/reference`)!),
    publicControl: structuredClone(native),
    native: { ...structuredClone(native), label: `${id}/source` },
  };
  verifyControl(oracles, id, control.source, actual, root);
  for (const kind of [
    "position-before",
    "position-after",
    "membership",
    "foreign-root",
    "code",
    "stderr",
    "status",
    "bytes-only",
  ]) {
    const mutated = structuredClone(actual);
    if (kind === "status") mutated.native.status = 0;
    else if (kind === "stderr") {
      mutated.native.stderr = "unexpected warning\n";
      mutated.native.stderrBase64 = Buffer.from(mutated.native.stderr).toString("base64");
    } else if (kind === "bytes-only") mutated.native.stdoutBase64 += "eA==";
    else {
      const report = JSON.parse(mutated.native.stdout);
      if (kind === "position-before" || kind === "position-after") {
        report.files[1].diagnostics[0] = report.files[1].diagnostics[0].replace(
          "error:14:7",
          kind === "position-before" ? "error:14:6" : "error:14:8",
        );
      } else if (kind === "membership") report.programs[0].files.pop();
      else if (kind === "foreign-root") report.programs[0].compilerOptions.rootDir = "/foreign";
      else if (kind === "code")
        report.files[1].diagnostics[0] = report.files[1].diagnostics[0].replace("TS2322", "TS9999");
      mutated.native.stdout = `${JSON.stringify(report, null, 2)}\n`;
      mutated.native.stdoutBase64 = Buffer.from(mutated.native.stdout).toString("base64");
    }
    assert.throws(
      () => verifyControl(oracles, id, control.source, mutated, root),
      { code: "ERR_ASSERTION" },
      kind,
    );
  }
  assert.throws(
    () => verifyControl(oracles, id, `${control.source}\n`, actual, root),
    /authored input/u,
  );
  const withAuthoredRoot = structuredClone(frozen);
  const report = JSON.parse(withAuthoredRoot.stdout);
  const authored = `authored literal '${frozen.cwd}' must remain exact`;
  report.files[1].diagnostics.push(authored);
  withAuthoredRoot.stdout = `${JSON.stringify(report, null, 2)}\n`;
  withAuthoredRoot.stdoutBase64 = Buffer.from(withAuthoredRoot.stdout).toString("base64");
  const transported = JSON.parse(nativeOracle(withAuthoredRoot, root).stdout);
  assert.equal(transported.programs[0].compilerOptions.rootDir, root);
  assert.equal(transported.files[1].diagnostics.at(-1), authored);
});

test("preview project input drift is refused before installation", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-preview-input-control-"));
  try {
    fs.cpSync(fixtureDirectory, root, { recursive: true });
    fs.appendFileSync(path.join(root, "project/src/pages/products/[id=int].vue.txt"), "\n");
    const expected = readProjectFixture().authority.files.find(
      (entry) => entry.path === "src/pages/products/[id=int].vue",
    )!;
    assert.throws(() => readProjectFixture(root), {
      code: "ERR_ASSERTION",
      actual: expected.bytes + 1,
      expected: expected.bytes,
      operator: "strictEqual",
    });
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

test("the real provider build cannot be started by a local helper invocation", () => {
  const previous = process.env.GITHUB_ACTIONS;
  delete process.env.GITHUB_ACTIONS;
  try {
    assert.throws(() => buildProvider("/unused-source", "/unused-output"), {
      code: "ERR_ASSERTION",
      actual: undefined,
      expected: "true",
      operator: "strictEqual",
    });
  } finally {
    if (previous === undefined) delete process.env.GITHUB_ACTIONS;
    else process.env.GITHUB_ACTIONS = previous;
  }
});

test("archive custody keeps complete identities and rejects traversal, links, and duplicates", () => {
  const text = (value: string) => new TextEncoder().encode(value);
  const archive = (kind: string) => {
    const entries: TarEntry[] = [
      { name: "package/", typeflag: "5" },
      { name: "package/package.json", data: text('{"name":"owned-control"}\n') },
      { name: "package/dist/index.d.ts", data: text("export type Value = number\n") },
    ];
    if (kind === "traversal") entries[1] = { name: "package/../outside.txt", data: text("x") };
    if (kind === "absolute") entries[1] = { name: "/package/outside.txt", data: text("x") };
    if (kind === "foreign") entries[1] = { name: "foreign/outside.txt", data: text("x") };
    if (kind === "duplicate") entries.push(entries[1]!);
    if (kind === "symlink" || kind === "hardlink")
      entries.push({
        name: "package/linked",
        typeflag: kind === "symlink" ? "2" : "1",
        linkname: "../../outside",
      });
    return writeTarGz(entries);
  };
  assert.deepEqual(packageInventory(archive("regular")), [
    { path: "dist/index.d.ts", bytes: 27, sha256: digest("export type Value = number\n") },
    { path: "package.json", bytes: 25, sha256: digest('{"name":"owned-control"}\n') },
  ]);
  for (const [kind, message] of [
    ["traversal", "unsafe package member: 'package/../outside.txt'"],
    ["absolute", "unsafe package member: '/package/outside.txt'"],
    ["foreign", "unsafe package member: 'foreign/outside.txt'"],
    ["duplicate", "duplicate package member: 'package.json'"],
    ["symlink", "non-regular package member: 'package/linked'"],
    ["hardlink", "non-regular package member: 'package/linked'"],
  ] as const) {
    assert.throws(() => packageInventory(archive(kind)), { message }, kind);
  }
  assert.throws(() => packageInventory(writeTarGz([{ name: "package/", typeflag: "5" }])), {
    message: "empty package archive",
  });
});
