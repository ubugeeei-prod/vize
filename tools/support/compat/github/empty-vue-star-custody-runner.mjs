import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { gunzipSync } from "node:zlib";
import { after } from "node:test";
import { pathToFileURL } from "node:url";
import {
  authenticateJavaScriptSfcSource,
  javascriptSfcSourceReader,
} from "./javascript-sfc-source-context.mjs";
import { authenticateEmptyVueStarJoinedSource } from "./empty-vue-star-javascript-source.mjs";
import { ensureHostedLiteralSource } from "../../../../tests/performance/support/warm-type-backed-git-reader-literal-source.ts";

const source = "3a680d0a0b59b720af0d23a70a85cf56c5c4f175";
const law = "tests/tooling/warm-type-backed-empty-vue-star-custody.test.ts";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const object = (kind, body) =>
  createHash("sha1")
    .update(kind + " " + body.length + "\0")
    .update(body)
    .digest("hex");
const git = (root, args, input) =>
  execFileSync(
    "git",
    ["--no-replace-objects", "-c", "gc.auto=0", "-c", "maintenance.auto=false", ...args],
    {
      cwd: root,
      input,
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
    },
  );
function objects(root, rows) {
  if (!rows.length) return new Map();
  const packet = git(root, ["cat-file", "--batch"], rows.map((row) => row.blob).join("\n") + "\n");
  const result = new Map();
  let offset = 0;
  for (const row of rows) {
    const stop = packet.indexOf(10, offset);
    const match = /^([a-f0-9]{40}) blob ([0-9]+)$/u.exec(packet.subarray(offset, stop).toString());
    assert(match && match[1] === row.blob, "whole selected original blob: " + row.path);
    const body = Buffer.from(packet.subarray(stop + 1, stop + 1 + Number(match[2])));
    assert.equal(body.length, Number(match[2]));
    assert.equal(object("blob", body), row.blob);
    offset = stop + 1 + body.length;
    assert.equal(packet[offset++], 10);
    result.set(row.path, body);
  }
  assert.equal(offset, packet.length);
  return result;
}
function entries(root, commit, names = []) {
  const bytes = git(root, [
    "ls-tree",
    "-r",
    "-z",
    commit,
    ...(names.length ? ["--", ...names] : []),
  ]);
  assert.deepEqual(Buffer.from(bytes.toString()), bytes);
  return new Map(
    bytes
      .toString()
      .split("\0")
      .filter(Boolean)
      .map((entry) => {
        const match = /^(100644|100755|120000|160000) (blob|commit) ([a-f0-9]{40})\t(.+)$/u.exec(
          entry,
        );
        assert(match, "complete original tree entry");
        return [match[4], { path: match[4], mode: match[1], type: match[2], blob: match[3] }];
      }),
  );
}
function select(root, frame) {
  const tree = entries(root, source),
    cache = new Map(),
    chosen = new Set();
  const read = (names) => {
    const missing = names.filter((name) => !cache.has(name));
    for (const name of missing) {
      const row = tree.get(name);
      assert(row && row.mode === "100644" && row.type === "blob", "regular selected path: " + name);
    }
    for (const [name, body] of objects(
      root,
      missing.map((name) => tree.get(name)),
    ))
      cache.set(name, body);
    return names.map((name) => cache.get(name));
  };
  const json = (name) => JSON.parse(read([name])[0]);
  const add = (names) => names.forEach((name) => chosen.add(name));
  const prefix = "tests/performance/support/";
  const starName = prefix + "warm-type-backed-empty-vue-star-manifest.json";
  const star = json(starName),
    correction = json(star.correction.file);
  add([starName, star.correction.file, ...star.parts.map((row) => row.file)]);
  star.parts.forEach((part) => add(json(part.file).map(([name]) => name)));
  add(correction.additions.map(([name]) => name));
  const joinName =
    "tests/_fixtures/differential/linter/no-mutating-props-options-uploader/source-custody/component-main-join/successor.json";
  const join = json(joinName);
  add([
    joinName,
    join.externalMaintenanceLaw.path,
    ...join.current.map((row) => row.path),
    ...join.literalFrames.map((row) => row.path),
  ]);
  const componentName = prefix + "warm-type-backed-component-value-manifest.json";
  const component = json(componentName);
  add([componentName, component.preservedBodies]);
  add(
    [...component.files, ...component.supplementalBodies, ...json(component.preservedBodies)].map(
      ([name]) => name,
    ),
  );
  add(
    [...tree.keys()].filter(
      (name) =>
        name.startsWith(
          "tests/_fixtures/differential/linter/component-import-value-host-before/",
        ) ||
        (name.startsWith(prefix + "warm-type-backed-git-reader-") && name.endsWith(".json")),
    ),
  );
  add([
    law,
    prefix + "warm-type-backed-empty-vue-star-host.ts",
    prefix + "warm-type-backed-host-qualifications.ts",
    "tests/tooling/warm-type-backed-empty-vue-star-host.test.ts",
    "tests/tooling/support/empty-vue-star-host-custody.ts",
  ]);
  let pending = [...chosen];
  while (pending.length) {
    const names = pending;
    pending = [];
    const bodies = read(names);
    names.forEach((name, index) => {
      if (!/\.(?:ts|mjs|js)$/u.test(name)) return;
      const body = bodies[index].toString();
      const relative = [
        ...body.matchAll(/(?:from\s*|import\s*)["'](\.[^"']+)["']/gu),
        ...body.matchAll(/new URL\(\s*["'](\.[^"']+)["']\s*,\s*import.meta.url/gu),
      ];
      const candidates = relative.map((match) =>
        path.posix.normalize(path.posix.join(path.posix.dirname(name), match[1])),
      );
      candidates.push(
        ...[...body.matchAll(/["']((?:tests|tools|crates|docs)\/[^"'\n]+)["']/gu)].map(
          (match) => match[1],
        ),
      );
      candidates.push(
        ...[...body.matchAll(/["']([A-Za-z0-9_.-]+\.(?:json|gz|txt))["']/gu)].map((match) =>
          path.posix.join(path.posix.dirname(name), match[1]),
        ),
      );
      for (const candidate of candidates) {
        const row = tree.get(candidate);
        if (row?.mode !== "100644" || row.type !== "blob" || chosen.has(candidate)) continue;
        chosen.add(candidate);
        pending.push(candidate);
      }
    });
  }
  const records = [...chosen]
    .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
    .map((name) => {
      const row = tree.get(name),
        body = cache.get(name);
      return [name, row.mode, row.blob, body.length, sha(body)];
    });
  assert.equal(records.length, frame.files);
  assert.equal(
    records.reduce((sum, row) => sum + row[3], 0),
    frame.byteLength,
  );
  assert.equal(sha(JSON.stringify(records)), frame.orderedModeBlobLengthSHA256);
  assert.deepEqual(
    records.find(([name]) => name === law),
    [law, "100644", frame.law.blob, frame.law.byteLength, frame.law.sha256],
  );
  return { records, cache, join };
}
function provision(root, selected) {
  const declared = selected.join.literalFrames.find(
    (row) => row.source === "f20740ef6b0958ddb0bbbd64fd1b38210ea5e207",
  );
  assert(declared);
  const frame = JSON.parse(gunzipSync(selected.cache.get(declared.path)));
  for (const row of frame.requiredGitSources) {
    ensureHostedLiteralSource(root, row.source);
    const commit = git(root, ["cat-file", "commit", row.source]);
    assert.equal(object("commit", commit), row.source);
    assert.equal(commit.length, row.bytes);
    assert.equal(sha(commit), row.sha256);
    const names = [...row.files.map((file) => file.path), ...row.absentPaths];
    const actual =
      row.completeTree || names.length
        ? entries(root, row.source, row.completeTree ? [] : names)
        : new Map();
    for (const name of row.absentPaths)
      assert(!actual.has(name), "genuine historical absence: " + name);
    const expected = row.files.map((file) => {
      const entry = actual.get(file.path);
      assert(entry && entry.mode === "100644" && entry.type === "blob");
      assert.equal(entry.blob, file.gitBlob);
      return entry;
    });
    const bodies = objects(root, expected);
    for (const file of row.files) {
      const body = bodies.get(file.path);
      assert.equal(body.length, file.bytes);
      assert.equal(sha(body), file.sha256);
    }
  }
}
/** Both real current source gates precede the whole original eight-law source fixture. */
export async function runRetainedEmptyVueStarCustody(root) {
  root = fs.realpathSync(root);
  const js = authenticateJavaScriptSfcSource(javascriptSfcSourceReader(root, true));
  assert.equal(js.receipt.bodyPairs, 93);
  const current = authenticateEmptyVueStarJoinedSource(root);
  assert.equal(current.completeCurrentBodies, 322);
  const data = JSON.parse(
    fs.readFileSync(new URL("empty-vue-star-javascript-source.json", import.meta.url)),
  );
  const frame = data.retainedCustodyFrame;
  assert.equal(frame.source, source);
  assert.equal(frame.tree, "577d549f8a9c6b0a51f6bf99bc23fb35eec5942a");
  const selected = select(root, frame);
  provision(root, selected);
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-star-original-custody-"));
  const keys = [
    "GIT_ALLOW_PROTOCOL",
    "GIT_NO_LAZY_FETCH",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_KEY_0",
    "GIT_CONFIG_VALUE_0",
    "GIT_CONFIG_KEY_1",
    "GIT_CONFIG_VALUE_1",
  ];
  const originalEnvironment = new Map(keys.map((key) => [key, process.env[key]]));
  const restore = () => {
    for (const [key, value] of originalEnvironment) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  };
  let cleanupRegistered = false;
  try {
    const common = git(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    const pool = fs.realpathSync(path.resolve(root, common, "objects"));
    assert(!/[\r\n]/u.test(pool));
    fs.mkdirSync(path.join(directory, ".git/objects/info"), { recursive: true });
    fs.mkdirSync(path.join(directory, ".git/refs"));
    fs.writeFileSync(
      path.join(directory, ".git/config"),
      '[core]\nrepositoryformatversion = 0\nbare = false\n[gc]\nauto = 0\n[maintenance]\nauto = false\n[remote "origin"]\nurl = https://github.com/ubugeeei-prod/vize\n',
    );
    fs.writeFileSync(path.join(directory, ".git/HEAD"), source + "\n");
    fs.writeFileSync(path.join(directory, ".git/objects/info/alternates"), pool + "\n");
    for (const [file] of selected.records) {
      const target = path.join(directory, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, selected.cache.get(file), { mode: 0o644, flag: "wx" });
    }
    assert.equal(git(directory, ["rev-parse", "HEAD"]).toString().trim(), source);
    assert.equal(git(directory, ["rev-parse", "HEAD^{tree}"]).toString().trim(), frame.tree);
    const original = selected.cache.get(law).toString();
    assert.deepEqual(
      [...original.matchAll(/(?:void )?test\("([^"]+)"/gu)].map((match) => match[1]),
      frame.tests,
    );
    Object.assign(process.env, {
      GIT_ALLOW_PROTOCOL: "",
      GIT_NO_LAZY_FETCH: "1",
      GIT_CONFIG_COUNT: "2",
      GIT_CONFIG_KEY_0: "gc.auto",
      GIT_CONFIG_VALUE_0: "0",
      GIT_CONFIG_KEY_1: "maintenance.auto",
      GIT_CONFIG_VALUE_1: "false",
    });
    after(() => {
      try {
        assert.equal(git(directory, ["rev-parse", "HEAD"]).toString().trim(), source);
        for (const [file, , , bytes, digest] of selected.records) {
          const target = path.join(directory, file),
            stat = fs.lstatSync(target);
          assert(stat.isFile() && !stat.isSymbolicLink() && (stat.mode & 0o777) === 0o644);
          const body = fs.readFileSync(target);
          assert.equal(body.length, bytes);
          assert.equal(sha(body), digest);
        }
        assert.equal(authenticateEmptyVueStarJoinedSource(root).head, current.head);
        assert.equal(
          authenticateJavaScriptSfcSource(javascriptSfcSourceReader(root, true)).receipt.bodyPairs,
          93,
        );
      } finally {
        restore();
        fs.rmSync(directory, { recursive: true, force: true });
      }
    });
    cleanupRegistered = true;
    await import(pathToFileURL(path.join(directory, law)).href);
    return Object.freeze({
      sourceOnly: true,
      currentBodies: 322,
      retainedSource: source,
      retainedFiles: frame.files,
      registeredOriginalTests: 8,
      nativeCredit: false,
      original400Credit: false,
      performanceCredit: false,
    });
  } finally {
    if (!cleanupRegistered) {
      restore();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }
}
