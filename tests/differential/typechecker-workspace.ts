import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { sha256 } from "./harness.mjs";
import { summarizeNativeAcceptance } from "./acceptance-rates.mjs";
import { passedFixtureTests, typecheckerSourceIdentity } from "./typechecker-shards.ts";
import {
  TYPECHECKER_TESTS,
  loadTypecheckerManifest,
  validateWorkspaceTypecheckerCapture,
  workspaceTypecheckerReport,
  validateWorkspaceProducerExecution as execution,
  type WorkspaceCapture,
  type WorkspaceInventory as Inventory,
  type WorkspaceExecution,
  type WorkspaceSource as Source,
  type WorkspaceReceipt as Receipt,
} from "./typechecker.ts";
import {
  readWorkspaceInventory,
  verifyWorkspaceExecution,
} from "../../tools/support/compat/github/rust-workspace-inventory.ts";

export type { WorkspaceExecution } from "./typechecker.ts";
type WorkspaceOptions = {
  repoRoot: string;
  resultDir: string;
  expectedExecution: WorkspaceExecution;
};

const files = {
  receipt: "typechecker-workspace-receipt.json",
  captures: "typechecker-workspace",
  packet: "typechecker-workspace-observations.json",
  ts40: "template-emit-ts40-projection",
  inventory: "workspace-inventory-ci.json",
  timing: "workspace-timing.json",
  junit: "workspace-junit-ci.xml",
};
const matrixPath = "tests/_fixtures/davinci-ts40-projection/matrix.json";
const json = <T>(file: string): T => JSON.parse(fs.readFileSync(file, "utf8"));
const bytes = (file: string) => fs.readFileSync(file);
const save = (file: string, value: unknown) =>
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`, { flag: "wx" });
function locations(repoRoot: string, resultDir: string, producer = false) {
  const root = path.resolve(repoRoot, resultDir);
  if (producer) assert.equal(root, path.resolve(repoRoot, "target/nextest/full"));
  return Object.fromEntries(
    Object.entries(files).map(([key, name]) => [key, path.join(root, name)]),
  ) as Record<keyof typeof files, string>;
}
function sourceIdentity(source: Source, repoRoot: string) {
  const identity = typecheckerSourceIdentity(repoRoot);
  assert.equal(source.sha, identity.sha);
  assert.equal(source.tree, identity.tree);
  assert(source.rustc.startsWith(`rustc ${identity.toolchain} `));
  assert.equal(typeof source.cargo, "string");
  assert(source.cargo.startsWith("cargo "));
  assert.equal(source.platform, process.platform);
  assert.equal(source.arch, process.arch);
  assert.equal(typeof source.workspaceRoot, "string");
  assert(path.isAbsolute(source.workspaceRoot));
  assert.deepEqual(
    Object.keys(source.files).sort(),
    [
      ".cargo/config.toml",
      ".config/nextest.toml",
      "Cargo.lock",
      "Cargo.toml",
      "rust-toolchain.toml",
    ].sort(),
  );
  for (const [name, digest] of Object.entries(source.files))
    assert.equal(sha256(bytes(path.join(repoRoot, name))), digest);
  assert.deepEqual(
    Object.keys(source.runtime).sort(),
    [
      "VIZE_TEST_REQUIRE_TSGO",
      "VIZE_TEST_DISABLE_TSGO",
      "VIZE_NUXT_CONFIG_ITERATIONS",
      "RUST_TEST_THREADS",
      "CARGO_BUILD_JOBS",
      "RUSTFLAGS",
      "RUSTDOCFLAGS",
      "CARGO_ENCODED_RUSTFLAGS",
      "CARGO_TARGET_DIR",
    ].sort(),
  );
  for (const value of Object.values(source.runtime))
    assert(value === null || typeof value === "string");
  assert.equal(source.runtime.RUST_TEST_THREADS, "4");
  // Build parallelism is the existing Q producer's 24, not MQ's 12.
  assert.equal(source.runtime.CARGO_BUILD_JOBS, "24");
  assert.equal(source.runtime.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.equal(source.runtime.VIZE_TEST_DISABLE_TSGO, null);
  assert.equal(source.runtime.VIZE_NUXT_CONFIG_ITERATIONS, "100");
  return identity;
}
function selectedBinary(raw: unknown) {
  const suites = (
    raw as {
      "rust-suites": Record<
        string,
        {
          "binary-id": string;
          "package-name": string;
          "binary-name": string;
          "binary-path": string;
        }
      >;
    }
  )["rust-suites"];
  const selected = Object.values(suites).filter(
    (suite) =>
      suite["package-name"] === "vize_canon" && suite["binary-name"] === "fix_history_diagnostics",
  );
  assert.equal(selected.length, 1, "one genuine listed Canon fixture executable required");
  const suite = selected[0];
  assert.equal(suite["binary-id"], "vize_canon::fix_history_diagnostics");
  assert.equal(typeof suite["binary-path"], "string");
  return {
    id: suite["binary-id"],
    package: suite["package-name"],
    name: suite["binary-name"],
    path: suite["binary-path"],
  };
}
function physicalBinary(binary: Receipt["binary"], source: Source) {
  const root = fs.realpathSync(source.workspaceRoot);
  const actual = fs.realpathSync(binary.path);
  assert.equal(actual, path.resolve(binary.path), "listed executable must already be canonical");
  const relative = path.relative(root, actual).replaceAll("\\", "/");
  assert.match(relative, /^target\/ci\/deps\/fix_history_diagnostics-[a-f0-9]+(?:\.exe)?$/);
  const stat = fs.lstatSync(actual);
  assert(stat.isFile());
  if (process.platform !== "win32") assert(stat.mode & 0o111, "listed file is not executable");
  assert.equal(sha256(bytes(actual)), binary.sha256, "listed pre-run executable changed");
  return binary.sha256;
}
function requiredCases(inventory: Inventory) {
  for (const [id, names] of [
    ["vize_canon::fix_history_diagnostics", Object.values(TYPECHECKER_TESTS)],
    [
      "vize_maestro::davinci_ts40_projection",
      [
        "current_projection_matrix_is_exact_and_non_empty",
        "mapping_drift_fails_closed",
        "diagnostic_drift_fails_closed",
      ],
    ],
  ] as const) {
    const suites = inventory.binaries.filter((item) => item.id === id);
    assert.equal(suites.length, 1);
    assert.deepEqual(
      suites[0].cases,
      [...names].sort((a, b) => a.localeCompare(b, "en")).map((name) => ({ name, ignored: false })),
    );
  }
}
export function prepareTypecheckerWorkspace({
  repoRoot,
  resultDir,
  inventoryPath,
  source,
  execution: producer,
  nextestVersion,
  environment = process.env,
}: {
  repoRoot: string;
  resultDir: string;
  inventoryPath: string;
  source: Source;
  execution: WorkspaceExecution;
  nextestVersion: string;
  environment?: NodeJS.ProcessEnv;
}) {
  const locations_ = locations(repoRoot, resultDir, true);
  execution(producer);
  assert.equal(nextestVersion, "0.9.146");
  assert.equal(path.resolve(repoRoot, inventoryPath), locations_.inventory);
  for (const key of [
    "VIZE_TEST_FIX_HISTORY_CAPTURE_DIR",
    "VIZE_TEST_FIX_HISTORY_ARCHIVE_RECEIPT",
    "VIZE_TEST_FIX_HISTORY_WORKSPACE_RECEIPT",
    "VIZE_TEMPLATE_EMIT_TS40_CAPTURE",
  ])
    assert.equal(environment[key], undefined, `ambient capture authority conflicts: ${key}`);
  const inventory = readWorkspaceInventory(locations_.inventory) as Inventory;
  assert.equal(inventory.cargoProfile, "ci");
  assert.deepEqual(inventory.source, source);
  sourceIdentity(source, repoRoot);
  requiredCases(inventory);
  assert.equal(fs.realpathSync(source.workspaceRoot), fs.realpathSync(repoRoot));
  const selected = selectedBinary(
    json(path.join(path.dirname(locations_.inventory), inventory.rawFile)),
  );
  const binary = {
    ...selected,
    listedPath: selected.path,
    path: fs.realpathSync(selected.path),
    sha256: sha256(bytes(selected.path)),
  };
  physicalBinary(binary, source);
  const loaded = loadTypecheckerManifest(
    path.join(repoRoot, "tests/_fixtures/differential/typechecker/manifest.json"),
  );
  const receipt: Receipt = {
    schema: "vize.typechecker-workspace-receipt",
    version: 1,
    kind: "full-workspace",
    source,
    execution: producer,
    cargoProfile: "ci",
    nextestVersion: "0.9.146",
    inventorySha256: sha256(bytes(locations_.inventory)),
    rawInventorySha256: inventory.rawSha256,
    manifestSha256: loaded.manifestSha256,
    ts40MatrixSha256: sha256(bytes(path.join(repoRoot, matrixPath))),
    binary,
  };
  // Exclusive fresh directories and receipt retain any stale collision for diagnosis.
  fs.mkdirSync(locations_.captures);
  fs.mkdirSync(locations_.ts40);
  save(locations_.receipt, receipt);
  return {
    receiptPath: locations_.receipt,
    captureDir: locations_.captures,
    ts40Dir: locations_.ts40,
    childEnvironment: {
      VIZE_TEST_FIX_HISTORY_CAPTURE_DIR: locations_.captures,
      VIZE_TEST_FIX_HISTORY_WORKSPACE_RECEIPT: locations_.receipt,
      VIZE_TEMPLATE_EMIT_TS40_CAPTURE: locations_.ts40,
    },
  };
}
function capturedFiles(root: string): string[] {
  assert(fs.lstatSync(root).isDirectory() && !fs.lstatSync(root).isSymbolicLink());
  return fs
    .readdirSync(root)
    .sort()
    .flatMap((name) => {
      const file = path.join(root, name);
      const stat = fs.lstatSync(file);
      assert(!stat.isSymbolicLink(), "capture tree cannot contain symlinks");
      if (stat.isDirectory()) return capturedFiles(file).map((part) => `${name}/${part}`);
      assert(stat.isFile(), "capture tree contains a special file");
      return [name];
    })
    .sort();
}
// The original Rust matrix/snapshot test remains the oracle. Retain and bind all
// raw projection bytes to that unchanged current snapshot, without parity credit.
function ts40Captures(repoRoot: string, captureRoot: string) {
  const matrix = json<{
    fixtures: Array<{ id: string; file: string; lineEnding: string; legacyVue2: boolean }>;
  }>(path.join(repoRoot, matrixPath));
  const expected = new Set<string>();
  const read = (relative: string) => {
    expected.add(relative);
    return bytes(path.join(captureRoot, relative));
  };
  for (const fixture of matrix.fixtures) {
    let original = fs.readFileSync(path.join(repoRoot, fixture.file), "utf8");
    if (fixture.lineEnding === "crlf")
      original = original.replaceAll("\r\n", "\n").replaceAll("\n", "\r\n");
    assert.equal(read(`${fixture.id}/original.vue`).toString("utf8"), original);
    const legacy = fixture.legacyVue2;
    // Canonical workspace argv has no features; Maestro's native/glyph defaults
    // and the current workspace dependency graph do not enable legacy generators.
    const feature = legacy ? "mixed-vue2__legacy-generators-disabled" : "default";
    const snapshot = fs.readFileSync(
      path.join(
        repoRoot,
        `crates/vize_maestro/tests/snapshots/davinci_ts40_projection__davinci_ts40__${fixture.id}__${feature}.snap`,
      ),
      "utf8",
    );
    assert(snapshot.includes(`source-sha256=${sha256(Buffer.from(original))}\n`));
    const lanes = {
      canon: {
        "text.ts": "text",
        "pre-rewrite-text.ts": "pre-rewrite-text",
        "mappings.txt": "mappings",
        "semantic-links.txt": "semantic-links",
        "import-map.txt": "import-source-map",
        "import-probes.txt": "import-source-map-probes",
      },
      "content-mapper": {
        "text.ts": "text",
        "mappings.txt": "mappings",
        "semantic-links.txt": "semantic-links",
        "diagnostics.txt": "diagnostics",
        "authored-hits.txt": "authored-hits",
      },
      maestro: {
        "text.txt": "text",
        "mappings.txt": "mappings",
        "authored-hits.txt": "authored-hits",
      },
    };
    for (const [lane, parts] of Object.entries(lanes)) {
      if (legacy && lane !== "content-mapper") continue;
      const section = snapshot.split(`[${lane}]\n`)[1]?.split("\n[")[0];
      assert(section);
      assert(/^status=ok(?::[^\n]+)?$/m.test(section), "unexpected TS40 error lane");
      for (const [file, key] of Object.entries(parts)) {
        const fields = section.split("\n").filter((row) => row.startsWith(`${key}=`));
        assert.equal(fields.length, 1);
        assert.equal(sha256(read(`${fixture.id}/${lane}/${file}`)), fields[0].slice(-64));
      }
      if (lane === "maestro") {
        const documents = bytes(path.join(captureRoot, fixture.id, "maestro/text.txt"))
          .toString("utf8")
          .split("\n")
          .filter(Boolean);
        assert(documents.length > 0);
        for (const [index, document] of documents.entries()) {
          const fields = /\|[^|]+\|(\d+)\|([a-f0-9]{64})$/.exec(document);
          assert(fields);
          const raw = read(`${fixture.id}/maestro-document-${index}/text.ts`);
          assert.equal(raw.length, Number(fields[1]));
          assert.equal(sha256(raw), fields[2]);
        }
      }
    }
  }
  assert.deepEqual(
    capturedFiles(captureRoot),
    [...expected].sort(),
    "incomplete/unknown TS40 projection files",
  );
  return [...expected].sort().map((name) => {
    const raw = bytes(path.join(captureRoot, name));
    return { path: name, bytes: raw.length, sha256: sha256(raw) };
  });
}
function validate({ repoRoot, resultDir, expectedExecution }: WorkspaceOptions, producer = false) {
  execution(expectedExecution);
  const locations_ = locations(repoRoot, resultDir, producer);
  const receiptBytes = bytes(locations_.receipt);
  const receipt = JSON.parse(receiptBytes.toString("utf8")) as Receipt;
  assert.equal(receipt.schema, "vize.typechecker-workspace-receipt");
  assert.equal(receipt.version, 1);
  assert.equal(receipt.kind, "full-workspace");
  assert(
    !Object.hasOwn(receipt, "archiveSha256") && !Object.hasOwn(receipt, "archiveReceiptSha256"),
  );
  assert.equal(receipt.cargoProfile, "ci");
  assert.equal(receipt.nextestVersion, "0.9.146");
  assert.deepEqual(receipt.execution, expectedExecution);
  sourceIdentity(receipt.source, repoRoot);
  const inventory = readWorkspaceInventory(locations_.inventory) as Inventory;
  requiredCases(inventory);
  assert.deepEqual(inventory.source, receipt.source);
  assert.equal(sha256(bytes(locations_.inventory)), receipt.inventorySha256);
  assert.equal(inventory.rawSha256, receipt.rawInventorySha256);
  assert.deepEqual(
    selectedBinary(json(path.join(path.dirname(locations_.inventory), inventory.rawFile))),
    {
      id: receipt.binary.id,
      package: receipt.binary.package,
      name: receipt.binary.name,
      path: receipt.binary.listedPath,
    },
  );
  assert.match(receipt.binary.sha256, /^[a-f0-9]{64}$/);
  // Downloaded consumers validate the original canonical producer path, not a
  // nonexistent local copy of its executable. Only finish performs physical I/O.
  const relative = path
    .relative(receipt.source.workspaceRoot, receipt.binary.path)
    .replaceAll("\\", "/");
  assert.match(relative, /^target\/ci\/deps\/fix_history_diagnostics-[a-f0-9]+(?:\.exe)?$/);
  const timingBytes = bytes(locations_.timing);
  const timing = JSON.parse(timingBytes.toString("utf8"));
  assert.deepEqual(timing.execution, {
    runId: String(expectedExecution.runId),
    attempt: String(expectedExecution.runAttempt),
  });
  const junit = bytes(locations_.junit);
  verifyWorkspaceExecution(inventory, timing, junit.toString("utf8"), receipt.source.sha, "ci");
  const loaded = loadTypecheckerManifest(
    path.join(repoRoot, "tests/_fixtures/differential/typechecker/manifest.json"),
  );
  assert.equal(loaded.manifestSha256, receipt.manifestSha256);
  assert.equal(sha256(bytes(path.join(repoRoot, matrixPath))), receipt.ts40MatrixSha256);
  const tests = passedFixtureTests(junit.toString("utf8"));
  assert.deepEqual(new Set(tests), new Set(Object.values(TYPECHECKER_TESTS)));
  assert.deepEqual(
    capturedFiles(locations_.captures),
    Object.keys(TYPECHECKER_TESTS)
      .map((pack) => `${pack}.json`)
      .sort(),
  );
  const captures = Object.keys(TYPECHECKER_TESTS).map((pack) => {
    const capture = json<WorkspaceCapture>(path.join(locations_.captures, `${pack}.json`));
    assert.equal(capture.pack, pack);
    assert.equal(capture.binaryPath, receipt.binary.path);
    return validateWorkspaceTypecheckerCapture(loaded, capture, {
      receipt: receiptBytes,
      binarySha256: receipt.binary.sha256,
    });
  });
  const projection = ts40Captures(repoRoot, locations_.ts40);
  return {
    loaded,
    captures,
    receipt,
    packet: {
      schema: "vize.typechecker-workspace-observations",
      version: 1,
      kind: "full-workspace",
      sourceRevision: receipt.source.sha,
      sourceTree: receipt.source.tree,
      execution: receipt.execution,
      manifestSha256: loaded.manifestSha256,
      receiptBase64: receiptBytes.toString("base64"),
      inventorySha256: receipt.inventorySha256,
      rawInventorySha256: receipt.rawInventorySha256,
      timingSha256: sha256(timingBytes),
      junitSha256: sha256(junit),
      tests,
      captures,
      ts40: projection,
      executable: {
        path: receipt.binary.path,
        preRunSha256: receipt.binary.sha256,
        postRunSha256: receipt.binary.sha256,
      },
    },
  };
}
export function finishTypecheckerWorkspace(options: WorkspaceOptions) {
  const verified = validate(options, true);
  assert.equal(
    fs.realpathSync(verified.receipt.source.workspaceRoot),
    fs.realpathSync(options.repoRoot),
  );
  physicalBinary(verified.receipt.binary, verified.receipt.source);
  save(locations(options.repoRoot, options.resultDir).packet, verified.packet);
  return verified.packet;
}
export function aggregateTypecheckerWorkspace(options: WorkspaceOptions) {
  const verified = validate(options);
  const locations_ = locations(options.repoRoot, options.resultDir);
  assert.deepEqual(
    json(locations_.packet),
    verified.packet,
    "workspace packet/raw producer chain differs",
  );
  const report = workspaceTypecheckerReport(
    verified.loaded,
    verified.captures,
    verified.receipt.source.sha,
  );
  const acceptance = summarizeNativeAcceptance(verified.loaded, report, {
    sourceRevision: verified.receipt.source.sha,
    buildReceiptSha256: sha256(bytes(locations_.receipt)),
    requiredStages: ["parse", "facts", "emit", "check"],
  });
  assert.equal(acceptance.total.nativeHandled, 0);
  assert.equal(acceptance.total.nativeEquivalent, 0);
  return { report, acceptance, packet: verified.packet };
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, sha, resultDir, runId, attempt] = process.argv.slice(2);
  assert.equal(mode, "verify");
  assert(sha && resultDir && runId && attempt);
  assert.equal(typecheckerSourceIdentity(process.cwd()).sha, sha);
  const expectedExecution = { runId: Number(runId), runAttempt: Number(attempt) };
  execution(expectedExecution);
  assert.equal(String(expectedExecution.runId), process.env.GITHUB_RUN_ID);
  assert(Number(process.env.GITHUB_RUN_ATTEMPT) >= expectedExecution.runAttempt);
  const result = aggregateTypecheckerWorkspace({
    repoRoot: process.cwd(),
    resultDir: path.resolve(resultDir),
    expectedExecution,
  });
  for (const [name, value] of Object.entries({
    report: result.report,
    acceptance: result.acceptance,
  }))
    fs.writeFileSync(
      path.join(resultDir, `typechecker-workspace-${name}.json`),
      `${JSON.stringify(value, null, 2)}\n`,
    );
  console.log(JSON.stringify(result.report.summary));
}
