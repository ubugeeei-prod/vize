// Installed CLI consumer of the same frozen semantic contract; no source-build identity.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { installedCliRuntime } from "./n8n-installed-cli.ts";
import type {
  SemanticFixture,
  SemanticCliCapture,
  Projection,
  SemanticCapture,
} from "./n8n-installed-contract-types.ts";
import {
  root,
  projection as frozenProjection,
  validateProjection,
} from "./n8n-cli-config-inputs.mjs";
import { errorPacket } from "./n8n-cli-config-oracle.mjs";
import {
  loadSemanticCases,
  validateSemanticCases,
  captureSemanticOracles as legacyCaptureSemanticOracles,
  assertSemanticOracles,
} from "./n8n-cli-config-semantic-oracle.mjs";
import { assertUnchanged, writeConfig, writeJson, sha256 } from "./n8n-cli-config-workspace.mjs";

assert.ok(process.argv[2] && process.argv[3], "new output and reviewed installed plan required");
const output = path.resolve(process.argv[2]);
const projection = frozenProjection as Projection;
// JS inference misses provider fields assigned after capture construction. The original whole oracle still validates the result.
const captureSemanticOracles = legacyCaptureSemanticOracles as unknown as SemanticCapture;
validateProjection();
const fixture = loadSemanticCases() as SemanticFixture;
validateSemanticCases(fixture);
const producer = installedCliRuntime(output, process.argv[3]);
const { identity, binary, installConfigPackages, runCli } = producer;
const workspace = producer.workspace("workspace");
const publicPackageFiles = installConfigPackages(workspace);
writeJson(path.join(output, "public-config-package.json"), publicPackageFiles);
const packageRoot = "owned";
fs.mkdirSync(path.join(workspace, packageRoot), { recursive: true });
const inputs = fixture.cases.map(({ id, source, sourceSha256 }) => {
  const file = path.join(packageRoot, id + ".vue");
  fs.writeFileSync(path.join(workspace, file), source);
  return { file, sha256: sourceSha256 };
});
writeJson(path.join(output, "inputs.json"), inputs);

const oracles = await captureSemanticOracles({
  onRecorded(packet) {
    writeJson(path.join(output, "provider", `${packet.caseId}-${packet.iteration}.json`), packet);
  },
});
writeJson(path.join(output, "provider/complete.json"), oracles);

const captures = [];
const configurations = [];
for (const sourceCase of fixture.cases) {
  const config = writeConfig(workspace, packageRoot, sourceCase.id, {
    ...projection.linter.ruleOptions,
    ...sourceCase.ruleOptions,
  });
  configurations.push({ file: path.relative(workspace, config.path), sha256: config.sha256 });
  const invoke = (mode: string, outsideCwd = false) => {
    try {
      return runCli({
        binary,
        workspace,
        packageRoot,
        files: [sourceCase.id + ".vue"],
        config,
        outsideCwd,
        receiptPath: path.join(output, "cli", sourceCase.id, mode + ".json"),
      });
    } catch (error) {
      // runCli already retains complete process/parse evidence. Collect the
      // other inputs too, then refuse this failure without normalizing it.
      return { failure: errorPacket(error) };
    }
  };
  captures.push({
    caseId: sourceCase.id,
    baseline: invoke("baseline"),
    repeat: invoke("repeat"),
    outside: invoke("outside-cwd", true),
  });
}
// Keep all actual CLI/provider responses before a whole-envelope law can fail.
writeJson(path.join(output, "cli/complete.json"), captures);
writeJson(path.join(output, "configurations.json"), configurations);
assertSemanticOracles(oracles);
for (const sourceCase of fixture.cases) {
  const actual = captures.find(({ caseId }) => caseId === sourceCase.id) as SemanticCliCapture;
  for (const mode of ["baseline", "repeat", "outside"] as const) {
    assert.ok(
      Array.isArray(actual[mode]),
      sourceCase.id + "/" + mode + ": complete process failure retained",
    );
  }
  const provider = oracles.find(({ caseId }) => caseId === sourceCase.id)!.observations[0];
  const inverse = Object.fromEntries(
    Object.entries(projection.oracleRules).map(([rule, oracle]) => [oracle, rule]),
  );
  const independentIds = provider.recorded.captures.flatMap(({ packets }) =>
    packets.flatMap(({ messages }) =>
      messages.map(({ ruleId }) => {
        assert.ok(inverse[ruleId], sourceCase.id + ": unexpected whole-provider finding " + ruleId);
        return inverse[ruleId];
      }),
    ),
  );
  // Named derived identity multiset only: retain each provider's complete,
  // ordered packet above and compare the entire native envelope below. The
  // providers' different child/tag ranges can legitimately order reports apart.
  assert.deepEqual(
    actual.baseline
      .flatMap(({ messages }) => messages.map(({ ruleId }) => ruleId))
      .sort((a, b) => a.localeCompare(b)),
    independentIds.sort((a, b) => a.localeCompare(b)),
    sourceCase.id + ": every independent finding reaches the CLI",
  );
  assert.deepEqual(
    actual.baseline,
    sourceCase.cliExpectations,
    sourceCase.id + ": whole actual CLI envelope",
  );
  assert.deepEqual(actual.repeat, actual.baseline, sourceCase.id + ": complete repeat");
  const outside = actual.outside.map((packet) => ({
    ...packet,
    file: packet.file.startsWith(packageRoot + "/")
      ? packet.file.slice(packageRoot.length + 1)
      : packet.file,
  }));
  assert.deepEqual(outside, actual.baseline, sourceCase.id + ": outside-cwd whole envelope");
}
assertUnchanged(workspace, [...inputs, ...publicPackageFiles, ...configurations]);
producer.recheck();
writeJson(path.join(output, "summary.json"), {
  schema: fixture.schema,
  source: identity,
  projectionSha256: sha256(
    fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json")),
  ),
  cases: fixture.cases.length,
  scriptless: fixture.cases.filter(({ scriptless }) => scriptless).length,
  providerObservations: oracles.reduce((n, { observations }) => n + observations.length, 0),
  cliInvocations: captures.length * 3,
  qualification: fixture.qualification,
});
console.log(
  "Installed CLI semantic routing passed: 28 cases, 56 whole provider observations, 84 actual CLI invocations",
);
