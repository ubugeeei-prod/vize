import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateJavaScriptSfcSource,
  javascriptSfcSourceReader,
  javascriptSfcSourcePaths,
  javascriptSfcBodyPairs,
} from "../../tools/support/compat/github/javascript-sfc-source-context.mjs";
import { retainedJavaScriptSfcOptionsAfter } from "../../tools/support/compat/github/javascript-sfc-options-source.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const data = JSON.parse(
  fs.readFileSync(
    new URL(
      "../../tools/support/compat/github/javascript-sfc-options-source.json",
      import.meta.url,
    ),
  ),
);
const sha = (body) => createHash("sha256").update(body).digest("hex");
const read = javascriptSfcSourceReader(root, true);
const bodies = new Map(javascriptSfcSourcePaths.map((file) => [file, read(file)]));
void test("complete current93 source retains the exact original88 model without lending native credit", () => {
  const current = authenticateJavaScriptSfcSource(read);
  assert.equal(current.receipt.bodyPairs, 93);
  assert.equal(current.receipt.baseBodyPairs, 88);
  assert.equal(current.receipt.runtimeCredit, false);
  assert.equal(javascriptSfcBodyPairs.length, 93);
  for (const entry of data.existing) {
    assert.deepEqual(bodies.get(entry.path), Buffer.from(entry.currentAfter.body, "base64"));
    assert.deepEqual(
      current.retainedCapturedAfter(entry.path),
      Buffer.from(entry.retainedAfter.body, "base64"),
    );
    const borrowed = current.retainedCapturedAfter(entry.path);
    borrowed[0] ^= 1;
    assert.deepEqual(
      current.retainedCapturedAfter(entry.path),
      Buffer.from(entry.retainedAfter.body, "base64"),
    );
  }
  for (const row of data.supplemental) {
    const pair = javascriptSfcBodyPairs.find(([file]) => file === row.path);
    assert.deepEqual(pair, [row.path, row.before?.sha256 ?? null, row.after.sha256]);
    assert.throws(() => current.retainedCapturedAfter(row.path));
  }
  read.confirmSource();
});
void test("each repaired old or missing body and every supplemental body refuses before an original source loan", () => {
  for (const row of [...data.existing, ...data.supplemental]) {
    const values = [null, Buffer.concat([bodies.get(row.path), Buffer.from("\n")])];
    if (row.retainedAfter) values.push(Buffer.from(row.retainedAfter.body, "base64"));
    for (const changed of values)
      assert.throws(() =>
        authenticateJavaScriptSfcSource((file) => (file === row.path ? changed : bodies.get(file))),
      );
  }
  assert.throws(() =>
    retainedJavaScriptSfcOptionsAfter(
      data.existing[0].path,
      new Map([[data.existing[0].path, bodies.get(data.existing[0].path)]]),
    ),
  );
  const foreign = new Map(bodies);
  foreign.set(data.existing[0].path, Buffer.from("foreign"));
  assert.throws(() => retainedJavaScriptSfcOptionsAfter(data.existing[1].path, foreign));
});
void test("genuine private raw and physical modes missing symlinks mutations and replacements refuse", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-js-options-source-"));
  const git = (args, input) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: fixture,
      input,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
    });
  try {
    git(["init", "--quiet"]);
    for (const [file, body] of bodies) {
      const target = path.join(fixture, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, body, { mode: 0o644 });
    }
    git(["add", "--", "."]);
    git([
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "--quiet",
      "-m",
      "complete current93 source",
    ]);
    const head = git(["rev-parse", "HEAD"]).toString().trim();
    assert.equal(
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)).receipt.bodyPairs,
      93,
    );
    for (const row of [...data.existing, ...data.supplemental]) {
      const target = path.join(fixture, row.path),
        body = bodies.get(row.path);
      fs.unlinkSync(target);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.writeFileSync(target, body, { mode: 0o755 });
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.chmodSync(target, 0o644);
      fs.unlinkSync(target);
      fs.symlinkSync(path.join(root, row.path), target);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.unlinkSync(target);
      fs.writeFileSync(target, body, { mode: 0o644 });
      const changed = Buffer.concat([body, Buffer.from("\n")]);
      fs.writeFileSync(target, changed);
      git(["add", "--", row.path]);
      const tree = git(["write-tree"]).toString().trim();
      const unknown = git(
        [
          "-c",
          "user.name=Fixture",
          "-c",
          "user.email=fixture@example.invalid",
          "commit-tree",
          tree,
        ],
        "unknown matching raw/physical source\n",
      )
        .toString()
        .trim();
      git(["update-ref", "HEAD", unknown]);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      git(["update-ref", "HEAD", head]);
      git(["read-tree", head]);
      fs.writeFileSync(target, body);
    }
    git(["update-ref", "refs/replace/" + head, head]);
    assert.throws(() => javascriptSfcSourceReader(fixture, true));
    git(["update-ref", "-d", "refs/replace/" + head]);
    assert.equal(
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)).receipt.bodyPairs,
      93,
    );
    assert.equal(sha(bodies.get(data.existing[0].path)), data.existing[0].currentAfter.sha256);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});
