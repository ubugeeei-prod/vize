import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { ownedCorsaSource, qualifyOwnedCorsaSource } from "./owned-corsa-source-custody.mjs";
import { ownedCorsaReader } from "./owned-corsa-source-reader.mjs";
import { nativeLexicalCompositionBodyPairs } from "../../../../tests/performance/support/warm-type-backed-native-lexical-composition-host.ts";

const packages = [
  "corsa",
  "corsa_core",
  "corsa_jsonrpc",
  "corsa_client",
  "corsa_lsp",
  "corsa_orchestrator",
];
const sha = (body) => createHash("sha256").update(body).digest("hex");
const git = (root, args) =>
  execFileSync("git", ["--no-replace-objects", ...args], {
    cwd: root,
    maxBuffer: 64 * 1024 * 1024,
    env: { ...process.env, GIT_NO_LAZY_FETCH: "1" },
  });

/** A complete source snapshot; only the two old vendor manifests are projected for the old checker. */
export function authenticateProcessReceiptVendorSource(root) {
  const head = git(root, ["rev-parse", "HEAD"]).toString().trim();
  const { baseline, rows } = ownedCorsaSource;
  const files = rows.map(({ file }) => file);
  const reader = ownedCorsaReader(root, baseline, head);
  const graph = qualifyOwnedCorsaSource({
    baseline,
    production: files,
    alreadyQualified: new Set(),
    reader,
    historicalPairs: nativeLexicalCompositionBodyPairs,
  });
  assert(graph, "complete genuine237 source bodies required before vendor manifest projection");
  const prefixes = packages.map((name) => "vendor/" + name + "/");
  const expected = files
    .filter((file) => prefixes.some((prefix) => file.startsWith(prefix)))
    .sort((a, b) => a.localeCompare(b));
  const raw = git(root, ["ls-tree", "-r", "--name-only", head, "--", ...prefixes]);
  assert.deepEqual(
    raw
      .toString()
      .trim()
      .split("\n")
      .sort((a, b) => a.localeCompare(b)),
    expected,
    "complete six-package raw source census",
  );
  const physical = [];
  const visit = (directory) => {
    assert(fs.lstatSync(path.join(root, directory)).isDirectory(), "physical vendor directory");
    for (const name of fs.readdirSync(path.join(root, directory))) {
      const file = directory + "/" + name;
      const stat = fs.lstatSync(path.join(root, file));
      if (stat.isDirectory()) visit(file);
      else {
        assert(stat.isFile() && (stat.mode & 0o777) === 0o644, "regular vendor source: " + file);
        physical.push(file);
      }
    }
  };
  for (const name of packages) visit("vendor/" + name);
  assert.deepEqual(
    physical.sort((a, b) => a.localeCompare(b)),
    expected,
    "complete six-package physical source census",
  );
  const projected = ["vendor/corsa_core/Cargo.toml", "vendor/corsa_lsp/Cargo.toml"];
  return {
    sourceContext: graph.sourceContext ?? null,
    actualReader: reader,
    checksum: (target) => {
      const file = path.relative(root, target).split(path.sep).join("/");
      assert(files.includes(file), "checksum outside authenticated source scope: " + file);
      const value = projected.includes(file)
        ? graph.historicalReader("after", file)
        : reader("after", file);
      assert(value && value.mode === "100644");
      return sha(value.body);
    },
    receipt: {
      authority: "complete-owned-corsa-vendor-source-counterfactual",
      head,
      baseline,
      manifestSHA256: graph.manifestSHA256,
      completeProductionBodies: 237,
      originalProductionBodies: 121,
      archiveMemberBodies: 112,
      additiveLicenseBodies: 4,
      completeVendorBodies: expected.length,
      historicalManifestPaths: projected,
      runtimeCredit: false,
      performanceCredit: false,
      scope:
        "declared237 source bodies and six vendor packages; unrelated incoming sources receive no qualification",
    },
  };
}
