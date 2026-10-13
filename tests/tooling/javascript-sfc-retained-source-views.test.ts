import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateJavaScriptSfcSource,
  decodeJavaScriptSfcRawBatch,
  javascriptSfcSourceReader,
  javascriptSfcSourcePaths,
} from "../../tools/support/compat/github/javascript-sfc-source-context.mjs";
import { retainedPhysicalHostBeforeComponentValues } from "./support/component-value-host-custody.ts";
import {
  authenticateNoMutatingComponentJoinPhysical,
  authenticateNoMutatingComponentJoinCommitted,
} from "./support/no-mutating-component-join.mjs";
import { authenticateJoinLiteralFrame } from "./support/no-mutating-component-join-frame.mjs";
import {
  qualifyAsciiOriginalCallers,
  retainedAsciiCallerFrame,
  originalAsciiCaller,
} from "./support/ascii-symbol-original-callers.ts";
const root = path.resolve(import.meta.dirname, "../..");
const sha = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const blob = (body: Buffer) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const git = (args: string[], input?: Buffer | string, cwd = root) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd,
    input,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
    maxBuffer: 16 * 1024 * 1024,
  });
const actual = javascriptSfcSourceReader(root, true);
const bodies = new Map(javascriptSfcSourcePaths.map((file: string) => [file, actual(file)]));
const read = (file: string) => bodies.get(file);
const hook = "tests/performance/support/warm-type-backed-host-qualifications.ts";
const capsule = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/linter/component-import-value-host-before/host-composition.json",
    ),
    "utf8",
  ),
);
void test("all93 JS and all323 current NoMutating sources precede the genuine historical NoUnused source check", () => {
  const view = authenticateJavaScriptSfcSource(read);
  assert.equal(view.receipt.bodyPairs, 93);
  assert.equal(view.receipt.runtimeCredit, false);
  assert.equal(
    sha(bodies.get(hook)!),
    "9d22006fc19d80e1ebb8051cba2dc753a5bbf7756d87e4e5216bdf4fa2a46be4",
  );
  const current = authenticateNoMutatingComponentJoinPhysical(root);
  assert.equal(current.completeCurrentClosedBodies, 323);
  const currentHead = authenticateNoMutatingComponentJoinCommitted(root);
  assert.equal(currentHead, git(["rev-parse", "HEAD"]).toString().trim());
  actual.confirmSource();
  const declared = current.manifest.literalFrames.filter(
    (row) => row.source === "f20740ef6b0958ddb0bbbd64fd1b38210ea5e207",
  );
  assert.equal(declared.length, 1);
  const historical = authenticateJoinLiteralFrame(root, declared[0]);
  assert.equal(historical.frame.files.length, 683);
  actual.confirmSource();
  assert.deepEqual(
    view.retainedHostQualificationBefore(),
    fs.readFileSync(path.join(historical.root, hook)),
  );
  assert.equal(
    sha(view.retainedHostQualificationBefore()),
    "b19f6783e50429d2779efeb7d2d21c64dcd98a67810fc4f6a1ccc698febae337",
  );
  assert.deepEqual(
    retainedPhysicalHostBeforeComponentValues(historical.root),
    Buffer.from(capsule.rawBase64, "base64"),
  );
  assert.equal(current.sourceOnly, true);
  assert.equal(current.compiled, false);
  assert.equal(current.native, false);
  assert.equal(current.performance, false);
  assert.equal(current.currentOld111Receipt, null);
  assert.equal(current.oldMigrationMeasurement, null);
  assert.equal(current.oldSetupMeasurement, null);
  const borrowed = view.retainedHostQualificationBefore();
  borrowed[0] ^= 1;
  assert.equal(
    sha(view.retainedHostQualificationBefore()),
    "b19f6783e50429d2779efeb7d2d21c64dcd98a67810fc4f6a1ccc698febae337",
  );
});
void test("each entire current JS body and every missing or hybrid caller refuses before a source loan", () => {
  for (const file of javascriptSfcSourcePaths)
    for (const changed of [null, Buffer.concat([bodies.get(file)!, Buffer.from("\n")])])
      assert.throws(
        () =>
          authenticateJavaScriptSfcSource((name: string) => (name === file ? changed : read(name))),
        undefined,
        file,
      );
  const view = authenticateJavaScriptSfcSource(read);
  assert.throws(() =>
    authenticateJavaScriptSfcSource((file: string) =>
      file === hook ? view.retainedHostQualificationBefore() : read(file),
    ),
  );
});
void test("the complete current10 ASCII frame has genuine current bytes and refuses each cell or historical hybrid", () => {
  const entries = retainedAsciiCallerFrame("reader").map((entry) => {
    const body = fs.readFileSync(path.join(root, entry.file));
    assert.deepEqual(entry.body, body);
    return { ...entry, body, blob: blob(body) };
  });
  const result = qualifyAsciiOriginalCallers(entries);
  assert.equal(result.kind, "reader");
  assert.equal(result.wholeCallerBodies, 10);
  assert.equal(result.runtimeCredit, false);
  for (let i = 0; i < entries.length; i++) {
    const changed = Buffer.concat([entries[i].body, Buffer.from("\n")]);
    assert.throws(() =>
      qualifyAsciiOriginalCallers(
        entries.map((entry, n) =>
          n === i ? { ...entry, body: changed, blob: blob(changed) } : entry,
        ),
      ),
    );
  }
  const old = retainedAsciiCallerFrame("original");
  const different = entries.findIndex((entry, i) => !entry.body.equals(old[i].body));
  assert.notEqual(different, -1);
  assert.throws(() =>
    qualifyAsciiOriginalCallers(entries.map((entry, i) => (i === different ? old[i] : entry))),
  );
  assert.throws(() => originalAsciiCaller("foreign", entries[0].file, entries));
  assert.throws(() => originalAsciiCaller(result.originalSource, "unknown", entries));
});
const selectedHead = git(["rev-parse", "HEAD"]).toString().trim();
const treeRecords = git(["ls-tree", "-z", selectedHead, "--", ...javascriptSfcSourcePaths])
  .toString()
  .split("\0");
assert.equal(treeRecords.pop(), "");
assert.equal(treeRecords.length, 93);
const treeEntries = new Map(
  treeRecords.map((record) => {
    const match = /^100644 blob ([a-f0-9]{40})\t(.+)$/u.exec(record);
    assert.ok(match);
    return [match[2], match[1]];
  }),
);
const objects = javascriptSfcSourcePaths.map((file: string) => {
  const object = treeEntries.get(file);
  assert.ok(object);
  return object;
});
const batch = git(["cat-file", "--batch"], objects.join("\n") + "\n");
void test("the whole actual93 Git batch and copied raw/physical source bodies retain complete custody", () => {
  const decoded = decodeJavaScriptSfcRawBatch(objects, batch);
  assert.equal(decoded.size, 93);
  for (const file of javascriptSfcSourcePaths) {
    assert.deepEqual(decoded.get(file).body, bodies.get(file));
    const borrowed = actual(file);
    borrowed[0] ^= 1;
    assert.deepEqual(actual(file), bodies.get(file));
  }
  const copied = Buffer.from(batch);
  const independent = decodeJavaScriptSfcRawBatch(objects, copied);
  copied[copied.indexOf(10) + 1] ^= 1;
  assert.deepEqual(
    independent.get(javascriptSfcSourcePaths[0]).body,
    bodies.get(javascriptSfcSourcePaths[0]),
  );
  assert.throws(() => decodeJavaScriptSfcRawBatch(objects, copied));
  actual.confirmSource();
});
void test("genuine whole Git batch rejects missing, partial, extra, foreign-oid, type, size and trailing frames", () => {
  const end = batch.indexOf(10);
  const header = batch.subarray(0, end).toString();
  const size = Number(header.split(" ")[2]);
  const tail = batch.subarray(end + 1);
  const replaceHeader = (changed: string) => Buffer.concat([Buffer.from(changed + "\n"), tail]);
  const denied = [
    ["missing", batch.subarray(end + 1 + size + 1)],
    ["partial", batch.subarray(0, batch.length - 2)],
    ["extra", Buffer.concat([batch, batch.subarray(0, end + 1 + size + 1)])],
    ["foreign-oid", replaceHeader(header.replace(objects[0], "0".repeat(40)))],
    ["type", replaceHeader(header.replace(" blob ", " tree "))],
    ["size", replaceHeader(header.replace(/ [0-9]+$/u, " " + (size + 1)))],
    ["noncanonical-size", replaceHeader(header.replace(/ [0-9]+$/u, " 0" + size))],
    ["unsafe-size", replaceHeader(header.replace(/ [0-9]+$/u, " 9007199254740992"))],
    ["trailing", Buffer.concat([batch, Buffer.from("\n")])],
  ] as const;
  for (const [name, response] of denied)
    assert.throws(() => decodeJavaScriptSfcRawBatch(objects, response), undefined, name);
  assert.throws(() => decodeJavaScriptSfcRawBatch(objects.slice(1), batch));
  assert.throws(() => decodeJavaScriptSfcRawBatch([...objects, objects[0]], batch));
  assert.throws(() => decodeJavaScriptSfcRawBatch(["unknown", ...objects.slice(1)], batch));
  assert.deepEqual(
    decodeJavaScriptSfcRawBatch(objects, batch).get(javascriptSfcSourcePaths[0]).body,
    bodies.get(javascriptSfcSourcePaths[0]),
  );
});
void test("a real changed HEAD or replacement ref refuses after the same complete93 physical source read", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-js-source-owner-"));
  try {
    git(["init", "--quiet", fixture]);
    const objectRoot = path.resolve(
      root,
      git(["rev-parse", "--git-common-dir"]).toString().trim(),
      "objects",
    );
    fs.writeFileSync(path.join(fixture, ".git/objects/info/alternates"), objectRoot + "\n");
    git(["update-ref", "HEAD", selectedHead], undefined, fixture);
    for (const file of javascriptSfcSourcePaths) {
      const target = path.join(fixture, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, bodies.get(file)!, { mode: 0o644 });
    }
    const owner = javascriptSfcSourceReader(fixture, true);
    assert.equal(authenticateJavaScriptSfcSource(owner).receipt.bodyPairs, 93);
    const changedPath = path.join(fixture, javascriptSfcSourcePaths[0]);
    const changedBody = Buffer.from(bodies.get(javascriptSfcSourcePaths[0])!);
    changedBody[0] ^= 1;
    fs.writeFileSync(changedPath, changedBody);
    assert.throws(() => authenticateJavaScriptSfcSource(owner));
    fs.writeFileSync(changedPath, bodies.get(javascriptSfcSourcePaths[0])!);
    assert.equal(authenticateJavaScriptSfcSource(owner).receipt.bodyPairs, 93);
    const otherHead = git(["rev-parse", "HEAD^"], undefined, fixture).toString().trim();
    assert.notEqual(otherHead, selectedHead);
    git(["update-ref", "HEAD", otherHead], undefined, fixture);
    assert.throws(() => authenticateJavaScriptSfcSource(owner));
    git(["update-ref", "HEAD", selectedHead], undefined, fixture);
    owner.confirmSource();
    git(["update-ref", "refs/replace/" + selectedHead, selectedHead], undefined, fixture);
    assert.throws(() => owner.confirmSource());
    assert.throws(() => javascriptSfcSourceReader(fixture, true));
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});
void test("a real changed mode, missing owner or symlink cannot become actual current JS source", () => {
  const file = javascriptSfcSourcePaths[0];
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-js-source-mode-"));
  const target = path.join(fixture, file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, fs.readFileSync(path.join(root, file)), { mode: 0o644 });
  try {
    fs.chmodSync(target, 0o755);
    assert.throws(() => javascriptSfcSourceReader(fixture)(file));
    fs.chmodSync(target, 0o644);
    fs.unlinkSync(target);
    assert.throws(() => javascriptSfcSourceReader(fixture)(file));
    fs.symlinkSync("foreign-source", target);
    assert.throws(() => javascriptSfcSourceReader(fixture)(file));
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});
