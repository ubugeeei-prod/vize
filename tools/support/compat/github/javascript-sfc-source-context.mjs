import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

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
for (const row of rows) {
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
export const javascriptSfcSourcePaths = Object.freeze(rows.map((row) => row.path));
export const javascriptSfcProductionPaths = Object.freeze([...data.production]);
export const javascriptSfcContextPaths = Object.freeze([
  "tools/support/compat/github/javascript-sfc-source-context.mjs",
  data.callerContext,
  ...[dataName, ...data.parts.map(([name]) => name)].map(
    (name) => `tools/support/compat/github/${name}`,
  ),
]);
export const javascriptSfcBodyPairs = Object.freeze(
  rows.map((row) => Object.freeze([row.path, row.before?.sha256 ?? null, row.after.sha256])),
);

/** Real regular source bytes; optional raw Git comparison never fetches or follows replacements. */
export function javascriptSfcSourceReader(root, rawGit = false) {
  root = fs.realpathSync(root);
  const git = (args) =>
    execFileSync("git", ["--no-replace-objects", ...args], {
      cwd: root,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
      maxBuffer: 16 * 1024 * 1024,
    });
  if (rawGit) assert.equal(git(["for-each-ref", "--format=%(refname)", "refs/replace"]).length, 0);
  return (file) => {
    assert.ok(javascriptSfcSourcePaths.includes(file), file);
    let current = root;
    for (const part of file.split("/")) {
      current = path.join(current, part);
      assert.ok(!fs.lstatSync(current).isSymbolicLink(), current);
    }
    const body = regular(current);
    if (rawGit) {
      const entry = git(["ls-tree", "-z", "HEAD", "--", file]);
      const blob = createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
      assert.deepEqual(entry, Buffer.from(`100644 blob ${blob}\t${file}\0`));
      assert.deepEqual(git(["cat-file", "blob", blob]), body);
    }
    return body;
  };
}

/** Authenticate ALL current JS producer/corpus bodies before offering one source-only historical view. */
export function authenticateJavaScriptSfcSource(read) {
  const actual = new Map();
  for (const row of rows) {
    const body = read(row.path);
    assert.ok(Buffer.isBuffer(body), row.path);
    assert.deepEqual(body, Buffer.from(row.after.body, "base64"), row.path);
    actual.set(row.path, Buffer.from(body));
  }
  return {
    receipt: {
      authority: "complete-javascript-sfc-source-context",
      bodyPairs: rows.length,
      changedProductionBodies: data.production.length,
      sourceOnly: true,
      runtimeCredit: false,
      original400Credit: false,
      performanceCredit: false,
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
