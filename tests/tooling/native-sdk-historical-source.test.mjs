import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  archivedNativeSdkModule,
  createNativeSdkHistoricalView,
  historicalCompositionBody,
  nativeSdkHistoricalView,
  nativeSdkSourceRoot,
} from "./support/native-sdk-historical-source.mjs";
import { sourcePaths } from "./support/native-sdk-historical-packets.mjs";
import { ownedCorsaSource } from "../../tools/support/compat/github/owned-corsa-source-custody.mjs";
import { authenticateOwnedCorsaAfterContext } from "../../tools/support/compat/github/owned-corsa-clippy-source.mjs";
import { ownedCorsaTestSourceIdentities } from "../../tools/support/compat/github/owned-corsa-test-source.mjs";

const sha = (body) => createHash("sha256").update(body).digest("hex");
const gitBlob = (body) =>
  createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex");
const physical = (file) => fs.readFileSync(path.join(nativeSdkSourceRoot, file));

void test("genuine current237 and two complete raw Git callers qualify separately from original121", () => {
  const current = createNativeSdkHistoricalView();
  assert.equal(current.receipt.completeProductionBodies, 237);
  assert.equal(current.receipt.completeCallerBodies, 2);
  assert.equal(current.receipt.originalProductionBodies, 121);
  assert.equal(current.receipt.historicalSourceBodies, 206);
  assert.equal(current.receipt.runtimeCredit, false);
  assert.equal(current.receipt.performanceCredit, false);
  assert.equal(current.receipt.currentAfterContext.wholeCurrentBodies, 237);
  assert.equal(current.receipt.currentAfterContext.wholeManifestBodies, 6);
  assert.equal(current.receipt.currentAfterContext.clippyExecutionCredit, false);
  assert.equal(current.receipt.currentAfterContext.wholeCorrectedRustBodies, 2);
  assert.deepEqual(current.receipt.currentAfterContext.correctedSourceFiles, [
    "vendor/corsa_client/src/api/handles.rs",
    "vendor/corsa/src/bin/bench_real_corsa/args.rs",
  ]);
  const afterIdentities = new Map(
    current.currentAfterIdentities.map(({ file, identity }) => [file, identity]),
  );
  assert.equal(afterIdentities.size, 237);
  const six = new Set(
    ["corsa_core", "corsa_jsonrpc", "corsa_client", "corsa_lsp", "corsa_orchestrator", "corsa"].map(
      (name) => `vendor/${name}/Cargo.toml`,
    ),
  );
  for (const row of ownedCorsaSource.rows) {
    if (
      ownedCorsaTestSourceIdentities.has(row.file) ||
      six.has(row.file) ||
      row.file === "vendor/corsa_client/src/api/handles.rs" ||
      row.file === "vendor/corsa/src/bin/bench_real_corsa/args.rs"
    )
      assert.notDeepEqual(afterIdentities.get(row.file), row.after);
    else assert.deepEqual(afterIdentities.get(row.file), row.after);
    for (const side of ["before", "after"]) {
      const expected = side === "before" ? row.before : afterIdentities.get(row.file);
      const value = current.actualReader(side, row.file);
      if (expected === null) assert.equal(value, null);
      else {
        assert(value && value.mode === expected.mode);
        assert.equal(value.blob, expected.blob);
        assert.equal(value.body.length, expected.bytes);
        assert.equal(sha(value.body), expected.sha256);
        assert.equal(gitBlob(value.body), expected.blob);
      }
    }
  }
  for (const file of [
    "tests/performance/support/warm-type-backed-native-terminal-final.ts",
    "tests/performance/support/warm-type-backed-workflow.ts",
  ]) {
    assert.deepEqual(current.actualReader("after", file).body, physical(file));
    assert.notDeepEqual(current.historicalReader("after", file).body, physical(file));
  }
});

void test("the original two-package lock law and the real current six-package graph remain separate", () => {
  const current = nativeSdkHistoricalView();
  const old = current.body("Cargo.lock").toString();
  assert.deepEqual(
    [...old.matchAll(/^name = "(corsa_(?:core|lsp))"$/gmu)].map((m) => m[1]),
    ["corsa_core", "corsa_lsp"],
  );
  const after = physical("Cargo.lock").toString();
  assert.notEqual(after, old);
  const packages = after
    .split("[[package]]")
    .filter((section) => /^\nname = "vize_corsa/mu.test(section));
  const expected = [
    ["vize_corsa", "corsa"],
    ["vize_corsa_client", "corsa_client"],
    ["vize_corsa_core", "corsa_core"],
    ["vize_corsa_jsonrpc", "corsa_jsonrpc"],
    ["vize_corsa_lsp", "corsa_lsp"],
    ["vize_corsa_orchestrator", "corsa_orchestrator"],
  ];
  assert.deepEqual(
    packages.map((section) => /^\nname = "([^"]+)"/u.exec(section)[1]),
    expected.map(([name]) => name),
  );
  for (let i = 0; i < expected.length; i++) {
    const [name, directory] = expected[i];
    assert.match(packages[i], /^version = "0\.443\.0"$/mu);
    assert.doesNotMatch(packages[i], /^(?:source|checksum) = /mu);
    assert.match(
      physical(`vendor/${directory}/Cargo.toml`).toString(),
      new RegExp(`^name = "${name}"$`, "mu"),
    );
    assert.match(
      physical("Cargo.toml").toString(),
      new RegExp(
        `^${directory} = \\{ package = "${name}", version = "=0\\.443\\.0", path = "vendor/${directory}" \\}$`,
        "mu",
      ),
    );
  }
});

void test("foreign, missing, nonregular and mixed complete actual source or caller bodies refuse before a loan", () => {
  const current = nativeSdkHistoricalView();
  const added = ownedCorsaSource.rows.find((row) => row.archiveMember).file;
  const final = "tests/performance/support/warm-type-backed-native-terminal-final.ts";
  for (const [side, file, change] of [
    ["before", "Cargo.toml", () => null],
    ["after", "Cargo.lock", () => null],
    ["after", added, (body) => ({ ...body, mode: "100755" })],
    [
      "after",
      "vendor/corsa_core/Cargo.toml",
      () => current.historicalReader("after", "vendor/corsa_core/Cargo.toml"),
    ],
    [
      "after",
      added,
      (body) => {
        const changed = Buffer.from(body.body);
        changed[0] ^= 1;
        return { ...body, blob: gitBlob(changed), body: changed };
      },
    ],
    ["after", final, () => current.historicalReader("after", final)],
  ])
    assert.throws(
      () =>
        createNativeSdkHistoricalView(nativeSdkSourceRoot, (s, name) => {
          const real = current.actualReader(s, name);
          return s === side && name === file ? change(real) : real;
        }),
      undefined,
      side + ":" + file,
    );
});

void test("a supplied callback is authenticated again before a requested historical whole body is returned", () => {
  const current = nativeSdkHistoricalView();
  let foreign = false;
  const loan = createNativeSdkHistoricalView(nativeSdkSourceRoot, (side, file) => {
    const value = current.actualReader(side, file);
    if (foreign && side === "after" && file === "Cargo.lock")
      return { ...value, body: Buffer.concat([value.body, Buffer.from("\n")]) };
    return value;
  });
  assert.notDeepEqual(loan.body("Cargo.lock"), physical("Cargo.lock"));
  foreign = true;
  assert.throws(() => loan.body("Cargo.lock"));
});

void test("unknown paths and modules refuse and returned historical buffers cannot mutate retained authority", () => {
  const current = nativeSdkHistoricalView();
  for (const file of ["vendor/corsa_core/src/unknown.rs", "../Cargo.lock", "/Cargo.lock"])
    assert.throws(() => current.body(file));
  assert.throws(() => archivedNativeSdkModule("tools/support/compat/github/unknown.mjs"));
  assert.throws(() => historicalCompositionBody("foreign", "Cargo.lock"));
  const body = current.body("Cargo.lock"),
    original = Buffer.from(body);
  body[0] ^= 1;
  assert.deepEqual(current.body("Cargo.lock"), original);
});

void test("all four complete historical parser modules and the original206 fixture closure remain nonexecuted", () => {
  const manifest = JSON.parse(
    physical("tests/performance/support/warm-type-backed-lexical-dollar-manifest.json"),
  );
  const files = sourcePaths(manifest);
  assert.equal(files.length, 206);
  assert.equal(new Set(files).size, 206);
  assert.equal(files.includes("vendor/corsa/Cargo.toml"), false);
  for (const name of [
    "native-lexical-dollar.sh",
    "native-lexical-dollar-metadata.mjs",
    "native-lexical-dollar-receipt.mjs",
    "native-lexical-composition-custody.mjs",
  ]) {
    const body = archivedNativeSdkModule("tools/support/compat/github/" + name);
    assert.equal(body.source, "8304eb7a08a96e58ca0a318cc2fa6ffc6e9acd75");
    assert.equal(body.mode, "100644");
    assert.equal(sha(body.body), body.sha256);
    assert.equal(gitBlob(body.body), body.blob);
    assert.equal(body.body.length, body.bytes);
  }
  assert.equal(nativeSdkHistoricalView().receipt.runtimeCredit, false);
});

void test("mixing exactly one retained old whole manifest into the real current six-manifest class refuses", () => {
  const current = nativeSdkHistoricalView();
  const bytes = physical("tools/support/compat/github/owned-corsa-clippy-source.json");
  assert.equal(sha(bytes), "9b9212acd2ae6a56c4c4a8fb86abec5e550a2a55347ae030a896785e77f8c96f");
  const entries = JSON.parse(bytes).entries;
  assert.equal(entries.length, 6);
  for (const entry of entries) {
    const old = entry.retainedOldAfter;
    const body = Buffer.from(old.rawBase64, "base64");
    assert.equal(body.toString("base64"), old.rawBase64);
    assert.equal(body.length, old.bytes);
    assert.equal(sha(body), old.sha256);
    assert.equal(gitBlob(body), old.blob);
    assert.equal(
      authenticateOwnedCorsaAfterContext(ownedCorsaSource.rows, (file) =>
        file === entry.file
          ? { mode: old.mode, blob: old.blob, body }
          : current.actualReader("after", file),
      ),
      null,
      entry.file,
    );
  }
});

void test("the whole incoming main lock is genuine current source and the original206 receives only its exact old historical lock", () => {
  const current = nativeSdkHistoricalView();
  const real = current.actualReader("after", "pnpm-lock.yaml");
  assert.equal(real.blob, "ab601a6961c855d1f52948750cc72f117fbf2c1d");
  assert.equal(sha(real.body), "f83208b48ac69a7fd603384bb4962d0da0116b04ff2b19793daa2a63dc5378db");
  assert.deepEqual(real.body, physical("pnpm-lock.yaml"));
  const historical = current.body("pnpm-lock.yaml");
  assert.equal(sha(historical), "12e66667fc82ac9a5b72d1e9ab2aa59f7c1466ea8be689f17a67678bf31a0915");
  assert.notDeepEqual(historical, real.body);
  assert.equal(current.receipt.completeCurrentFrameBodies, 126);
  for (const credit of [
    "ancestryCredit",
    "squashCredit",
    "dependencyCredit",
    "toolchainCredit",
    "runtimeCredit",
    "performanceCredit",
  ])
    assert.equal(current.receipt[credit], false);
  let foreign = false;
  const loan = createNativeSdkHistoricalView(nativeSdkSourceRoot, (side, file) => {
    const value = current.actualReader(side, file);
    if (foreign && side === "after" && file === "pnpm-lock.yaml")
      return { ...value, body: Buffer.concat([value.body, Buffer.from("\n")]) };
    return value;
  });
  assert.deepEqual(loan.body("pnpm-lock.yaml"), historical);
  foreign = true;
  assert.throws(() => loan.body("pnpm-lock.yaml"));
});
