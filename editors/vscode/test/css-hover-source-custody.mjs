import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { isDeepStrictEqual } from "node:util";

const directory = path.dirname(fileURLToPath(import.meta.url));
export const historicalBaselineRevision = "2934b35b18393c188b3da5f8e50595299589fb8c";
const captureRevision = "0cc8615736db2f2ef6b82177aa2e92a7de9f86dc";
const reviewedBuildPaths = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo"];
const historicalDirectory =
  "tests/_fixtures/differential/lsp/css-hover-stylesheet-examples-10572/historical-2934";
const reviewedLayers = [
  {
    name: "declaration-admission-10562",
    baseRevision: historicalBaselineRevision,
    reviewedRevision: "76fbd6e7f405b4cdc10193be356f1609cbf3c5f4",
  },
  {
    name: "stylesheet-examples-10572",
    baseRevision: historicalBaselineRevision,
    reviewedRevision: "9fa7d40b7aaebbd7b0b1058939e7c7cc3032023c",
  },
];
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const sorted = (records) =>
  [...records].sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
const fingerprint = (records) => hash(JSON.stringify(sorted(records)));

export function composeReviewedCssTrees(baseline, layers) {
  const current = new Map(baseline.map((entry) => [entry.path, entry]));
  assert.equal(current.size, baseline.length, "duplicate baseline path");
  for (const layer of layers) {
    const paths = new Set();
    for (const change of layer.changes) {
      assert.ok(!paths.has(change.path), "duplicate reviewed change");
      paths.add(change.path);
      assert.ok(change.path.startsWith("crates/"));
      assert.deepEqual(
        current.get(change.path) ?? null,
        change.before,
        `reviewed preimage: ${change.path}`,
      );
      assert.notDeepEqual(change.before, change.after, "reviewed change cannot be a no-op");
      if (change.after === null) current.delete(change.path);
      else {
        assert.equal(change.after.path, change.path);
        current.set(change.path, change.after);
      }
    }
  }
  return sorted(current.values());
}

export function assertExactCssTree(actual, expected) {
  const current = new Map(actual.map((entry) => [entry.path, entry]));
  const reviewed = new Map(expected.map((entry) => [entry.path, entry]));
  assert.equal(current.size, actual.length, "duplicate actual tree path");
  assert.equal(reviewed.size, expected.length, "duplicate reviewed tree path");
  const differences = [...new Set([...current.keys(), ...reviewed.keys()])]
    .sort()
    .filter((name) => !isDeepStrictEqual(current.get(name), reviewed.get(name)))
    .map((name) => ({
      path: name,
      actual: current.get(name) ?? null,
      expected: reviewed.get(name) ?? null,
    }));
  assert.deepEqual(
    differences,
    [],
    "current crates tree differs from reviewed blob/mode composition",
  );
}

export function assertCleanCssSourceWorkingTree(repoRoot) {
  const status = execFileSync(
    "git",
    ["status", "--porcelain=v1", "--untracked-files=all", "--", "crates", ...reviewedBuildPaths],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 32 * 1024 * 1024,
    },
  );
  assert.equal(
    status,
    "",
    `tracked, staged or untracked source bytes differ from HEAD:\n${status}`,
  );
}

export function verifyCssHoverSourceCustody(repoRoot, expectedRevision) {
  const git = (...args) =>
    execFileSync("git", args, { cwd: repoRoot, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  const readTree = (revision, paths) =>
    git("ls-tree", "-r", "-z", revision, "--", ...paths)
      .split("\0")
      .filter(Boolean)
      .map((line) => {
        const tab = line.indexOf("\t");
        assert.ok(tab > 0);
        const [mode, type, oid] = line.slice(0, tab).split(" ");
        return { path: line.slice(tab + 1), mode, type, oid };
      });
  const contractBytes = fs.readFileSync(path.join(directory, "css-hover-source-custody.json"));
  const contract = JSON.parse(contractBytes);
  assert.equal(contract.schema, "vize.css-hover.source-custody");
  assert.equal(contract.version, 1);
  assert.equal(contract.historicalBaselineRevision, historicalBaselineRevision);
  assert.equal(contract.historicalCapture.sourceRevision, captureRevision);
  assert.deepEqual(contract.buildPaths, reviewedBuildPaths);
  assert.deepEqual(
    contract.layers.map(({ name, baseRevision, reviewedRevision }) => ({
      name,
      baseRevision,
      reviewedRevision,
    })),
    reviewedLayers,
  );
  const revision = git("rev-parse", "HEAD").trim();
  assert.equal(revision, expectedRevision, "source-custody head must match the requested source");
  assertCleanCssSourceWorkingTree(repoRoot);
  git("merge-base", "--is-ancestor", reviewedLayers[0].reviewedRevision, "HEAD");
  const baseline = readTree(historicalBaselineRevision, ["crates"]);
  assert.equal(
    git("rev-parse", `${historicalBaselineRevision}:crates`).trim(),
    contract.baselineCratesTreeOid,
  );
  assert.equal(fingerprint(baseline), contract.baselineCratesSha256);
  assert.equal(contract.historicalCapture.cratesSha256, contract.baselineCratesSha256);
  const p1 = composeReviewedCssTrees(baseline, [contract.layers[0]]);
  assertExactCssTree(readTree(reviewedLayers[0].reviewedRevision, ["crates"]), p1);
  // The old Docs revision may be unreachable after rebase. Its reviewed
  // before/after blobs and modes are pinned in the committed contract; do not
  // infer a new base, fetch a floating ref, or skip a missing old revision.
  const expected = composeReviewedCssTrees(baseline, contract.layers);
  const actual = readTree("HEAD", ["crates"]);
  assertExactCssTree(actual, expected);
  assert.deepEqual(
    sorted(readTree("HEAD", contract.buildPaths)),
    contract.buildInputs,
    "workspace build inputs changed",
  );

  for (const witness of contract.historicalCapture.witnesses) {
    assert.ok(
      witness.path.startsWith(`${historicalDirectory}/`) && !witness.path.split("/").includes(".."),
    );
    assert.equal(
      hash(fs.readFileSync(path.join(repoRoot, witness.path))),
      witness.sha256,
      `historical witness bytes: ${witness.path}`,
    );
  }
  const historicalFile = (name) =>
    JSON.parse(fs.readFileSync(path.join(repoRoot, historicalDirectory, name), "utf8"));
  const identity = historicalFile("identity.json");
  const receipt = historicalFile("build-receipt.json");
  assert.equal(identity.baselineRevision, historicalBaselineRevision);
  assert.equal(identity.source.sourceRevision, captureRevision);
  assert.equal(receipt.sourceRevision, captureRevision);
  assert.equal(receipt.binarySha256, identity.source.binarySha256);
  for (const name of ["hex-baseline-hover.json", "rgba-baseline-hover.json"]) {
    const packet = historicalFile(name);
    assert.equal(packet.raw, null);
    assert.deepEqual(packet.hovers, []);
  }
  return {
    historicalBaselineRevision,
    historicalCapture: contract.historicalCapture,
    sourceRevision: revision,
    contractSha256: hash(contractBytes),
    reviewedLayers: contract.layers,
    cratesSha256: fingerprint(actual),
    productionChanges: [
      ...new Set(contract.layers.flatMap((layer) => layer.changes.map((change) => change.path))),
    ].sort(),
    buildInputs: contract.buildInputs,
  };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const root = path.resolve(directory, "../../..");
  const revision =
    process.env.SOURCE_SHA ??
    execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  const report = verifyCssHoverSourceCustody(root, revision);
  console.log(
    JSON.stringify(
      {
        sourceRevision: report.sourceRevision,
        historicalBaselineRevision,
        productionChanges: report.productionChanges,
      },
      null,
      2,
    ),
  );
}
