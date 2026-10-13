import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  javascriptSfcOptionsRows,
  javascriptSfcOptionsContextPaths,
  javascriptSfcOptionsProductionPaths,
  retainedJavaScriptSfcOptionsAfter,
} from "./javascript-sfc-options-source.mjs";
import {
  javascriptSfcStarRows,
  javascriptSfcStarContextPaths,
  retainedJavaScriptSfcStarOptions,
} from "./javascript-sfc-star-source.mjs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const regular = (file) => {
  const entry = fs.lstatSync(file);
  assert.ok(entry.isFile() && !entry.isSymbolicLink(), file);
  assert.equal(entry.mode & 0o777, 0o644, file);
  return fs.readFileSync(file);
};
const dataName = "javascript-sfc-source-context.json";
const dataBytes = regular(new URL(dataName, import.meta.url));
assert.equal(sha(dataBytes), "c1013f07ab435417d46c10a83ea4b8e4ba816fe85e3a414af5bed6b90b6709ec");
const data = JSON.parse(dataBytes);
assert.equal(data.version, 1);
assert.equal(data.sourceOnly, true);
assert.equal(data.beforeSource, "a93e78147b0990c813e965a7868b32dda341eb77");
assert.equal(data.parts.length, 7);
const producerRows = data.parts.flatMap(([name, checksum], index) => {
  assert.equal(name, `javascript-sfc-source-context-${String(index).padStart(2, "0")}.json`);
  const bytes = regular(new URL(name, import.meta.url));
  assert.equal(sha(bytes), checksum);
  return JSON.parse(bytes);
});
assert.equal(producerRows.length, 83);
assert.deepEqual(
  producerRows.map((row) => row.path),
  data.paths,
);
assert.equal(new Set(data.paths).size, producerRows.length);
assert.equal(data.callerContext, "tests/_fixtures/ci/javascript-sfc-source-callers.json");
const callerBytes = regular(new URL(`../../../../${data.callerContext}`, import.meta.url));
assert.equal(sha(callerBytes), data.callerContextSHA256);
const callers = JSON.parse(callerBytes);
assert.equal(callers.length, 5);
assert.deepEqual(
  callers.map((row) => row.path),
  data.callerPaths,
);
const callerCell = (encoded, checksum) => {
  const body = Buffer.from(encoded, "base64");
  assert.equal(sha(body), checksum);
  return {
    body: encoded,
    sha256: checksum,
    byteLength: body.length,
    blob: createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
  };
};
const rows = [
  ...producerRows,
  ...callers.map((row) => {
    let restored = Buffer.from(row.afterBody, "base64").toString();
    for (const [_i, _j, start, end, before, after] of [...row.inverse].reverse()) {
      assert.equal(restored.slice(start, end), after);
      restored = restored.slice(0, start) + before + restored.slice(end);
    }
    assert.deepEqual(Buffer.from(restored), Buffer.from(row.beforeBody, "base64"));
    return {
      path: row.path,
      before: callerCell(row.beforeBody, row.beforeSHA256),
      after: callerCell(row.afterBody, row.afterSHA256),
    };
  }),
];
assert.equal(new Set(rows.map((row) => row.path)).size, 88);
const assertionName = "javascript-sfc-assertion-source-cell.json";
const assertionBytes = regular(new URL(assertionName, import.meta.url));
assert.equal(
  sha(assertionBytes),
  "a83983a7ae98ba2ee8c393be9f2dbe1ee402061d98de586a54a5f485965d98ac",
);
const assertion = JSON.parse(assertionBytes);
assert.equal(assertion.version, 1);
assert.equal(assertion.sourceOnly, true);
assert.equal(assertion.runtimeCredit, false);
assert.equal(assertion.original400Credit, false);
assert.equal(assertion.performanceCredit, false);
assert.equal(assertion.path, "crates/vize/tests/support/event_completion_witness.rs");
const retainedAssertion = rows.find((row) => row.path === assertion.path);
assert.deepEqual(assertion.retainedCapturedAfter, retainedAssertion.after);
const retainedBody = Buffer.from(retainedAssertion.after.body, "base64");
const correctedBody = Buffer.from(assertion.currentAfter.body, "base64");
assert.equal(retainedBody.toString().split(assertion.clause.before).length - 1, 1);
assert.deepEqual(
  Buffer.from(retainedBody.toString().replace(assertion.clause.before, assertion.clause.after)),
  correctedBody,
);
assert.equal(correctedBody.toString().split(assertion.clause.after).length - 1, 1);
assert.deepEqual(
  Buffer.from(correctedBody.toString().replace(assertion.clause.after, assertion.clause.before)),
  retainedBody,
);
const currentRows = javascriptSfcStarRows(
  javascriptSfcOptionsRows(
    rows.map((row) =>
      row.path === assertion.path ? { ...row, after: assertion.currentAfter } : row,
    ),
  ),
);

for (const row of currentRows) {
  assert.match(row.path, /^(?:crates|tests|tools|docs|\.github)\//u);
  assert.ok(!row.path.split("/").some((part) => ["", ".", ".."].includes(part)));
  for (const cell of [row.before, row.after]) {
    if (cell === null) continue;
    const body = Buffer.from(cell.body, "base64");
    assert.equal(body.toString("base64"), cell.body);
    assert.equal(body.length, cell.byteLength);
    assert.equal(sha(body), cell.sha256);
    assert.equal(
      createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      cell.blob,
    );
  }
  assert.ok(row.after);
}
export const javascriptSfcSourcePaths = Object.freeze(currentRows.map((row) => row.path));
export const javascriptSfcProductionPaths = Object.freeze([
  ...data.production,
  ...javascriptSfcOptionsProductionPaths,
]);
export const javascriptSfcContextPaths = Object.freeze([
  "tools/support/compat/github/javascript-sfc-source-context.mjs",
  ...javascriptSfcOptionsContextPaths,
  ...javascriptSfcStarContextPaths,
  data.callerContext,
  "tools/support/compat/github/" + assertionName,
  ...[dataName, ...data.parts.map(([name]) => name)].map(
    (name) => `tools/support/compat/github/${name}`,
  ),
]);
export const javascriptSfcBodyPairs = Object.freeze(
  currentRows.map((row) => Object.freeze([row.path, row.before?.sha256 ?? null, row.after.sha256])),
);

/** Decode the whole real Git batch for this finite scope; this alone grants no source authority. */
export function decodeJavaScriptSfcRawBatch(objects, batch) {
  assert.ok(Buffer.isBuffer(batch));
  assert.equal(objects.length, javascriptSfcSourcePaths.length);
  for (const object of objects) assert.match(object, /^[a-f0-9]{40}$/u);
  const decoded = new Map();
  let cursor = 0;
  for (let i = 0; i < objects.length; i++) {
    const newline = batch.indexOf(10, cursor);
    assert.ok(newline >= cursor);
    const header = batch.subarray(cursor, newline);
    assert.deepEqual(Buffer.from(header.toString()), header);
    const match = /^([a-f0-9]{40}) blob (0|[1-9][0-9]*)$/u.exec(header.toString());
    assert.ok(match && match[1] === objects[i]);
    const size = Number(match[2]);
    assert.ok(Number.isSafeInteger(size) && size >= 0);
    const start = newline + 1,
      end = start + size;
    assert.ok(Number.isSafeInteger(end) && end < batch.length && batch[end] === 10);
    const body = Buffer.from(batch.subarray(start, end));
    const blob = createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
    assert.equal(blob, objects[i]);
    decoded.set(javascriptSfcSourcePaths[i], { blob, body });
    cursor = end + 1;
  }
  assert.equal(cursor, batch.length, "whole framed raw source response");
  return decoded;
}

/** Real regular source bytes; optional raw Git comparison never fetches or follows replacements. */
export function javascriptSfcSourceReader(root, rawGit = false) {
  root = fs.realpathSync(root);
  const git = (args, input) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: root,
      input,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
      maxBuffer: 16 * 1024 * 1024,
    });
  let selectedHead;
  const raw = new Map();
  if (rawGit) {
    assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
    selectedHead = git(["rev-parse", "HEAD"]).toString().trim();
    assert.match(selectedHead, /^[a-f0-9]{40}$/u);
    const tree = git(["ls-tree", "-z", selectedHead, "--", ...javascriptSfcSourcePaths]);
    assert.deepEqual(Buffer.from(tree.toString()), tree);
    const entries = new Map();
    assert.equal(tree.at(-1), 0, "whole raw source tree response");
    const records = tree.toString().split("\0");
    assert.equal(records.pop(), "");
    assert.equal(records.length, javascriptSfcSourcePaths.length);
    for (const record of records) {
      const match = /^100644 blob ([a-f0-9]{40})\t(.+)$/u.exec(record);
      assert.ok(match, "one regular current raw source record");
      assert.ok(javascriptSfcSourcePaths.includes(match[2]) && !entries.has(match[2]));
      entries.set(match[2], match[1]);
    }
    assert.equal(entries.size, javascriptSfcSourcePaths.length);
    const objects = javascriptSfcSourcePaths.map((file) => entries.get(file));
    const batch = git(["cat-file", "--batch"], objects.join("\n") + "\n");
    for (const [file, value] of decodeJavaScriptSfcRawBatch(objects, batch)) raw.set(file, value);
  }
  const confirmSource = () => {
    if (!rawGit) return;
    assert.equal(
      git(["rev-parse", "HEAD"]).toString().trim(),
      selectedHead,
      "same complete source owner",
    );
    assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  };
  const read = (file) => {
    assert.ok(javascriptSfcSourcePaths.includes(file), file);
    let current = root;
    for (const part of file.split("/")) {
      current = path.join(current, part);
      assert.ok(!fs.lstatSync(current).isSymbolicLink(), current);
    }
    const body = regular(current);
    if (rawGit) {
      const value = raw.get(file);
      const blob = createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
      assert.equal(blob, value.blob, file);
      assert.deepEqual(body, value.body, file);
    }
    return body;
  };
  return Object.assign(read, { confirmSource });
}

/** Authenticate ALL current JS producer/corpus bodies before offering one source-only historical view. */
export function authenticateJavaScriptSfcSource(read) {
  const actual = new Map();
  for (const row of currentRows) {
    const body = read(row.path);
    assert.ok(Buffer.isBuffer(body), row.path);
    assert.deepEqual(body, Buffer.from(row.after.body, "base64"), row.path);
    actual.set(row.path, Buffer.from(body));
  }
  read.confirmSource?.();
  return {
    receipt: {
      authority: "complete-javascript-sfc-source-context",
      bodyPairs: currentRows.length,
      baseBodyPairs: rows.length,
      changedProductionBodies: javascriptSfcProductionPaths.length,
      sourceOnly: true,
      runtimeCredit: false,
      original400Credit: false,
      performanceCredit: false,
    },
    retainedHostQualificationBefore() {
      const file = "tests/performance/support/warm-type-backed-host-qualifications.ts";
      assert.ok(data.callerPaths.includes(file) && actual.has(file));
      const row = rows.find((row) => row.path === file);
      return Buffer.from(row.before.body, "base64");
    },
    retainedCapturedAfter(file) {
      assert.ok(
        rows.some((row) => row.path === file),
        "one original88 source owner",
      );
      return retainedJavaScriptSfcOptionsAfter(file, retainedJavaScriptSfcStarOptions(actual));
    },
    retainedOptionsAfter(file) {
      assert.ok(actual.has(file), file);
      return Buffer.from(retainedJavaScriptSfcStarOptions(actual).get(file));
    },
    historicalAfter(file) {
      assert.ok(actual.has(file), file);
      const row = rows.find((row) => row.path === file);
      return Buffer.from(
        data.historicalLexicalAfter.includes(file)
          ? Buffer.from(row.before.body, "base64")
          : actual.get(file),
      );
    },
  };
}
