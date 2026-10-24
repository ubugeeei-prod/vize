import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
const root = fileURLToPath(new URL("../../", import.meta.url));

// Controlled protocol fixtures never execute the stand-in binary or claim native
// acceptance. Real source/Rust/TS40 observations remain an Actions obligation.
import { execFileSync, spawnSync } from "node:child_process";
import { tmpdir } from "node:os";
import { sha256 } from "../differential/harness.mjs";
import {
  TYPECHECKER_TESTS,
  loadTypecheckerManifest,
  type WorkspaceCapture,
  type WorkspaceSource,
} from "../differential/typechecker.ts";
import {
  prepareTypecheckerWorkspace,
  finishTypecheckerWorkspace,
  aggregateTypecheckerWorkspace,
} from "../differential/typechecker-workspace.ts";
import { normalizeWorkspaceInventory } from "../../tools/support/compat/github/rust-workspace-inventory.ts";

const execution = { runId: 53, runAttempt: 2 };
const names = Object.values(TYPECHECKER_TESTS);
const ts40Names = [
  "current_projection_matrix_is_exact_and_non_empty",
  "mapping_drift_fails_closed",
  "diagnostic_drift_fails_closed",
];
function controlled(t: { after: (callback: () => void) => void }) {
  const repoRoot = fs.realpathSync(
    fs.mkdtempSync(path.join(tmpdir(), "vize-workspace-observation-control-")),
  );
  t.after(() => fs.rmSync(repoRoot, { recursive: true, force: true }));
  const write = (file: string, value: string) => {
    fs.mkdirSync(path.dirname(path.join(repoRoot, file)), { recursive: true });
    fs.writeFileSync(path.join(repoRoot, file), value);
  };
  const save = (file: string, value: unknown) => write(file, `${JSON.stringify(value, null, 2)}\n`);
  const matrix = "tests/_fixtures/davinci-ts40-projection/matrix.json";
  save(matrix, {
    schemaVersion: 1,
    fixtures: [{ id: "controlled", file: "controlled.vue", lineEnding: "lf", legacyVue2: false }],
  });
  write("controlled.vue", "<template>controlled protocol only</template>\n");
  const projection: Record<string, string> = {
    "original.vue": fs.readFileSync(path.join(repoRoot, "controlled.vue"), "utf8"),
    "canon/text.ts": "controlled projection\n",
    "canon/pre-rewrite-text.ts": "controlled projection\n",
    "canon/mappings.txt": "controlled map",
    "canon/semantic-links.txt": "",
    "canon/import-map.txt": "",
    "canon/import-probes.txt": "",
    "content-mapper/text.ts": "controlled projection\n",
    "content-mapper/mappings.txt": "controlled map",
    "content-mapper/semantic-links.txt": "",
    "content-mapper/diagnostics.txt": "",
    "content-mapper/authored-hits.txt": "controlled hit",
    "maestro/text.txt": `controlled.vue.ts|TypeScript|3|${sha256(Buffer.from("ts\n"))}`,
    "maestro/mappings.txt": "controlled map",
    "maestro/authored-hits.txt": "controlled hit",
    "maestro-document-0/text.ts": "ts\n",
  };
  const snapshot = [
    `fixture=controlled`,
    `source-sha256=${sha256(Buffer.from(projection["original.vue"]))}`,
  ];
  for (const lane of ["canon", "content-mapper", "maestro"]) {
    snapshot.push(`[${lane}]`, "status=ok");
    for (const [file, value] of Object.entries(projection).filter(([name]) =>
      name.startsWith(`${lane}/`),
    )) {
      const stem = path.basename(file).replace(/\.(ts|txt)$/, "");
      const key =
        stem === "import-map"
          ? "import-source-map"
          : stem === "import-probes"
            ? "import-source-map-probes"
            : stem;
      snapshot.push(`${key}=0:${sha256(Buffer.from(value))}`);
    }
  }
  write(
    "crates/vize_maestro/tests/snapshots/davinci_ts40_projection__davinci_ts40__controlled__default.snap",
    snapshot.join("\n") + "\n",
  );
  const fixtureRoot = "tests/_fixtures/differential/typechecker";
  fs.mkdirSync(path.join(repoRoot, path.dirname(fixtureRoot)), { recursive: true });
  fs.cpSync(path.join(root, fixtureRoot), path.join(repoRoot, fixtureRoot), { recursive: true });
  const sourceFiles = {
    "Cargo.toml": '[workspace]\nmembers = []\n[profile.ci]\ninherits = "dev"\n',
    "Cargo.lock": "version = 4\n",
    "rust-toolchain.toml": '[toolchain]\nchannel = "1.99.1"\n',
    ".cargo/config.toml": "# controlled\n",
    ".config/nextest.toml": "[profile.full]\nretries = 0\n",
  };
  for (const [file, value] of Object.entries(sourceFiles)) write(file, value);
  const git = (...args: string[]) =>
    execFileSync("git", args, {
      cwd: repoRoot,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  git("init", "--quiet");
  git("add", ".");
  git("config", "user.name", "Protocol Control");
  git("config", "user.email", "control@example.invalid");
  git("commit", "--quiet", "-m", "controlled source, no native execution");
  const source: WorkspaceSource = {
    sha: git("rev-parse", "HEAD"),
    tree: git("rev-parse", "HEAD^{tree}"),
    files: Object.fromEntries(
      Object.entries(sourceFiles).map(([file, value]) => [file, sha256(Buffer.from(value))]),
    ),
    rustc: "rustc 1.99.1 controlled",
    cargo: "cargo 1.99.1 controlled",
    platform: process.platform,
    arch: process.arch,
    workspaceRoot: repoRoot,
    runtime: {
      VIZE_TEST_REQUIRE_TSGO: "1",
      VIZE_TEST_DISABLE_TSGO: null,
      VIZE_NUXT_CONFIG_ITERATIONS: "100",
      RUST_TEST_THREADS: "4",
      CARGO_BUILD_JOBS: "24",
      RUSTFLAGS: null,
      RUSTDOCFLAGS: null,
      CARGO_ENCODED_RUSTFLAGS: null,
      CARGO_TARGET_DIR: null,
    },
    inferredProfileDefaults: {
      basis: "committed-manifest-config-and-runtime-environment",
      debugAssertions: true,
      overflowChecks: true,
    },
  };
  const resultDir = path.join(repoRoot, "target/nextest/full");
  fs.mkdirSync(resultDir, { recursive: true });
  const binary = path.join(repoRoot, "target/ci/deps/fix_history_diagnostics-cafe");
  write(path.relative(repoRoot, binary), "controlled bytes, never executed");
  fs.chmodSync(binary, 0o755);
  const suite = (id: string, packageName: string, binaryName: string, tests: string[]) => ({
    "binary-id": id,
    "package-name": packageName,
    "binary-name": binaryName,
    "binary-path": binary,
    status: "listed",
    testcases: Object.fromEntries(
      tests.map((name) => [name, { ignored: false, "filter-match": { status: "matches" } }]),
    ),
  });
  const raw = {
    "rust-suites": {
      "vize_canon::fix_history_diagnostics": suite(
        "vize_canon::fix_history_diagnostics",
        "vize_canon",
        "fix_history_diagnostics",
        names,
      ),
      "vize_maestro::davinci_ts40_projection": suite(
        "vize_maestro::davinci_ts40_projection",
        "vize_maestro",
        "davinci_ts40_projection",
        ts40Names,
      ),
    },
  };
  const rawFile = "workspace-inventory-ci.raw.json";
  save(`target/nextest/full/${rawFile}`, raw);
  const inventory = {
    schema: "vize.rust.workspace-inventory",
    schemaVersion: 1,
    source,
    cargoProfile: "ci",
    rawFile,
    rawSha256: sha256(fs.readFileSync(path.join(resultDir, rawFile))),
    binaries: normalizeWorkspaceInventory(raw),
  };
  const inventoryPath = path.join(resultDir, "workspace-inventory-ci.json");
  save(path.relative(repoRoot, inventoryPath), inventory);
  const loaded = loadTypecheckerManifest(path.join(repoRoot, fixtureRoot, "manifest.json"));
  const options = { repoRoot, resultDir, expectedExecution: execution };
  const prepare = (environment: NodeJS.ProcessEnv = {}, resultDir = options.resultDir) =>
    prepareTypecheckerWorkspace({
      ...options,
      resultDir,
      inventoryPath: path.join(resultDir, path.basename(inventoryPath)),
      source,
      execution,
      nextestVersion: "0.9.146",
      environment,
    });
  const complete = () => {
    const receipt = fs.readFileSync(path.join(resultDir, "typechecker-workspace-receipt.json"));
    const captures = Object.entries(TYPECHECKER_TESTS).map(([pack, name]) => {
      const fixtures = loaded.cases.filter((item) => item.pack === pack);
      const capture: WorkspaceCapture = {
        schema: "vize.typechecker-fixture-observation",
        version: 2,
        kind: "full-workspace",
        pack: pack as WorkspaceCapture["pack"],
        test: name,
        fixturePackSha256: fixtures[0].packReference.sha256,
        workspaceReceiptSha256: sha256(receipt),
        binaryPath: binary,
        binarySha256: sha256(fs.readFileSync(binary)),
        missingFields: ["end", "relatedInformation", "raw backend diagnostics"],
        matchedContract: "batch-start-diagnostics-v1",
        unbaselinedFields: ["diagnostics[].blockType", "exitCode", "success"],
        cases: fixtures.map((fixture) => ({
          id: fixture.case,
          projectRoot: "/controlled/project",
          inputs: fixture.inputs.map((input) => ({ file: input.file, sha256: input.sha256 })),
          diagnostics: structuredClone(fixture.diagnostics),
          publicResult: {
            exitCode: 0,
            success: true,
            diagnostics: fixture.diagnostics.map((diagnostic) => ({
              ...diagnostic,
              file: `/controlled/project/${diagnostic.file}`,
              line: diagnostic.line - 1,
              column: diagnostic.column - 1,
              blockType: null,
            })),
          },
        })),
      };
      save(`target/nextest/full/typechecker-workspace/${pack}.json`, capture);
      return capture;
    });
    for (const [file, value] of Object.entries(projection))
      write(`target/nextest/full/template-emit-ts40-projection/controlled/${file}`, value);
    const junit = Object.entries(raw["rust-suites"])
      .flatMap(([id, suite]) =>
        Object.keys(suite.testcases).map((name) => `<testcase classname="${id}" name="${name}"/>`),
      )
      .join("");
    write("target/nextest/full/workspace-junit-ci.xml", junit);
    save("target/nextest/full/workspace-timing.json", {
      sourceSha: source.sha,
      source,
      cargoProfile: "ci",
      runner: "nextest",
      inventoryOnly: false,
      status: "passed",
      execution: { runId: String(execution.runId), attempt: String(execution.runAttempt) },
      phases: [
        {
          name: "inventory",
          exitCode: 0,
          args: [
            "nextest",
            "list",
            "--locked",
            "--workspace",
            "--cargo-profile",
            "ci",
            "--profile",
            "full",
            "--run-ignored",
            "all",
            "--message-format",
            "json",
          ],
        },
        {
          name: "workspace",
          exitCode: 0,
          args: [
            "nextest",
            "run",
            "--locked",
            "--workspace",
            "--cargo-profile",
            "ci",
            "--profile",
            "full",
          ],
        },
        {
          name: "doctests",
          exitCode: 0,
          args: ["test", "--locked", "--workspace", "--profile", "ci", "--doc"],
        },
      ],
    });
    return captures;
  };
  return {
    ...options,
    binary,
    source,
    raw,
    inventory,
    inventoryPath,
    write,
    save,
    prepare,
    complete,
  };
}

type Fixture = ReturnType<typeof controlled>;
const artifact = (f: Fixture, name: string) => path.join(f.resultDir, name);
void test("workspace producer/consumer bind physical bytes, all registered bodies, complete JUnit and raw TS40 without native credit", (t) => {
  const f = controlled(t);
  const prepared = f.prepare({}, "target/nextest/full");
  const env = prepared.childEnvironment;
  assert(Object.values(env).every((value) => path.isAbsolute(value)));
  assert(!Object.hasOwn(env, "VIZE_TEST_FIX_HISTORY_CAPTURE_DIR"));
  assert.equal(env.VIZE_NATIVE_SYNTAX_CAPTURE_DIR, artifact(f, "typechecker-fixtures"));
  assert(fs.lstatSync(env.VIZE_NATIVE_SYNTAX_CAPTURE_DIR).isDirectory());
  assert.equal(env.VIZE_TEST_FIX_HISTORY_WORKSPACE_CAPTURE_DIR, prepared.captureDir);
  assert(fs.readFileSync(path.resolve(f.repoRoot, "crates/vize_maestro", prepared.receiptPath)));
  assert(!Object.hasOwn(process.env, "VIZE_TEST_FIX_HISTORY_WORKSPACE_RECEIPT"));
  assert.equal(env.VIZE_TEST_FIX_HISTORY_WORKSPACE_RECEIPT, prepared.receiptPath);
  f.complete();
  const packet = finishTypecheckerWorkspace(f);
  assert.equal(packet.captures.length, names.length);
  assert.equal(packet.execution.runAttempt, 2);
  fs.unlinkSync(f.binary); // A downloaded consumer must not pretend to have the producer executable.
  const downloaded = path.join(f.repoRoot, ".artifacts/rust-full-workspace");
  fs.cpSync(f.resultDir, downloaded, { recursive: true });
  const result = aggregateTypecheckerWorkspace({ ...f, resultDir: downloaded });
  assert.equal(result.report.summary.legacyMatches, 94);
  assert.equal(result.acceptance.total.nativeHandled, 0);
  assert.equal(result.acceptance.total.nativeEquivalent, 0);
  assert(
    result.report.rows.every(
      (row) => !Object.hasOwn(row.legacy.provenance, "archiveReceiptSha256"),
    ),
  );
  assert.throws(() =>
    aggregateTypecheckerWorkspace({ ...f, expectedExecution: { ...execution, runAttempt: 3 } }),
  );
  assert.throws(() => finishTypecheckerWorkspace(f));
});

void test("pre-run attestation refuses stale capture state, changed provenance, excluded tests and conflicting ambient modes", (t) => {
  for (const key of [
    "VIZE_NATIVE_SYNTAX_CAPTURE_DIR",
    "VIZE_TEST_FIX_HISTORY_CAPTURE_DIR",
    "VIZE_TEST_FIX_HISTORY_WORKSPACE_CAPTURE_DIR",
    "VIZE_TEST_FIX_HISTORY_ARCHIVE_RECEIPT",
    "VIZE_TEST_FIX_HISTORY_WORKSPACE_RECEIPT",
    "VIZE_TEMPLATE_EMIT_TS40_CAPTURE",
  ]) {
    const f = controlled(t);
    assert.throws(() => f.prepare({ [key]: "" }), key);
  }
  for (const change of [
    (f: Fixture) => {
      f.source.tree = "0".repeat(40);
    },
    (f: Fixture) => {
      f.source.rustc = "rustc 1.98.0 controlled";
    },
    (f: Fixture) => {
      f.source.runtime.RUST_TEST_THREADS = "1";
    },
    (f: Fixture) => {
      f.source.runtime.CARGO_BUILD_JOBS = "12";
    },
    (f: Fixture) => {
      f.inventory.cargoProfile = "test";
    },
    (f: Fixture) => {
      fs.unlinkSync(f.binary);
    },
    (f: Fixture) => {
      delete f.raw["rust-suites"]["vize_canon::fix_history_diagnostics"].testcases[names[0]];
    },
    (f: Fixture) => {
      f.raw["rust-suites"]["vize_canon::fix_history_diagnostics"].testcases[names[0]].ignored =
        true;
    },
    (f: Fixture) => {
      f.raw["rust-suites"]["vize_canon::fix_history_diagnostics"]["binary-path"] = f.inventoryPath;
    },
  ]) {
    const f = controlled(t);
    change(f);
    const rawFile = "target/nextest/full/workspace-inventory-ci.raw.json";
    f.save(rawFile, f.raw);
    f.inventory.rawSha256 = sha256(fs.readFileSync(path.join(f.repoRoot, rawFile)));
    f.inventory.binaries = normalizeWorkspaceInventory(f.raw);
    f.save(path.relative(f.repoRoot, f.inventoryPath), f.inventory);
    assert.throws(() => f.prepare(), String(change));
  }
  const stale = controlled(t);
  stale.prepare();
  assert.throws(() => stale.prepare());
});

void test("no missing, forged, failed, stale or modified workspace chain can produce a successful packet", (t) => {
  const mutateJson = (f: Fixture, file: string, field: string, replacement: unknown) => {
    const value = JSON.parse(fs.readFileSync(artifact(f, file), "utf8")) as Record<string, unknown>;
    const parts = field.split(".");
    const target = parts
      .slice(0, -1)
      .reduce((node, part) => node[part] as Record<string, unknown>, value);
    target[parts.at(-1)!] = replacement;
    f.save(`target/nextest/full/${file}`, value);
  };
  const jsonChanges: Array<[string, string, unknown]> = [
    ["typechecker-workspace-receipt.json", "execution.runAttempt", 3],
    ["typechecker-workspace-receipt.json", "kind", "archive"],
    ["workspace-timing.json", "phases.2.exitCode", 7],
    ["workspace-timing.json", "execution.attempt", "3"],
    ["typechecker-workspace/event-handler-narrowing.json", "version", 1],
    ["typechecker-workspace/event-handler-narrowing.json", "archiveReceiptSha256", "0".repeat(64)],
    [
      "typechecker-workspace/event-handler-narrowing.json",
      "cases.0.inputs.0.sha256",
      "0".repeat(64),
    ],
    [
      "typechecker-workspace/event-handler-narrowing.json",
      "cases.0.publicResult.exitCode",
      undefined,
    ],
  ];
  const changes: Array<(f: Fixture) => void> = [
    (f) => f.write("target/nextest/full/typechecker-workspace/foreign.json", "{}"),
    ...jsonChanges.map(
      ([file, field, replacement]) =>
        (f: Fixture) =>
          mutateJson(f, file, field, replacement),
    ),
    (f) => {
      fs.appendFileSync(f.binary, " changed after preparation");
    },
    (f) => {
      fs.unlinkSync(artifact(f, "workspace-inventory-ci.raw.json"));
    },
    (f) => {
      f.write("target/nextest/full/workspace-junit-ci.xml", "");
    },
    (f) => {
      fs.appendFileSync(
        artifact(f, "workspace-junit-ci.xml"),
        `<testcase classname="vize_canon::fix_history_diagnostics" name="${names[0]}"/>`,
      );
    },
    (f) => {
      const file = artifact(f, "workspace-junit-ci.xml");
      fs.writeFileSync(file, fs.readFileSync(file, "utf8").replace("/>", "><skipped/></testcase>"));
    },
    (f) => {
      fs.unlinkSync(artifact(f, "typechecker-workspace/event-handler-narrowing.json"));
    },
    (f) => {
      fs.unlinkSync(
        path.join(f.resultDir, "template-emit-ts40-projection/controlled/canon/mappings.txt"),
      );
    },
    (f) => {
      fs.appendFileSync(
        artifact(f, "template-emit-ts40-projection/controlled/maestro-document-0/text.ts"),
        "lost raw projection",
      );
    },
  ];
  for (const change of changes) {
    const f = controlled(t);
    f.prepare();
    f.complete();
    change(f);
    assert.throws(() => finishTypecheckerWorkspace(f), String(change));
    assert(!fs.existsSync(artifact(f, "typechecker-workspace-observations.json")));
  }
  const expectedOnly = controlled(t);
  expectedOnly.prepare();
  expectedOnly.complete();
  assert.throws(
    () => aggregateTypecheckerWorkspace(expectedOnly),
    "expected bodies without completed physical producer packet cannot count",
  );
});

void test("new workspace mode refuses incompatible runner flags before invoking Cargo", () => {
  const runner = path.join(root, "tools/support/compat/github/run-rust-workspace-tests.ts");
  for (const args of [
    ["--runner", "cargo", "--cargo-profile", "ci"],
    ["--cargo-profile", "test"],
    ["--cargo-profile", "ci", "--inventory-only"],
  ]) {
    const result = spawnSync(
      process.execPath,
      [runner, "--capture-typechecker-workspace", ...args],
      { encoding: "utf8", env: { ...process.env, PATH: "" } },
    );
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /workspace observation requires complete nextest ci execution/);
  }
});
