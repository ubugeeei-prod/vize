import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  authenticateStarListingSource,
  qualifyStarListingBD,
  qualifyStarListingSDK,
  qualifyStarListingMain,
  starListingLexicalReader,
} from "../../tools/support/compat/github/empty-vue-star-listing-source.mjs";

const root = fs.realpathSync(new URL("../../", import.meta.url));
const manifestPath = "tools/support/compat/github/empty-vue-star-listing-source.json";
const manifest = JSON.parse(fs.readFileSync(path.join(root, manifestPath)));
const rows = manifest.parts.flatMap(([file]) => JSON.parse(fs.readFileSync(path.join(root, file))));
const executing = [manifestPath, ...manifest.parts.map(([file]) => file), ...manifest.executing];
const files = [...rows.map(([file]) => file), ...executing];
const execution = "crates/vize_canon/tests/empty_vue_star_exports/execution.rs";
const sha = (body) => createHash("sha256").update(body).digest("hex");
const pairs = new Map(rows.map(([file, , , before, , , after]) => [file, { before, after }]));
for (const file of executing)
  pairs.set(file, { before: null, after: sha(fs.readFileSync(path.join(root, file))) });
const digest = (side, file) => pairs.get(file)?.[side] ?? null;
const sdkBefore = new Map(manifest.sdkShippingBefore);
const sdkDigest = (side, file) =>
  side === "before" && sdkBefore.has(file) ? sdkBefore.get(file) : digest(side, file);
const git = (owner, args, input) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: owner,
    input,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1", GIT_ALLOW_PROTOCOL: "" },
  });
function fixture(run) {
  const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-star-listing-refusals-"));
  try {
    git(owner, ["init", "--quiet"]);
    const common = git(root, ["rev-parse", "--git-common-dir"]).toString().trim();
    fs.writeFileSync(
      path.join(owner, ".git/objects/info/alternates"),
      fs.realpathSync(path.resolve(root, common, "objects")) + "\n",
    );
    const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
    git(owner, ["read-tree", head]);
    git(owner, ["update-ref", "refs/heads/current", head]);
    git(owner, ["symbolic-ref", "HEAD", "refs/heads/current"]);
    for (const file of new Set([...files, ...manifest.mainUnion.cells.map(([file]) => file)])) {
      const target = path.join(owner, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(root, file), target);
      fs.chmodSync(target, 0o644);
    }
    run(owner, head);
  } finally {
    fs.rmSync(owner, { recursive: true, force: true });
  }
}

await test("the recorded BD union admits only one production path after complete current raw source custody", () => {
  const packet = authenticateStarListingSource(root);
  assert.equal(packet.originalStarBodies, 322);
  assert.equal(packet.originalJSBodies, 93);
  assert.equal(packet.runtimeCredit, false);
  assert.equal(packet.original400Credit, false);
  // The union was qualified against its own change when it merged. Every later
  // PR and queue candidate carries a different diff, so this qualifies the
  // recorded footprint instead of whatever change happens to be under test.
  assert.deepEqual(qualifyStarListingBD([execution], new Set(), digest)?.files, [execution]);
});

await test("every complete BD counterpart and executing body on both sides refuses drift and absence", () => {
  for (const file of files)
    for (const side of ["before", "after"])
      for (const value of [null, "f".repeat(64)]) {
        if (value === digest(side, file)) continue;
        assert.equal(
          qualifyStarListingBD([execution], new Set(), (s, name) =>
            s === side && name === file ? value : digest(s, name),
          ),
          null,
          side + ":" + file,
        );
      }
  for (const footprint of [
    [],
    [execution, execution],
    [execution, "unknown.rs"],
    [execution, "crates/vize_canon/src/lib.rs"],
  ])
    for (const qualified of [
      new Set(),
      new Set(footprint),
      new Set(["crates/vize_canon/src/lib.rs"]),
    ])
      assert.equal(qualifyStarListingBD(footprint, qualified, digest), null);
  assert.equal(qualifyStarListingBD([execution], new Set([execution]), digest), null);
});

await test("the actual SDK shipping before-image admits only its complete listing singleton", () => {
  assert.equal(qualifyStarListingSDK([execution], new Set(), sdkDigest)?.files[0], execution);
  assert.equal(qualifyStarListingSDK([execution], new Set(), digest), null);
  assert.equal(qualifyStarListingBD([execution], new Set(), sdkDigest), null);
  for (const production of [
    [],
    [execution, execution],
    [execution, "crates/vize_canon/src/lib.rs"],
  ])
    assert.equal(qualifyStarListingSDK(production, new Set(), sdkDigest), null);
  assert.equal(qualifyStarListingSDK([execution], new Set([execution]), sdkDigest), null);
  for (const file of files)
    for (const side of ["before", "after"])
      assert.equal(
        qualifyStarListingSDK([execution], new Set(), (selected, path) =>
          selected === side && path === file ? "f".repeat(64) : sdkDigest(selected, path),
        ),
        null,
        "every complete SDK singleton source cell remains mandatory",
      );
  for (const [file, checksum] of manifest.sdkShippingBefore) {
    const raw = git(root, ["show", manifest.sdkUnion.actualIncoming + ":" + file]);
    assert.equal(sha(raw), checksum, "actual merged SDK before-image");
  }
});

await test("actual incoming main singleton retains every complete source cell and rejects its neighbors", () => {
  const overrides = new Map(manifest.mainShippingBefore);
  const mainCells = new Map(
    manifest.mainUnion.cells.map(([file, , , before, , , after]) => [file, { before, after }]),
  );
  if (manifest.mainUnion.canonicalSuccessor) {
    const [file, , , after] = manifest.mainUnion.canonicalSuccessor.after;
    mainCells.get(file).after = after;
  }
  const mainDigest = (side, file) =>
    side === "before" && overrides.has(file)
      ? overrides.get(file)
      : (mainCells.get(file)?.[side] ?? digest(side, file));
  assert.deepEqual(qualifyStarListingMain([execution], new Set(), mainDigest)?.files, [execution]);
  for (const cells of [files, [...mainCells.keys()]])
    for (const file of cells)
      for (const side of ["before", "after"])
        assert.equal(
          qualifyStarListingMain([execution], new Set(), (s, f) =>
            s === side && f === file ? "f".repeat(64) : mainDigest(s, f),
          ),
          null,
        );
  for (const production of [
    [],
    [execution, execution],
    [execution, "crates/vize_croquis/src/lib.rs"],
  ])
    assert.equal(qualifyStarListingMain(production, new Set(), mainDigest), null);
  assert.equal(qualifyStarListingMain([execution], new Set([execution]), mainDigest), null);
  assert.equal(qualifyStarListingMain([execution], new Set(), sdkDigest), null);
  for (const [file, checksum] of manifest.mainShippingBefore)
    assert.equal(sha(git(root, ["show", manifest.mainUnion.source + ":" + file])), checksum);
});

await test("foreign raw bodies, changed entrypoints, hybrid source, modes and replacement refs refuse before replay", () => {
  fixture((owner, head) => {
    assert(authenticateStarListingSource(owner));
    const readLexical = starListingLexicalReader(owner);
    for (const file of [
      "tests/performance/support/warm-type-backed-host-qualifications.ts",
      "tests/tooling/warm-type-backed-lexical-dollar-host.test.ts",
    ]) {
      assert.deepEqual(readLexical(file), git(root, ["show", manifest.base + ":" + file]));
      const target = path.join(owner, file),
        original = fs.readFileSync(target);
      fs.writeFileSync(target, Buffer.concat([original, Buffer.from("foreign copied source\n")]));
      assert.throws(() => readLexical(file));
      fs.writeFileSync(target, original);
    }
    assert.throws(() => readLexical("unknown.ts"));
    assert.throws(() => readLexical(execution));
    readLexical.confirmSource();
    for (const file of [
      execution,
      "tests/performance/support/warm-type-backed-host-qualifications.ts",
      manifest.tests[0][0],
      "tests/tooling/javascript-sfc-star-source.test.mjs",
      "tests/tooling/javascript-sfc-options-source.test.mjs",
      "tests/tooling/warm-type-backed-lexical-dollar-host.test.ts",
      manifest.executing[0],
      manifestPath,
    ]) {
      const target = path.join(owner, file),
        original = fs.readFileSync(target);
      fs.writeFileSync(target, Buffer.concat([original, Buffer.from("foreign source\n")]));
      assert.throws(() => authenticateStarListingSource(owner));
      fs.writeFileSync(target, original);
      fs.chmodSync(target, 0o755);
      assert.throws(() => authenticateStarListingSource(owner));
      fs.chmodSync(target, 0o644);
      const old = git(root, ["show", manifest.base + ":" + execution]);
      fs.writeFileSync(target, old);
      assert.throws(() => authenticateStarListingSource(owner));
      fs.writeFileSync(target, original);
    }
    // A committed change to a named file is ordinary later work, not drift: the
    // ledger authenticates the tree it was recorded against.
    const target = path.join(owner, execution),
      original = fs.readFileSync(target);
    const changed = Buffer.concat([original, Buffer.from("later committed source\n")]);
    const blob = git(owner, ["hash-object", "-w", "--stdin"], changed).toString().trim();
    git(owner, ["update-index", "--cacheinfo", "100644," + blob + "," + execution]);
    const later = git(
      owner,
      [
        "-c",
        "user.name=Later work",
        "-c",
        "user.email=later@invalid.example",
        "commit-tree",
        git(owner, ["write-tree"]).toString().trim(),
        "-p",
        head,
      ],
      "later committed source\n",
    )
      .toString()
      .trim();
    git(owner, ["update-ref", "refs/heads/current", later]);
    fs.writeFileSync(target, changed);
    assert(authenticateStarListingSource(owner));
    git(owner, ["update-ref", "refs/replace/" + later, head]);
    assert.throws(() => authenticateStarListingSource(owner));
    git(owner, ["update-ref", "-d", "refs/replace/" + later]);
    assert(authenticateStarListingSource(owner));
  });
});
