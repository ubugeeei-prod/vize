import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { qualifyAsciiSymbolHost } from "../performance/support/warm-type-backed-ascii-symbol-host.ts";
import { qualifyHostMoves } from "../performance/support/warm-type-backed-host-qualifications.ts";
import { originalAsciiCaller } from "./support/ascii-symbol-original-callers.ts";

const root = path.resolve(import.meta.dirname, "../..");
const manifest = JSON.parse(
  fs.readFileSync(
    new URL("../performance/support/warm-type-backed-ascii-symbol-host.json", import.meta.url),
    "utf8",
  ),
) as {
  originalSource: string;
  files: Array<[string, string | null, string]>;
  preservedBodies: Array<[string, string]>;
};
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const rawGit = (cwd: string, args: string[], input?: string | Buffer) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd, input });

// This law independently reads literal regular blobs; it does not borrow the host reader.
function rawBody(cwd: string, revision: string, file: string): Buffer | null {
  assert.match(revision, /^[a-f0-9]{40}$/u);
  const entry = rawGit(cwd, ["ls-tree", "-z", revision, "--", file]);
  if (entry.length === 0) return null;
  const record = entry.toString("utf8");
  assert.ok(Buffer.from(record).equals(entry));
  const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(record);
  assert.ok(match, `one regular source blob is required: ${file}`);
  assert.equal(match[2], file);
  return rawGit(cwd, ["cat-file", "blob", match[1]]);
}

function physicalBody(cwd: string, file: string): Buffer {
  const target = path.join(cwd, file);
  const stat = fs.lstatSync(target);
  assert.ok(stat.isFile(), `physical regular source required: ${file}`);
  assert.equal(stat.mode & 0o111, 0, `nonexecuted source required: ${file}`);
  return fs.readFileSync(target);
}

assert.match(manifest.originalSource, /^[a-f0-9]{40}$/u);
const lookup = () =>
  rawGit(
    root,
    ["cat-file", "--batch-check=%(objectname) %(objecttype)"],
    `${manifest.originalSource}\n`,
  ).toString("utf8");
const originalLookup = lookup();
if (originalLookup === `${manifest.originalSource} missing\n`) {
  rawGit(root, ["fetch", "--no-tags", "--depth=1", "origin", manifest.originalSource]);
}
assert.equal(lookup(), `${manifest.originalSource} commit\n`);

const pairs: Array<[string, string | null, string]> = [
  ...manifest.files,
  ...manifest.preservedBodies.map(([file, original]): [string, string, string] => [
    file,
    original,
    original,
  ]),
];
const actual = new Map<string, { before: string | null; after: string }>();
for (const [file, before, after] of pairs) {
  const original = rawBody(root, manifest.originalSource, file);
  const current = physicalBody(root, file);
  actual.set(file, { before: original === null ? null : sha(original), after: sha(current) });
  assert.equal(actual.get(file)!.before, before, `literal whole original body: ${file}`);
  assert.equal(actual.get(file)!.after, after, `physical whole current body: ${file}`);
}
const production = manifest.files.map(([file]) => file);
const digest = (side: "before" | "after", file: string) => actual.get(file)?.[side] ?? null;

test("the two actual producer/law bodies and sixteen whole unchanged bodies grant only source qualification", () => {
  const result = qualifyAsciiSymbolHost(production, new Set(), digest);
  assert.ok(result);
  assert.deepEqual(result.files, production);
  assert.equal(result.completeBodyPairs, 2);
  assert.equal(result.preservedBodyPairs, 16);
  assert.equal(actual.size, 18);
  assert.equal(result.runtimeCredit, false);
  assert.equal(result.performanceCredit, false);
});

test("all thirty-six complete pair sides and every required presence refuse drift", () => {
  let mutatedSides = 0;
  for (const [file] of pairs)
    for (const side of ["before", "after"] as const) {
      mutatedSides++;
      for (const changed of ["f".repeat(64), null]) {
        if (changed === digest(side, file)) continue;
        assert.equal(
          qualifyAsciiSymbolHost(production, new Set(), (s, name) =>
            s === side && name === file ? changed : digest(s, name),
          ),
          null,
          `${side}:${file}:${changed}`,
        );
      }
    }
  assert.equal(mutatedSides, 36);
});

test("unknown, missing, duplicate and inherited footprints cannot lend required authority", () => {
  assert.equal(qualifyAsciiSymbolHost([], new Set(), digest), null);
  assert.equal(qualifyAsciiSymbolHost(production, new Set(production), digest), null);
  assert.equal(
    qualifyAsciiSymbolHost([...production, "crates/vize/src/unknown.rs"], new Set(), digest),
    null,
  );
  for (const file of production) {
    const missing = production.filter((name) => name !== file);
    assert.equal(qualifyAsciiSymbolHost(missing, new Set(), digest), null, file);
    assert.equal(qualifyAsciiSymbolHost(missing, new Set([file]), digest), null, file);
    assert.equal(qualifyAsciiSymbolHost([...production, file], new Set(), digest), null, file);
  }
  const inherited = new Set(["crates/vize_canon/src/lsp_client.rs"]);
  const result = qualifyAsciiSymbolHost([...production, ...inherited], inherited, digest);
  assert.ok(result);
  assert.deepEqual(result.files, production);
  assert.equal(qualifyAsciiSymbolHost([...inherited], inherited, digest), null);
  for (const [file] of manifest.preservedBodies) {
    assert.equal(
      qualifyAsciiSymbolHost([...production, file], new Set([file]), digest),
      null,
      file,
    );
    assert.equal(
      qualifyAsciiSymbolHost([...production, ...inherited], inherited, (side, name) =>
        name === file ? null : digest(side, name),
      ),
      null,
      file,
    );
  }
});

test("composition keeps old null/disabled receipts literal and grants no cap or foreign exemption", () => {
  const oldNull = { pathHostMove: null, timingHostMove: null, defaultMigrationHost: null };
  assert.deepEqual(qualifyHostMoves([], new Set(), true, digest), oldNull);
  assert.deepEqual(
    qualifyHostMoves(production, new Set(), false, () => {
      throw Error("disabled digest called");
    }),
    oldNull,
  );
  assert.deepEqual(qualifyHostMoves(production, new Set(), true, digest), {
    ...oldNull,
    asciiSymbolHost: qualifyAsciiSymbolHost(production, new Set(), digest),
  });
  for (const file of ["Cargo.toml", "tests/performance/support/warm-type-backed-controls.ts"])
    assert.deepEqual(qualifyHostMoves([...production, file], new Set(), true, digest), oldNull);
});

test("removing only the bounded composition hook restores its complete original Git body", () => {
  const callerEntries = currentAsciiCallers();
  const file = "tests/performance/support/warm-type-backed-host-qualifications.ts";
  let current = physicalBody(root, file).toString("utf8");
  for (const insertion of [
    'import { qualifyAsciiSymbolHost } from "./warm-type-backed-ascii-symbol-host.ts";\n',
    "  const asciiSymbolHost = enabled\n    ? qualifyAsciiSymbolHost(production, alreadyQualified, digest)\n    : null;\n",
    "    ...(asciiSymbolHost ? { asciiSymbolHost } : {}),\n",
  ]) {
    assert.equal(current.split(insertion).length, 2);
    current = current.replace(insertion, "");
  }
  assert.deepEqual(Buffer.from(current), rawBody(root, manifest.originalSource, file));
  for (const name of [
    "warm-type-backed-path-host.ts",
    "warm-type-backed-path-host-manifest.json",
    "warm-type-backed-default-migration-host.ts",
    "warm-type-backed-default-migration-manifest.json",
    "warm-type-backed-lexical-dollar-host.ts",
    "warm-type-backed-lexical-dollar-manifest.json",
    "warm-type-backed-workflow.ts",
    "warm-type-backed-source.ts",
    "warm-type-backed-controls.ts",
    "warm-type-backed-requests.ts",
  ]) {
    const preserved = `tests/performance/support/${name}`;
    assert.deepEqual(
      originalAsciiCaller(manifest.originalSource, preserved, callerEntries),
      rawBody(root, manifest.originalSource, preserved),
    );
  }
});

test("the independent reader ignores commit/blob replacements and rejects executable/symlink owners", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-ascii-symbol-source-"));
  const git = (args: string[], input?: string | Buffer) => rawGit(fixture, args, input);
  const object = (body: Buffer | string) =>
    git(["hash-object", "-w", "--stdin"], body).toString().trim();
  const commit = (mode: string, blob: string) => {
    const tree = git(["mktree"], `${mode} blob ${blob}\tsource.rs\n`).toString().trim();
    return git(
      ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit-tree", tree],
      "source fixture\n",
    )
      .toString()
      .trim();
  };
  try {
    git(["init", "--quiet"]);
    const body = Buffer.from('// complete raw body\r\nconst KEY: &str = "日本語";\r\n');
    const originalBlob = object(body);
    const changedBlob = object("replacement body\n");
    const original = commit("100644", originalBlob);
    const changed = commit("100644", changedBlob);
    git(["replace", original, changed]);
    git(["replace", originalBlob, changedBlob]);
    assert.deepEqual(rawBody(fixture, original, "source.rs"), body);
    assert.equal(rawBody(fixture, original, "absent.rs"), null);
    assert.throws(() => rawBody(fixture, "HEAD", "source.rs"));
    for (const mode of ["100755", "120000"])
      assert.throws(
        () => rawBody(fixture, commit(mode, originalBlob), "source.rs"),
        /regular source/u,
      );
    fs.writeFileSync(path.join(fixture, "source.rs"), body, { mode: 0o755 });
    assert.throws(() => physicalBody(fixture, "source.rs"), /nonexecuted source/u);
    fs.symlinkSync("source.rs", path.join(fixture, "link.rs"));
    assert.throws(() => physicalBody(fixture, "link.rs"), /physical regular source/u);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});

// Authenticate all current callers independently of the historical positive replay.
function currentAsciiCallers() {
  const head = rawGit(root, ["rev-parse", "HEAD"]).toString().trim();
  return [
    "warm-type-backed-path-host.ts",
    "warm-type-backed-path-host-manifest.json",
    "warm-type-backed-default-migration-host.ts",
    "warm-type-backed-default-migration-manifest.json",
    "warm-type-backed-lexical-dollar-host.ts",
    "warm-type-backed-lexical-dollar-manifest.json",
    "warm-type-backed-workflow.ts",
    "warm-type-backed-source.ts",
    "warm-type-backed-controls.ts",
    "warm-type-backed-requests.ts",
  ].map((name) => {
    const file = `tests/performance/support/${name}`;
    const body = physicalBody(root, file);
    assert.deepEqual(body, rawBody(root, head, file));
    const entry = rawGit(root, ["ls-tree", "-z", head, "--", file]).toString();
    const matched = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(entry);
    assert.ok(matched);
    assert.equal(matched[2], file);
    return { file, body, mode: "100644", blob: matched[1] };
  });
}
