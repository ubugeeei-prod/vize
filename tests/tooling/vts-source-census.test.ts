import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { parse } from "yaml";
import {
  assertPacket,
  censusFixture,
  packetDiagnostics,
} from "../../tools/benchmarks/scripts/vts-source-census-fixture.ts";
import {
  VTS_SOURCE_CENSUS_REPETITIONS,
  VTS_SOURCE_CENSUS_SIZES,
  censusProfile,
  compareIdentityMultisets,
  type IdentityEntry,
  type IdentityReceipt,
  type Profile,
} from "../../tools/benchmarks/scripts/vts-source-census-profile.ts";

const digestA = "11".repeat(32),
  digestB = "22".repeat(32),
  receiptSha = "33".repeat(32);
const entry = (sourceType: string, sha256: string, count = 1, byteLength = 4): IdentityEntry => ({
  sourceType,
  byteLength,
  sha256,
  count,
});
const receipt = (producers: IdentityEntry[], consumers: IdentityEntry[]): IdentityReceipt => ({
  schema: "vize.vts-source-identity",
  version: 1,
  algorithm: "sha256",
  domain: "vize.vts-source-census.sha256.v1",
  identity: "canonical SourceType tuple, byte length and domain-separated source digest",
  multiplicity: "count-preserving order-independent multiset",
  producers,
  consumers,
  comparison: compareIdentityMultisets(producers, consumers),
});

void test("tiny census graph retains every feature and exact diagnostic tuple", () => {
  for (const sfcs of VTS_SOURCE_CENSUS_SIZES) {
    const valid = censusFixture(sfcs, false),
      invalid = censusFixture(sfcs, true);
    assert.equal(valid.expected.length, 0);
    assert.equal(invalid.expected.length, sfcs + 5);
    assert.equal(Object.keys(valid.files).filter((file) => file.endsWith(".vue")).length, sfcs);
    assert.ok(Object.keys(valid.files).some((file) => file.endsWith(".tsx")));
    const config = JSON.parse(valid.files["tsconfig.json"]);
    assert.equal(config.compilerOptions.incremental, false);
    assert.equal(config.files.filter((file: string) => file.endsWith(".vue")).length, sfcs);
    for (const diagnostic of invalid.expected) {
      const line = invalid.files[diagnostic.file].split("\n")[diagnostic.line - 1],
        token = line.slice(diagnostic.column - 1);
      assert.ok(
        token.startsWith(diagnostic.file.endsWith(".vue") ? "text: string" : "dependencyCoverage"),
      );
    }
    assert.ok(valid.files["src/App000.vue"].includes("\r\n"));
    assert.equal(
      valid.files["src/graph/barrel.ts"],
      'export { value } from "./dependency-with-a-deliberately-long-module-specifier-for-allocation-census";\nexport type { Model } from "./types";\n',
    );
  }
});

void test("full diagnostic vectors reject hidden duplicates and unclassified entries", () => {
  const expected = censusFixture(1, true).expected;
  assertPacket([...expected].reverse(), expected);
  for (const actual of [[...expected, expected[0]], expected.slice(1)])
    assert.throws(() => assertPacket(actual, expected), /diagnostic multiplicity is mandatory/u);
  assert.throws(
    () =>
      packetDiagnostics(
        {
          errorCount: 1,
          warningCount: 0,
          fileCount: 1,
          files: [{ file: "x.vue", diagnostics: ["unclassified"] }],
        },
        "/fixture",
      ),
    /retain and reject any unclassified diagnostic/u,
  );
});

void test("identity multiset admits an exact match", () => {
  const result = compareIdentityMultisets(
    [entry("language=1;module=1;variant=0;extension=0", digestA)],
    [entry("language=1;module=1;variant=0;extension=0", digestA)],
  );
  assert.deepEqual(
    [result.matchedCalls, result.producerOnlyCalls, result.consumerOnlyCalls, result.matches],
    [1, 0, 0, true],
  );
});

void test("identity multiset preserves duplicate multiplicity", () => {
  const result = compareIdentityMultisets(
    [entry("language=1;module=1;variant=0;extension=0", digestA, 2)],
    [entry("language=1;module=1;variant=0;extension=0", digestA, 3)],
  );
  assert.deepEqual(
    [result.matchedCalls, result.producerOnlyCalls, result.consumerOnlyCalls, result.matches],
    [2, 0, 1, false],
  );
});

void test("identity multiset retains SourceType mismatch as data", () => {
  const result = compareIdentityMultisets(
    [entry("language=1;module=1;variant=0;extension=0", digestA)],
    [entry("language=1;module=1;variant=1;extension=0", digestA)],
  );
  assert.deepEqual(
    [result.matchedCalls, result.producerOnlyCalls, result.consumerOnlyCalls, result.matches],
    [0, 1, 1, false],
  );
});

void test("identity multiset retains byte mismatch as data", () => {
  const result = compareIdentityMultisets(
    [entry("language=1;module=1;variant=0;extension=0", digestA, 1, 4)],
    [entry("language=1;module=1;variant=0;extension=0", digestB, 1, 5)],
  );
  assert.deepEqual(
    [result.matchedCalls, result.producerOnlyCalls, result.consumerOnlyCalls, result.matches],
    [0, 1, 1, false],
  );
});

const counter = (name: string, units: number) => ({
  key: name,
  name,
  file: "test.rs",
  module: "test",
  kind: "counter",
  line: 1,
  calls: 1,
  units,
});
const spanKey = "canon.dependency_scan.content.clone@test.rs:2";
const profile = (uncached = 1): Profile => ({
  schema_version: 1,
  truncation: { dropped_counters: 0, dropped_spans: 0 },
  allocation: { calls: 5, requested_bytes: 100 },
  spans: [
    {
      key: spanKey,
      count: 1,
      wall_ns: {},
      alloc: { calls: 1, bytes: 20, self_calls: 1, self_bytes: 20 },
    },
  ],
  micro: [
    counter("canon.dependency_scan.content.clone.calls", 1),
    counter("canon.dependency_scan.content.clone.source_bytes", 100),
    counter("canon.dependency_scan.content.generated_source_bytes", 90),
    counter("canon.dependency_scan.occurrences.uncached.calls", uncached),
    counter("canon.source_census.producer.calls", 1),
    {
      key: spanKey,
      name: "canon.dependency_scan.content.clone",
      file: "test.rs",
      module: "test",
      kind: "span",
      line: 2,
      calls: 1,
      units: 0,
    },
  ],
});

void test("profile admission requires non-vacuous digest identity but permits mismatch", () => {
  const type = "language=1;module=1;variant=0;extension=0",
    matchedReceipt = receipt([entry(type, digestA)], [entry(type, digestA)]),
    matched = censusProfile(profile(), matchedReceipt, receiptSha, 1);
  assert.equal(matched.sameContentParseIdentity.matches, true);
  assert.match(matched.sameContentParseIdentity.producerMultisetSha256, /^[a-f0-9]{64}$/u);
  const mismatched = censusProfile(
    profile(),
    receipt([entry(type, digestA)], [entry(type, digestB)]),
    receiptSha,
    1,
  );
  assert.equal(mismatched.sameContentParseIdentity.matches, false);
  assert.equal(mismatched.sameContentParseIdentity.consumerOnlyCalls, 1);
  assert.throws(
    () => censusProfile(profile(0), matchedReceipt, receiptSha, 1),
    /cached and uncached dependency occurrence counters are both empty/u,
  );
  assert.throws(
    () => censusProfile(profile(), receipt([entry(type, digestA)], []), receiptSha, 1),
    /identity cannot pass vacuously without consumers/u,
  );
});

type WorkflowStep = {
  name?: string;
  run?: string;
  uses?: string;
  if?: string;
  with?: Record<string, unknown>;
};
type CensusWorkflow = {
  on: {
    workflow_dispatch: { inputs: Record<string, unknown> };
  };
  permissions: Record<string, string>;
  jobs: Record<string, { steps: WorkflowStep[] }>;
};
const parserInstallContract = `set -euo pipefail
parser_link="$GITHUB_WORKSPACE/node_modules/yaml"
parser_target="$RUNNER_TEMP/census-test-runtime/node_modules/yaml"
parser_marker="$RUNNER_TEMP/census-test-runtime/workspace-yaml-link-owned"
mkdir "$RUNNER_TEMP/census-test-runtime"
cd "$RUNNER_TEMP/census-test-runtime"
node -e 'require("node:fs").writeFileSync("package.json",JSON.stringify({private:true,dependencies:{yaml:"2.9.0"}}))'
npm install --ignore-scripts --no-audit --no-fund
test ! -e "$parser_marker"
test ! -L "$parser_marker"
printf 'none\\n' > "$parser_marker"
test "$(cat "$parser_marker")" = "none"
mkdir "$GITHUB_WORKSPACE/node_modules"
printf 'root\\n' > "$parser_marker"
test "$(cat "$parser_marker")" = "root"
test ! -e "$parser_link"
test ! -L "$parser_link"
ln -s "$parser_target" "$parser_link"
test -L "$parser_link"
test "$(readlink "$parser_link")" = "$parser_target"`;
const parserCleanupContract = `set -euo pipefail
parser_link="$GITHUB_WORKSPACE/node_modules/yaml"
parser_target="$RUNNER_TEMP/census-test-runtime/node_modules/yaml"
parser_marker="$RUNNER_TEMP/census-test-runtime/workspace-yaml-link-owned"
test -f "$parser_marker"
test ! -L "$parser_marker"
parser_state=$(cat "$parser_marker")
case "$parser_state" in
  none) ;;
  root)
    if test ! -e "$parser_link" && test ! -L "$parser_link"; then
      :
    else
      test -L "$parser_link"
      test "$(readlink "$parser_link")" = "$parser_target"
      unlink "$parser_link"
    fi
    rmdir "$GITHUB_WORKSPACE/node_modules"
    test ! -e "$GITHUB_WORKSPACE/node_modules"
    test ! -L "$GITHUB_WORKSPACE/node_modules"
    test ! -e "$parser_link"
    test ! -L "$parser_link"
    printf 'none\\n' > "$parser_marker"
    ;;
  *) exit 1 ;;
esac`;
function censusWorkflow(): CensusWorkflow {
  return parse(
    fs.readFileSync(".github/workflows/vts-source-census.yml", "utf8"),
  ) as CensusWorkflow;
}
function workflowUses(value: unknown): string[] {
  if (Array.isArray(value)) return value.flatMap(workflowUses);
  if (!value || typeof value !== "object") return [];
  return Object.entries(value).flatMap(([key, nested]) => {
    if (key !== "uses") return workflowUses(nested);
    assert.equal(typeof nested, "string");
    return [nested as string];
  });
}

void test("workflow is structurally pinned to exact sources and three repetitions per size", () => {
  const workflow = censusWorkflow(),
    rootManifest = fs.readFileSync("crates/vize/Cargo.toml", "utf8");
  assert.deepEqual(workflow.on.workflow_dispatch.inputs, {
    source_sha: {
      description: "Immutable source commit to check out and measure",
      type: "string",
      required: true,
    },
    source_parent: {
      description: "Immutable immediate production parent",
      type: "string",
      required: true,
    },
  });
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  assert.deepEqual(VTS_SOURCE_CENSUS_SIZES, [1, 10, 100]);
  assert.equal(VTS_SOURCE_CENSUS_REPETITIONS, 3);
  assert.match(rootManifest, /"vize_canon\/profiling"/u);

  const uses = workflowUses(workflow),
    steps = Object.values(workflow.jobs).flatMap((job) => job.steps),
    actionSteps = steps.filter((step) => step.uses !== undefined);
  const parserInstall = steps.find(
      (step) => step.name === "Install only the pinned workflow parser",
    ),
    sourcePreflight = steps.find(
      (step) => step.name === "Verify exact source, observer laws and production projection",
    ),
    parserCleanup = steps.find((step) => step.name === "Remove the workflow parser link"),
    providerInstall = steps.find(
      (step) => step.name === "Install only the pinned public Vue checker and schema provider",
    ),
    measure = steps.find(
      (step) => step.name === "Build one fresh profiling CLI and measure three paired repetitions",
    ),
    profilingLaws = steps.find(
      (step) => step.name === "Run profiling-only identity and custody laws in the built target",
    );
  assert.ok(
    parserInstall &&
      sourcePreflight &&
      parserCleanup &&
      providerInstall &&
      measure &&
      profilingLaws,
  );
  assert.ok(steps.indexOf(parserInstall) < steps.indexOf(sourcePreflight));
  assert.ok(steps.indexOf(sourcePreflight) < steps.indexOf(parserCleanup));
  assert.ok(steps.indexOf(parserCleanup) < steps.indexOf(providerInstall));
  const nodeTest = sourcePreflight.run?.indexOf("node --test") ?? -1,
    sourceProjection = sourcePreflight.run?.indexOf("vts-source-census-preflight.ts") ?? -1;
  assert.ok(nodeTest >= 0 && nodeTest < sourceProjection);
  assert.match(sourcePreflight.run ?? "", /git fetch --no-tags --depth=1 origin "\$driver_sha"/u);
  assert.match(
    sourcePreflight.run ?? "",
    /vts-source-census-preflight\.ts "\$SOURCE_PARENT" "\$SOURCE_SHA" "\$driver_sha"/u,
  );
  assert.match(measure.run ?? "", /--driver-sha "\$GITHUB_WORKFLOW_SHA"/u);
  assert.match(
    profilingLaws.run ?? "",
    /cargo test --locked -p vize --features profiling --lib[\s\\]+commands::profile_export::tests::aliased_receipt_creation_fails_after_profile_write -- --exact/u,
  );
  assert.equal(parserInstall.run?.trim(), parserInstallContract);
  assert.equal(parserCleanup.if, "always()");
  assert.equal(parserCleanup.run?.trim(), parserCleanupContract);
  assert.ok(
    parserCleanupContract.indexOf('readlink "$parser_link"') <
      parserCleanupContract.indexOf('unlink "$parser_link"'),
  );
  assert.doesNotMatch(parserCleanupContract, /(?:^|\s)rm\s+-[^\n]*r/u);
  assert.ok(uses.length > 0);
  for (const action of uses) assert.match(action, /^[^@\s]+@[a-f0-9]{40}$/u);
  const uploads = actionSteps.filter((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.equal(uploads.length, 2);
  assert.deepEqual(
    uploads.map((upload) => upload.with?.name),
    [
      "vts-source-census-small-${{ github.run_id }}-${{ github.run_attempt }}",
      "vts-source-census-raw-${{ github.run_id }}-${{ github.run_attempt }}",
    ],
  );
  for (const upload of uploads) {
    assert.equal(upload.if, "always()");
    assert.equal(upload.with?.["if-no-files-found"], "error");
  }
});

void test("workflow parser workspace custody is fail-closed and non-destructive", (t) => {
  const run = (script: string, root: string) =>
    spawnSync("bash", ["-euo", "pipefail", "-c", script], {
      encoding: "utf8",
      env: { ...process.env, GITHUB_WORKSPACE: root, RUNNER_TEMP: root },
    });
  const root = () => {
    const value = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vts-parser-custody-"));
    t.after(() => fs.rmSync(value, { recursive: true, force: true }));
    fs.mkdirSync(path.join(value, "census-test-runtime"));
    return value;
  };
  const marker = (value: string) =>
    path.join(value, "census-test-runtime", "workspace-yaml-link-owned");
  const mutation = [
    "set -euo pipefail",
    'parser_link="$GITHUB_WORKSPACE/node_modules/yaml"',
    'parser_target="$RUNNER_TEMP/census-test-runtime/node_modules/yaml"',
    'parser_marker="$RUNNER_TEMP/census-test-runtime/workspace-yaml-link-owned"',
    parserInstallContract.slice(parserInstallContract.indexOf('mkdir "$GITHUB_WORKSPACE')),
  ].join("\n");
  const rootReceipt = (value: string) => {
    const candidate = path.join(value, "node_modules"),
      stat = fs.lstatSync(candidate);
    return stat.isSymbolicLink()
      ? { kind: "link", target: fs.readlinkSync(candidate) }
      : { kind: "directory", entries: fs.readdirSync(candidate).toSorted() };
  };

  for (const kind of ["empty", "nonempty", "symlink", "dangling"] as const) {
    const value = root(),
      workspaceRoot = path.join(value, "node_modules"),
      foreign = path.join(value, kind === "dangling" ? "missing-root" : "foreign-root");
    fs.writeFileSync(marker(value), "none\n");
    if (kind === "symlink" || kind === "dangling") {
      if (kind === "symlink") fs.mkdirSync(foreign);
      fs.symlinkSync(foreign, workspaceRoot);
    } else {
      fs.mkdirSync(workspaceRoot);
      if (kind === "nonempty") fs.writeFileSync(path.join(workspaceRoot, "foreign"), "retain");
    }
    const before = rootReceipt(value),
      install = run(mutation, value);
    assert.notEqual(install.status, 0, `${kind} root must reject ownership`);
    assert.equal(run(parserCleanupContract, value).status, 0);
    assert.deepEqual(rootReceipt(value), before);
    assert.equal(fs.readFileSync(marker(value), "utf8"), "none\n");
  }

  const owned = root(),
    workspaceRoot = path.join(owned, "node_modules"),
    link = path.join(workspaceRoot, "yaml"),
    target = path.join(owned, "census-test-runtime", "node_modules", "yaml");
  fs.mkdirSync(target, { recursive: true });
  fs.writeFileSync(marker(owned), "none\n");
  assert.equal(run(mutation, owned).status, 0);
  assert.equal(fs.readFileSync(marker(owned), "utf8"), "root\n");
  assert.equal(fs.readlinkSync(link), target);
  assert.equal(run(parserCleanupContract, owned).status, 0);
  assert.equal(fs.existsSync(workspaceRoot), false);
  assert.equal(fs.lstatSync(marker(owned)).isSymbolicLink(), false);
  assert.equal(fs.readFileSync(marker(owned), "utf8"), "none\n");

  for (const kind of ["wrong-link", "regular-file"] as const) {
    const value = root(),
      ownedRoot = path.join(value, "node_modules"),
      yaml = path.join(ownedRoot, "yaml");
    fs.mkdirSync(ownedRoot);
    fs.writeFileSync(marker(value), "root\n");
    if (kind === "wrong-link") fs.symlinkSync(path.join(value, "foreign-yaml"), yaml);
    else fs.writeFileSync(yaml, "retain");
    assert.notEqual(run(parserCleanupContract, value).status, 0);
    assert.equal(
      kind === "wrong-link" ? fs.readlinkSync(yaml) : fs.readFileSync(yaml, "utf8"),
      kind === "wrong-link" ? path.join(value, "foreign-yaml") : "retain",
    );
    assert.equal(fs.readFileSync(marker(value), "utf8"), "root\n");
  }

  const sibling = root(),
    siblingRoot = path.join(sibling, "node_modules"),
    siblingLink = path.join(siblingRoot, "yaml"),
    siblingTarget = path.join(sibling, "census-test-runtime", "node_modules", "yaml");
  fs.mkdirSync(siblingTarget, { recursive: true });
  fs.mkdirSync(siblingRoot);
  fs.symlinkSync(siblingTarget, siblingLink);
  fs.writeFileSync(path.join(siblingRoot, "foreign"), "retain");
  fs.writeFileSync(marker(sibling), "root\n");
  assert.notEqual(run(parserCleanupContract, sibling).status, 0);
  assert.equal(fs.existsSync(siblingLink), false);
  assert.equal(fs.readFileSync(path.join(siblingRoot, "foreign"), "utf8"), "retain");
  assert.equal(fs.readFileSync(marker(sibling), "utf8"), "root\n");
});
