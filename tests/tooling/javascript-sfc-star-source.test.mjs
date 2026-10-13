import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  authenticateJavaScriptSfcSource,
  javascriptSfcSourcePaths,
  javascriptSfcSourceReader,
} from "../../tools/support/compat/github/javascript-sfc-source-context.mjs";
import {
  retainedJavaScriptSfcStarOptions,
  javascriptSfcStarSourceReceipt,
  javascriptSfcStarContextCells,
} from "../../tools/support/compat/github/javascript-sfc-star-source.mjs";

const root = path.resolve(import.meta.dirname, "../..");
const data = { cells: javascriptSfcStarContextCells() };
const sha = (body) => createHash("sha256").update(body).digest("hex");
const reader = javascriptSfcSourceReader(root, true);
const bodies = new Map(javascriptSfcSourcePaths.map((file) => [file, reader(file)]));

void test("all93 actual joined bodies precede either exact Options or original source-only loan", () => {
  const context = authenticateJavaScriptSfcSource(reader);
  assert.equal(context.receipt.bodyPairs, 93);
  assert.deepEqual(javascriptSfcStarSourceReceipt, {
    authority: "complete-javascript-star-current-source-context",
    bodyPairs: 93,
    beforeSource: "3a680d0a0b59b720af0d23a70a85cf56c5c4f175",
    wholeContextSourceBodies: 6,
    retainedOriginalSourceBodies: 88,
    retainedOptionsSourceBodies: 93,
    sourceOnly: true,
    runtimeCredit: false,
    original400Credit: false,
    performanceCredit: false,
  });
  const borrowed = retainedJavaScriptSfcStarOptions(bodies);
  assert.equal(borrowed.size, 93);
  for (const cell of data.cells) {
    assert.deepEqual(bodies.get(cell.path), Buffer.from(cell.currentAfter.body, "base64"));
    assert.deepEqual(
      context.retainedOptionsAfter(cell.path),
      Buffer.from(cell.optionsAfter.body, "base64"),
    );
    assert.deepEqual(borrowed.get(cell.path), Buffer.from(cell.optionsAfter.body, "base64"));
  }
  borrowed.get(data.cells[0].path)[0] ^= 1;
  borrowed.delete(data.cells[1].path);
  borrowed.set("unknown", Buffer.from("foreign"));
  const repeated = retainedJavaScriptSfcStarOptions(bodies);
  assert.equal(repeated.size, 93);
  for (const cell of data.cells)
    assert.deepEqual(repeated.get(cell.path), Buffer.from(cell.optionsAfter.body, "base64"));
  assert.throws(() => context.retainedOptionsAfter("unknown"));
  reader.confirmSource();
});

void test("missing changed and both complete historical hybrids refuse before a joined source loan", () => {
  for (const cell of data.cells) {
    const current = bodies.get(cell.path);
    for (const changed of [
      null,
      Buffer.concat([current, Buffer.from("\n")]),
      ...[cell.currentBefore, cell.optionsAfter, cell.retainedBefore].map((entry) =>
        Buffer.from(entry.body, "base64"),
      ),
    ]) {
      assert.notDeepEqual(changed, current);
      const foreign = new Map(bodies);
      foreign.set(cell.path, changed);
      assert.throws(() => authenticateJavaScriptSfcSource((file) => foreign.get(file)));
      assert.throws(() => retainedJavaScriptSfcStarOptions(foreign));
    }
  }
  for (const side of ["currentBefore", "optionsAfter"]) {
    const hybrid = new Map(bodies);
    for (const cell of data.cells) hybrid.set(cell.path, Buffer.from(cell[side].body, "base64"));
    assert.throws(() => authenticateJavaScriptSfcSource((file) => hybrid.get(file)));
    assert.throws(() => retainedJavaScriptSfcStarOptions(hybrid));
  }
  assert.throws(() =>
    retainedJavaScriptSfcStarOptions(
      new Map([[data.cells[0].path, bodies.get(data.cells[0].path)]]),
    ),
  );
  const extra = new Map(bodies);
  extra.set("unknown", Buffer.from("foreign"));
  assert.throws(() => retainedJavaScriptSfcStarOptions(extra));
});

void test("real raw and physical joined cells refuse old committed bodies modes symlinks and replacements", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-js-star-source-"));
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
    git(["add", "."]);
    git([
      "-c",
      "user.name=Source law",
      "-c",
      "user.email=law@invalid.example",
      "commit",
      "--quiet",
      "-m",
      "complete actual joined93",
    ]);
    const head = git(["rev-parse", "HEAD"]).toString().trim();
    assert.equal(
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)).receipt.bodyPairs,
      93,
    );
    for (const cell of data.cells) {
      const target = path.join(fixture, cell.path),
        current = bodies.get(cell.path);
      fs.chmodSync(target, 0o755);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.chmodSync(target, 0o644);
      fs.unlinkSync(target);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.symlinkSync(path.join(root, cell.path), target);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      fs.unlinkSync(target);
      fs.writeFileSync(target, Buffer.from(cell.optionsAfter.body, "base64"), { mode: 0o644 });
      git(["add", "--", cell.path]);
      const tree = git(["write-tree"]).toString().trim();
      const old = git(
        ["-c", "user.name=Source law", "-c", "user.email=law@invalid.example", "commit-tree", tree],
        "old committed cell\n",
      )
        .toString()
        .trim();
      git(["update-ref", "HEAD", old]);
      assert.throws(() =>
        authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)),
      );
      git(["update-ref", "HEAD", head]);
      git(["read-tree", head]);
      fs.writeFileSync(target, current);
      assert.equal(sha(fs.readFileSync(target)), cell.currentAfter.sha256);
    }
    git(["update-ref", "refs/replace/" + head, head]);
    assert.throws(() => javascriptSfcSourceReader(fixture, true));
    git(["update-ref", "-d", "refs/replace/" + head]);
    assert.equal(
      authenticateJavaScriptSfcSource(javascriptSfcSourceReader(fixture, true)).receipt.bodyPairs,
      93,
    );
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});
