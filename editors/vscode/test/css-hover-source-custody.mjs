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
const mergedMainRevision = "116a05062ad5ce4b9fa56bba0c801835498418fa";
const successorMainRevision = "9c8388e3dde4a03e5be74ee8a0ced20560983aea";
const vsixInfrastructureRevision = "9e5fa02567cb8757e5740269694e062761acfda6";
const reviewedVsixPaths = [
  ".github/workflows/vscode-vsix-validation.yml",
  "tests/tooling/differential-lsp-source-sessions.test.ts",
  "tests/tooling/source-session-tooling-project.test.ts",
  "tests/tooling/vscode-vsix-configuration.test.ts",
  "tools/commands/editors/vscode/assert-vsix-package.rs",
  "tools/support/editors/vsix_configuration.rs",
  "tools/support/lib.rs",
];
const historicalBuildPaths = ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo"];
const reviewedBuildPaths = [
  ...historicalBuildPaths,
  "davinci",
  "vendor",
  "assets",
  "tools/benchmarks/crates",
  "tests/davinci_test_support",
  "tests/vize_test_runner",
  "npm/cli/schemas/vize.config.schema.json",
  "docs/davinci/plan/budgets.toml",
];
const historicalDirectory =
  "tests/_fixtures/differential/lsp/css-hover-stylesheet-examples-10572/historical-2934";
const contractPath = "editors/vscode/test/css-hover-source-custody.json";
// These are complete module/fixture memberships, not a list of changed files.
// A new or renamed input inside an owned module must require a new review.
const ownedSourcePaths = [
  "crates/vize_maestro/src/ide/style.rs",
  "crates/vize_maestro/src/ide/style",
  "crates/vize_maestro/src/server/annotations.rs",
  "crates/vize_maestro/src/server/annotations/document_color.rs",
  "crates/vize_maestro/src/server/annotations/document_color",
  "crates/vize_maestro/src/server/handlers/typed.rs",
  "crates/vize_maestro/src/server/handlers/tests/completion_resolve.rs",
];
const ownedFixturePaths = [
  "tests/_fixtures/differential/lsp/css-color-declaration-admission-10562",
  "tests/_fixtures/differential/lsp/css-color-declaration-admission-10562-css",
  "tests/_fixtures/differential/lsp/css-color-declaration-admission-10562-stylus",
  "tests/_fixtures/differential/lsp/css-hover-stylesheet-examples-10572",
  "tests/_fixtures/differential/lsp/css-concrete-color-hover-10556",
];
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
const compareStrings = (a, b) => (a < b ? -1 : a > b ? 1 : 0);
const sorted = (records) => [...records].sort((a, b) => compareStrings(a.path, b.path));
const fingerprint = (records) => hash(JSON.stringify(sorted(records)));
const belongsTo = (name, roots) =>
  roots.some((root) => name === root || name.startsWith(`${root}/`));

function readGitTree(repoRoot, revision, paths, recursive = true) {
  return execFileSync(
    "git",
    ["ls-tree", ...(recursive ? ["-r"] : []), "-z", revision, "--", ...paths],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 },
  )
    .split("\0")
    .filter(Boolean)
    .map((line) => {
      const tab = line.indexOf("\t");
      assert.ok(tab > 0);
      const [mode, type, oid] = line.slice(0, tab).split(" ");
      return { path: line.slice(tab + 1), mode, type, oid };
    });
}

// Squash commits retain contents but deliberately omit the source branch's
// commit ancestry. Authenticate the entire historical product composition and
// the current committed owned inputs. This grants no current-host acceptance:
// unrelated main inputs are qualified only by the separate full-depth runner.
export function assertCssSnapshotProvenance(
  repoRoot,
  contractBytes,
  expectedCrates,
  snapshot,
  fixtures,
) {
  const loadedContract = JSON.parse(contractBytes);
  assert.deepEqual(snapshot, loadedContract.productSnapshot, "foreign product snapshot identity");
  assert.deepEqual(fixtures, loadedContract.reviewedFixtures, "foreign fixture snapshot identity");
  assert.match(snapshot.sourceRevision, /^[a-f0-9]{40}$/);
  assert.equal(snapshot.cratesSha256, fingerprint(expectedCrates), "foreign product snapshot");
  assert.equal(snapshot.cratesEntries, expectedCrates.length, "product snapshot membership");
  assert.equal(snapshot.fixturesSha256, fingerprint(fixtures), "foreign fixture snapshot");
  const git = (...args) => execFileSync("git", args, { cwd: repoRoot, encoding: "utf8" });
  assert.equal(
    hash(git("show", `HEAD:${contractPath}`)),
    hash(contractBytes),
    "current committed source-custody contract differs from the loaded contract",
  );
  assertExactCssTree(
    readGitTree(repoRoot, "HEAD", ownedSourcePaths),
    expectedCrates.filter(({ path: name }) => belongsTo(name, ownedSourcePaths)),
  );
  assert.ok(fixtures.every(({ path: name }) => belongsTo(name, ownedFixturePaths)));
  assertExactCssTree(readGitTree(repoRoot, "HEAD", ownedFixturePaths), fixtures);
  assert.equal(
    git(
      "status",
      "--porcelain=v1",
      "--untracked-files=all",
      "--",
      ...ownedSourcePaths,
      ...ownedFixturePaths,
      contractPath,
    ),
    "",
    "dirty, staged or untracked owned CSS inputs",
  );
}

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
    .sort(compareStrings)
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

export function composeCurrentVsixInputs(inputs, changes) {
  assert.deepEqual(
    changes.map(({ path }) => path),
    ["tests/tooling/differential-lsp-source-sessions.test.ts"],
  );
  const current = new Map(inputs.map((entry) => [entry.path, entry]));
  for (const change of changes) {
    assert.deepEqual(current.get(change.path), change.before, "current tooling preimage");
    assert.equal(change.after.path, change.path);
    current.set(change.path, change.after);
  }
  return sorted(current.values());
}

export function assertCleanCssSourceWorkingTree(repoRoot) {
  const status = execFileSync(
    "git",
    [
      "status",
      "--porcelain=v1",
      "--untracked-files=all",
      "--",
      "crates",
      ...reviewedBuildPaths,
      ...reviewedVsixPaths,
    ],
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

function verifyCssSnapshot(repoRoot, expectedRevision, pinnedProduct) {
  const git = (...args) =>
    execFileSync("git", args, { cwd: repoRoot, encoding: "utf8", maxBuffer: 32 * 1024 * 1024 });
  const readTree = (revision, paths, recursive = true) =>
    readGitTree(repoRoot, revision, paths, recursive);
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
  const head = git("rev-parse", "HEAD").trim();
  const revision = pinnedProduct ? contract.productTreeRevision : head;
  if (pinnedProduct) {
    assert.match(revision, /^[a-f0-9]{40}$/);
  } else {
    assert.equal(head, expectedRevision, "source-custody head must match the requested source");
    assertCleanCssSourceWorkingTree(repoRoot);
  }
  assert.equal(contract.mergedMain.sourceRevision, mergedMainRevision);
  if (!pinnedProduct) git("merge-base", "--is-ancestor", mergedMainRevision, revision);
  const baseline = readTree(historicalBaselineRevision, ["crates"]);
  assert.equal(
    git("rev-parse", `${historicalBaselineRevision}:crates`).trim(),
    contract.baselineCratesTreeOid,
  );
  assert.equal(fingerprint(baseline), contract.baselineCratesSha256);
  assert.equal(contract.historicalCapture.cratesSha256, contract.baselineCratesSha256);
  const p1 = composeReviewedCssTrees(baseline, [contract.layers[0]]);
  // The original P1 commit is no longer an ancestor after its squash merge.
  // Its exact immutable baseline/preimage/postimage composition remains pinned;
  // do not fetch an unreachable object or substitute a floating main revision.
  assert.equal(contract.historicalP1.sourceRevision, reviewedLayers[0].reviewedRevision);
  assert.equal(fingerprint(p1), contract.historicalP1.cratesSha256);
  assert.deepEqual(
    sorted(readTree(historicalBaselineRevision, historicalBuildPaths)),
    contract.historicalBuildInputs,
  );
  const integration = contract.mergedMain.integrationLayer;
  assert.deepEqual(
    {
      name: integration.name,
      baseRevision: integration.baseRevision,
      reviewedRevision: integration.reviewedRevision,
    },
    {
      name: "merged-main-integration-116a",
      baseRevision: reviewedLayers[0].reviewedRevision,
      reviewedRevision: mergedMainRevision,
    },
  );
  assert.equal(integration.changes.length, 34);
  const integrated = composeReviewedCssTrees(p1, [integration]);
  assert.equal(
    git("rev-parse", `${mergedMainRevision}:crates`).trim(),
    contract.mergedMain.cratesTreeOid,
  );
  assert.equal(fingerprint(integrated), contract.mergedMain.cratesSha256);
  assertExactCssTree(readTree(mergedMainRevision, ["crates"]), integrated);
  const successor = contract.successorMain;
  assert.equal(successor.sourceRevision, successorMainRevision);
  if (!pinnedProduct) git("merge-base", "--is-ancestor", successorMainRevision, revision);
  assert.equal(git("rev-parse", `${successorMainRevision}^{tree}`).trim(), successor.sourceTreeOid);
  assert.equal(successor.integrationLayer.baseRevision, mergedMainRevision);
  assert.equal(successor.integrationLayer.reviewedRevision, successorMainRevision);
  assert.equal(successor.integrationLayer.name, "merged-main-integration-9c8388e");
  assert.equal(successor.integrationLayer.changes.length, 122);
  const currentMain = composeReviewedCssTrees(integrated, [successor.integrationLayer]);
  assert.equal(git("rev-parse", `${successorMainRevision}:crates`).trim(), successor.cratesTreeOid);
  assert.equal(fingerprint(currentMain), successor.cratesSha256);
  assertExactCssTree(readTree(successorMainRevision, ["crates"]), currentMain);
  // The old Docs revision may be unreachable after rebase. Its reviewed
  // before/after blobs and modes are pinned in the committed contract; do not
  // infer a new base, fetch a floating ref, or skip a missing old revision.
  const currentContracts = contract.currentContracts;
  assert.equal(currentContracts.name, "stylesheet-current-completion-contract-10572");
  assert.equal(currentContracts.baseRevision, successorMainRevision);
  assert.deepEqual(
    currentContracts.changes.map(({ path }) => path),
    ["crates/vize_maestro/src/server/handlers/tests/completion_resolve.rs"],
  );
  const expected = composeReviewedCssTrees(currentMain, [
    ...contract.layers.slice(1),
    currentContracts,
  ]);
  assert.equal(contract.productSnapshot.sourceRevision, contract.productTreeRevision);
  assert.equal(
    contract.productSnapshot.cratesSha256,
    fingerprint(expected),
    "foreign product snapshot",
  );
  assert.equal(contract.productSnapshot.cratesEntries, expected.length);
  if (pinnedProduct)
    assertCssSnapshotProvenance(
      repoRoot,
      contractBytes,
      expected,
      contract.productSnapshot,
      contract.reviewedFixtures,
    );
  const actual = pinnedProduct ? expected : readTree(revision, ["crates"]);
  if (!pinnedProduct) assertExactCssTree(actual, expected);
  assert.deepEqual(
    sorted(readTree(mergedMainRevision, contract.buildPaths, false)),
    contract.mergedMain.buildInputs,
    "merged main build context changed",
  );
  assert.deepEqual(
    sorted(readTree(successorMainRevision, contract.buildPaths, false)),
    contract.buildInputs,
    "successor main build context changed",
  );
  assert.equal(contract.productSnapshot.buildInputsSha256, fingerprint(contract.buildInputs));
  if (!pinnedProduct)
    assert.deepEqual(
      sorted(readTree(revision, contract.buildPaths, false)),
      contract.buildInputs,
      "workspace build inputs changed",
    );
  const infrastructure = contract.vsixInfrastructure;
  assert.equal(infrastructure.baseRevision, successorMainRevision);
  assert.equal(infrastructure.sourceRevision, vsixInfrastructureRevision);
  assert.deepEqual(
    infrastructure.changes.map(({ path }) => path),
    reviewedVsixPaths,
  );
  if (!pinnedProduct) git("merge-base", "--is-ancestor", vsixInfrastructureRevision, revision);
  assert.deepEqual(
    sorted(readTree(successorMainRevision, reviewedVsixPaths)),
    sorted(infrastructure.changes.flatMap(({ before }) => (before ? [before] : []))),
  );
  const vsixInputs = sorted(infrastructure.changes.map(({ after }) => after));
  assert.deepEqual(sorted(readTree(vsixInfrastructureRevision, reviewedVsixPaths)), vsixInputs);
  const expectedVsixInputs = composeCurrentVsixInputs(vsixInputs, currentContracts.toolingChanges);
  assert.equal(contract.productSnapshot.vsixInputsSha256, fingerprint(expectedVsixInputs));
  if (!pinnedProduct)
    assert.deepEqual(
      sorted(readTree(revision, reviewedVsixPaths)),
      expectedVsixInputs,
      "VSIX infrastructure inputs changed",
    );
  assert.deepEqual(
    currentContracts.witnesses.map(({ path }) => path),
    [
      `tests/_fixtures/differential/lsp/css-hover-stylesheet-examples-10572/local-completion-resolve.expected-current.json`,
    ],
  );
  for (const witness of currentContracts.witnesses)
    assert.equal(
      hash(fs.readFileSync(path.join(repoRoot, witness.path))),
      witness.sha256,
      "current complete completion reference bytes",
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
    proofKind: pinnedProduct ? "historical-composition-and-owned-checkout" : "current-checkout",
    currentCheckoutRevision: head,
    currentSourceAcceptance: !pinnedProduct,
    currentHostAcceptance: false,
    contractSha256: hash(contractBytes),
    reviewedLayers: contract.layers,
    mergedMain: contract.mergedMain,
    successorMain: successor,
    currentContracts,
    vsixInfrastructure: infrastructure,
    cratesSha256: fingerprint(actual),
    productionChanges: [
      ...new Set(
        [...contract.layers, integration, successor.integrationLayer].flatMap((layer) =>
          layer.changes.map((change) => change.path),
        ),
      ),
    ].sort(compareStrings),
    reviewedCssChanges: [
      ...new Set(contract.layers.flatMap((layer) => layer.changes.map((change) => change.path))),
    ].sort(compareStrings),
    buildInputs: contract.buildInputs,
  };
}

// Ordinary checks authenticate a complete immutable historical composition and
// the current owned CSS inputs, including genuine one-parent squash commits.
// They grant no current-host acceptance. The strict full-depth verifier retains
// all historical main ancestry, complete HEAD/build-context and dirty-source gates.
export function verifyPinnedCssHoverProduct(repoRoot) {
  return verifyCssSnapshot(repoRoot, undefined, true);
}

export function verifyCssHoverSourceCustody(repoRoot, expectedRevision) {
  return verifyCssSnapshot(repoRoot, expectedRevision, false);
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
