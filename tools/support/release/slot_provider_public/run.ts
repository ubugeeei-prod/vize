import assert from "node:assert/strict";
import { existsSync, mkdirSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { Recorder } from "../../config/windows_consumer/common.ts";
import { assertCustody, assertDeclarationCustody, inside } from "../computed_event_custody.ts";
import { authenticate, publicVersion } from "./authority.ts";
import { loadFixtures } from "./fixtures.ts";
import { install } from "./install.ts";
import { observe, writeProject } from "./observe.ts";

assert.equal(process.env.GITHUB_ACTIONS, "true", "public products run only in Actions");
assert.equal(process.env.GITHUB_REF, "refs/heads/main", "merged main workflow required");
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
assert.equal(process.version, "v24.14.0");
for (const [name, value] of Object.entries(process.env))
  if (
    /^(?:VIZE_|CORSA_|TSGO_|TSRS_|NAPI_RS_)/u.test(name) ||
    ["NODE_OPTIONS", "NODE_PATH", "CARGO_BIN_EXE_vize"].includes(name)
  )
    assert.ok(!value, "initial runtime override forbidden: " + name);
const [version, releaseRun, blob, fixtureRoot, output] = process.argv.slice(2);
assert.ok(version && releaseRun && blob && fixtureRoot && output);
assert.equal(version, publicVersion);
assert.ok(process.env.RUNNER_TEMP);
assert.ok(inside(realpathSync(process.env.RUNNER_TEMP), output));
assert.equal(existsSync(output), false, "preserve earlier observations");
mkdirSync(output);
const record = new Recorder(realpathSync(output));
const result: Record<string, unknown> = {
  schema: "vize.slotProvider.installedPublic.v1",
  status: "RUNNING",
  accepted: false,
  version,
  releaseRun,
  authorityBlob: blob,
  consumerHead: process.env.GITHUB_SHA,
  consumerRun: process.env.GITHUB_RUN_ID,
  consumerAttempt: process.env.GITHUB_RUN_ATTEMPT,
  node: process.version,
  scope:
    "frozen original10 separately from imported7/registered15/collision16, with labelled repairs",
  capsuleTrust:
    "release owner supplies immutable capsule after genuine whole producer ZIP authentication",
  sourceSdkUsed: false,
  umbrella11027Closed: false,
  selfDynamicQualified: false,
  NoAny10Qualified: false,
  arbitrary9_17CorrelationQualified: false,
  timingUsable: false,
};
record.json("qualification.json", result);
try {
  const fixtures = loadFixtures(fixtureRoot);
  record.json("fixture-custody.json", fixtures.custody);
  record.json(
    "fixture-manifest.json",
    fixtures.stages.map((stage) => ({
      population: stage.population,
      id: stage.id,
      expected: stage.expected,
      repairCopy: stage.repairCopy,
      files: [...stage.files].map(([file, utf8]) => ({ file, utf8 })),
    })),
  );
  const authority = await authenticate(version, releaseRun, blob, record);
  result.producer = authority.ownership;
  const installed = await install(
    join(output, "registry-installation"),
    version,
    authority,
    record,
  );
  const populations: Record<string, ReturnType<typeof observe>[]> = {
    original10: [],
    imported7: [],
    registered15: [],
    collision16: [],
  };
  result.populations = populations;
  for (const stage of fixtures.stages) {
    result.activeStage = stage.population + "/" + stage.id;
    record.json("qualification.json", result);
    const project = join(output, stage.population, stage.id);
    assert.ok(inside(output, project));
    writeProject(project, stage, installed.modules);
    populations[stage.population].push(
      observe(record, project, stage, installed, fixtures.helperText),
    );
    record.json("qualification.json", result);
  }
  assert.deepEqual(
    Object.values(populations).map((rows) => rows.length),
    [10, 7, 15, 16],
  );
  assertCustody(fixtures.custody);
  assertCustody(installed.images);
  assertDeclarationCustody(installed.cliRoot, installed.graph.files, installed.graph.edges);
  result.status = "SCOPED_PUBLIC_SLOT_PROVIDER_VECTORS_ACCEPTED";
  result.accepted = true;
  result.original10 = { stages: 10, semanticProcesses: 30, directTransactions: 10 };
  result.separateOwnerPopulation = {
    imported: 7,
    registered: 15,
    collision: 16,
    stages: 38,
    semanticProcesses: 114,
    directTransactions: 38,
  };
} catch (error) {
  result.status = "FAILED";
  result.error = error instanceof Error ? error.message : String(error);
  throw error;
} finally {
  record.json("qualification.json", result);
}
