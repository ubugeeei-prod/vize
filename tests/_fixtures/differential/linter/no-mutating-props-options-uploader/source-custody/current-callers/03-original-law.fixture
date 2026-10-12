import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import {
  gitReaderNoDeepChanges,
  gitReaderNoDeepGuard,
  gitReaderNoDeepPairs,
  gitReaderNoDeepSource,
  historicalIncomingSource,
  prepareLiteralSource,
  qualifyGitReaderNoDeepHarness,
} from "../performance/support/warm-type-backed-git-reader-nodeep.ts";
import {
  gitReaderSetupChanges,
  gitReaderSetupGuard,
  gitReaderSetupPairs,
  qualifyGitReaderSetupHarness,
  rawFirstParentProof,
} from "../performance/support/warm-type-backed-git-reader-setup.ts";

const root = path.resolve(import.meta.dirname, "../..");
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const rawGit = (cwd: string, args: string[], input?: string | Buffer) =>
  execFileSync("git", ["--no-replace-objects", ...args], { cwd, input });
function rawBody(cwd: string, source: string, file: string): Buffer | null {
  assert.match(source, /^[a-f0-9]{40}$/u);
  const entry = rawGit(cwd, ["ls-tree", "-z", source, "--", file]);
  if (entry.length === 0) return null;
  const text = entry.toString("utf8");
  assert.ok(Buffer.from(text).equals(entry));
  const match = /^100644 blob ([a-f0-9]{40})\t(.+)\0$/u.exec(text);
  assert.ok(match, `one regular source blob required: ${file}`);
  assert.equal(match[2], file);
  return rawGit(cwd, ["cat-file", "blob", match[1]]);
}
function physicalBody(cwd: string, file: string): Buffer {
  const target = path.join(cwd, file);
  const stat = fs.lstatSync(target);
  assert.ok(stat.isFile() && (stat.mode & 0o111) === 0, `regular physical source: ${file}`);
  return fs.readFileSync(target);
}
const pairs = new Map(
  gitReaderNoDeepPairs.map(([file, before, after]) => [file, { before, after }]),
);
const digest = (side: "before" | "after", file: string) => pairs.get(file)![side];

test("the actual setup successor and final NoDeep9591 whole twelve have their own source authority", async () => {
  prepareLiteralSource(root, gitReaderNoDeepSource);
  prepareLiteralSource(root, historicalIncomingSource);
  const actual = (side: "before" | "after", file: string) => {
    const body =
      side === "before" ? rawBody(root, gitReaderNoDeepSource, file) : physicalBody(root, file);
    return body === null ? null : sha(body);
  };
  const head = rawGit(root, ["rev-parse", "HEAD"]).toString().trim();
  assert.match(head, /^[a-f0-9]{40}$/u);
  rawFirstParentProof(root, gitReaderNoDeepSource, head);
  const result = qualifyGitReaderSetupHarness(gitReaderSetupChanges, new Set(), actual);
  assert.ok(result);
  assert.equal(result.completeBodyPairs, gitReaderSetupPairs.length);
  assert.equal(result.namedFamilyBodies, 87);
  assert.equal(result.acceptanceCredit, "none");
  assert.equal(rawBody(root, gitReaderNoDeepSource, gitReaderNoDeepGuard), null);
  physicalBody(root, gitReaderNoDeepGuard);
  assert.deepEqual(
    rawBody(root, head, gitReaderNoDeepGuard),
    physicalBody(root, gitReaderNoDeepGuard),
  );
  for (const [file, , expected] of gitReaderSetupPairs) {
    const body = rawBody(root, head, file);
    assert.equal(
      body === null ? null : sha(body),
      expected,
      `complete committed current source: ${file}`,
    );
  }
  const { gitReaderHarness } =
    await import("../performance/support/warm-type-backed-git-reader-harness.ts");
  assert.deepEqual(
    [...gitReaderHarness(gitReaderSetupChanges, new Set(), root, gitReaderNoDeepSource, head)],
    gitReaderSetupChanges,
  );
  const { authenticateRuntimeApiSourceFamily } =
    await import("./support/runtime-props-api-source-custody.mjs");
  const { authenticateSetupExportMembership } =
    await import("./support/setup-export-preset-source-custody.mjs");
  const { authenticateDeepPropsSourceFamily } =
    await import("./support/props-deep-source-custody.mjs");
  const read = (file: string) => physicalBody(root, file);
  const runtime = authenticateRuntimeApiSourceFamily(root, read);
  authenticateSetupExportMembership(root, runtime, read);
  const family = authenticateDeepPropsSourceFamily(root, read);
  const current = family.currentSuccessor.currentSuccessor;
  assert.equal(current.candidate, "ecb8c9b25ee083725858229a2894052bb6de1fbf");
  const directory = "tests/_fixtures/differential/lint/props-deep-destructure-8090/source-custody/";
  assert.equal(
    sha(read(directory + "trait-scope-correction/successor.json")),
    "9591f993ff7ff20537ffbc532ce84af6ef201ec26e248b99d7788725a7dece59",
  );
  assert.equal(current.sources.length, 12);
  for (const source of current.sources) {
    for (const arm of [source.before, source.after]) {
      const body = read(arm.archive);
      assert.equal(body.length, arm.bytes);
      assert.equal(sha(body), arm.sha256);
      assert.equal(
        createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
        arm.gitBlob,
      );
    }
    assert.deepEqual(read(source.path), read(source.after.archive));
  }
  assert.deepEqual(read(family.producer.after.path), read(family.producer.after.archive));
  const helper = JSON.parse(read(directory + "helper-integration/manifest.json").toString());
  for (const entry of helper.helpers)
    for (const side of ["before", "after"] as const) {
      const body = read(directory + "helper-integration/" + entry[side].archive);
      assert.equal(sha(body), entry[side].sha256);
      assert.deepEqual(
        body,
        side === "before" ? rawBody(root, historicalIncomingSource, entry.path) : read(entry.path),
      );
    }
});

test("all204 complete pair sides and every required presence refuse drift", () => {
  let sides = 0;
  for (const [file] of gitReaderNoDeepPairs)
    for (const side of ["before", "after"] as const) {
      sides++;
      for (const changed of ["f".repeat(64), null]) {
        if (changed === digest(side, file)) continue;
        assert.equal(
          qualifyGitReaderNoDeepHarness(gitReaderNoDeepChanges, new Set(), (s, name) =>
            s === side && name === file ? changed : digest(s, name),
          ),
          null,
          `${side}:${file}:${changed}`,
        );
      }
    }
  assert.equal(sides, 204);
});

test("unknown, missing, duplicate and inherited owners cannot lend whole family authority", () => {
  assert.equal(qualifyGitReaderNoDeepHarness([], new Set(), digest), null);
  assert.equal(
    qualifyGitReaderNoDeepHarness([...gitReaderNoDeepChanges, "unknown.ts"], new Set(), digest),
    null,
  );
  for (const file of gitReaderNoDeepChanges) {
    const missing = gitReaderNoDeepChanges.filter((name) => name !== file);
    assert.equal(qualifyGitReaderNoDeepHarness(missing, new Set([file]), digest), null, file);
    assert.equal(
      qualifyGitReaderNoDeepHarness([...gitReaderNoDeepChanges, file], new Set(), digest),
      null,
      file,
    );
  }
  const inherited = new Set(["already-qualified.ts"]);
  assert.ok(
    qualifyGitReaderNoDeepHarness([...gitReaderNoDeepChanges, ...inherited], inherited, digest),
  );
  const family = "crates/vize_patina/src/linter/script_rules.rs";
  assert.equal(
    qualifyGitReaderNoDeepHarness([...gitReaderNoDeepChanges, family], new Set([family]), digest),
    null,
  );
  assert.equal(
    qualifyGitReaderNoDeepHarness(
      [...gitReaderNoDeepChanges, ...inherited],
      inherited,
      (side, file) =>
        file === family
          ? "88ea29b144566d5324df9299009331dd50da1c160ecf753d01d1b001dba0100e"
          : digest(side, file),
    ),
    null,
  );
});

test("the declared first-block inverse and finite wrapper hunk restore complete historical bodies", () => {
  prepareLiteralSource(root, historicalIncomingSource);
  const law = "tests/tooling/warm-type-backed-git-reader-incoming.test.ts";
  const oldLaw = rawBody(root, historicalIncomingSource, law)!.toString();
  const currentLaw = physicalBody(root, law).toString();
  const tail = 'test("every complete incoming before and after body remains mandatory", () => {';
  const oldTail = oldLaw.indexOf(tail);
  const currentTail = currentLaw.indexOf(tail);
  assert.ok(oldTail > 0 && currentTail > 0);
  assert.equal(currentLaw.slice(currentTail), oldLaw.slice(oldTail));
  assert.deepEqual(
    Buffer.from(oldLaw.slice(0, oldTail) + currentLaw.slice(currentTail)),
    Buffer.from(oldLaw),
  );
  const file = "tests/performance/support/warm-type-backed-git-reader-harness.ts";
  const original = rawBody(root, historicalIncomingSource, file)!.toString();
  const imported =
    'import {\n  gitReaderNoDeepGuard,\n  qualifyGitReaderNoDeepHarness,\n} from "./warm-type-backed-git-reader-nodeep.ts";\n';
  const guardSHA = sha(physicalBody(root, gitReaderNoDeepGuard));
  const branch = `  const nodeep =
    !qualified &&
    !incoming &&
    gitBodyDigest(root, before, gitReaderNoDeepGuard) === null &&
    gitBodyDigest(root, after, gitReaderNoDeepGuard) ===
      "${guardSHA}"
      ? qualifyGitReaderNoDeepHarness(changed, inherited, (side, file) =>
          gitBodyDigest(root, side === "before" ? before : after, file),
        )
      : null;
`;
  const anchor = '} from "./warm-type-backed-git-reader-incoming.ts";\n';
  const accepted = "  const accepted = qualified ?? incoming;\n";
  assert.equal(original.split(anchor).length, 2);
  assert.equal(original.split(accepted).length, 2);
  const expected = original
    .replace(anchor, anchor + imported)
    .replace(accepted, branch + "  const accepted = qualified ?? incoming ?? nodeep;\n");
  const setupImport =
    'import {\n  gitReaderSetupGuard,\n  qualifyGitReaderSetupHarness,\n} from "./warm-type-backed-git-reader-setup.ts";\n';
  const setupBranch = `  const setup =
    !qualified &&
    !incoming &&
    !nodeep &&
    gitBodyDigest(root, before, gitReaderSetupGuard) === null &&
    gitBodyDigest(root, after, gitReaderSetupGuard) ===
      "${sha(physicalBody(root, gitReaderSetupGuard))}"
      ? qualifyGitReaderSetupHarness(changed, inherited, (side, file) =>
          gitBodyDigest(root, side === "before" ? before : after, file),
        )
      : null;
`;
  const setupAccepted = "  const accepted = qualified ?? incoming ?? nodeep ?? setup;\n";
  let restored = physicalBody(root, file).toString();
  for (const insertion of [setupImport, setupBranch]) {
    assert.equal(restored.split(insertion).length, 2);
    restored = restored.replace(insertion, "");
  }
  assert.equal(restored.split(setupAccepted).length, 2);
  restored = restored.replace(
    setupAccepted,
    "  const accepted = qualified ?? incoming ?? nodeep;\n",
  );
  assert.deepEqual(Buffer.from(restored), Buffer.from(expected));
});

test("independent literal readers refuse real replacements, executable/symlink modes and borrowed HEAD", () => {
  const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-reader-nodeep-git-"));
  const git = (args: string[], input?: string | Buffer) => rawGit(fixture, args, input);
  const object = (body: string) => git(["hash-object", "-w", "--stdin"], body).toString().trim();
  const commit = (mode: string, blob: string) => {
    const tree = git(["mktree"], `${mode} blob ${blob}\tsource.rs\n`).toString().trim();
    return git(
      ["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit-tree", tree],
      "literal fixture\n",
    )
      .toString()
      .trim();
  };
  try {
    git(["init", "--quiet"]);
    const body = '// complete original\r\nconst KEY: &str = "日本語";\r\n';
    const originalBlob = object(body);
    const replacementBlob = object("foreign replacement\n");
    const original = commit("100644", originalBlob);
    git(["replace", original, commit("100644", replacementBlob)]);
    git(["replace", originalBlob, replacementBlob]);
    assert.deepEqual(rawBody(fixture, original, "source.rs"), Buffer.from(body));
    assert.equal(rawBody(fixture, original, "absent.rs"), null);
    assert.throws(() => rawBody(fixture, "HEAD", "source.rs"));
    for (const mode of ["100755", "120000"])
      assert.throws(
        () => rawBody(fixture, commit(mode, originalBlob), "source.rs"),
        /regular source/u,
      );
    fs.writeFileSync(path.join(fixture, "source.rs"), body, { mode: 0o755 });
    assert.throws(() => physicalBody(fixture, "source.rs"), /regular physical source/u);
    fs.symlinkSync("source.rs", path.join(fixture, "link.rs"));
    assert.throws(() => physicalBody(fixture, "link.rs"), /regular physical source/u);
  } finally {
    fs.rmSync(fixture, { recursive: true, force: true });
  }
});
