import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { loadLspManifest } from "../../../differential/lsp-manifest.ts";
import { readPinnedArtifact } from "../../../differential/harness.mjs";
import { captureConsumerInputs, sha256 } from "./providers.mjs";

export function prepareOwnedShutdownCampaign(repoRoot, consumer) {
  assert.ok(["canonical-702", "alias-702"].includes(consumer.kind));
  const pack = path.join(repoRoot, "tests/_fixtures/differential/lsp");
  const loaded = loadLspManifest(path.join(pack, "manifest.json"));
  const fixture = loaded.cases.find(
    (entry) => entry.id === "lsp/regression/owned-outer-shutdown-10760",
  );
  assert.ok(fixture, "ordinary pinned source-inspected LSP case is mandatory");
  const raw = JSON.parse(readPinnedArtifact(pack, fixture.case).toString("utf8"));
  const fixtureRoot = path.join(pack, fixture.inputs.root);
  const wireBytes = readPinnedArtifact(fixtureRoot, raw.provenance.wireContract);
  const wireContract = JSON.parse(wireBytes.toString("utf8"));
  assert.equal(wireContract.version, 1);
  assert.equal(wireContract.id, "lsp/owned-outer-shutdown-10760");
  const initializeBytes = readPinnedArtifact(fixtureRoot, wireContract.initializeRequest);
  assert.equal(
    sha256(fs.readFileSync(path.join(repoRoot, wireContract.initializerSource.path))),
    wireContract.initializerSource.sha256,
    "actual primary initializer bytes remain pinned",
  );
  for (const file of fixture.files) {
    const target = path.join(consumer.configRoot, file.runtimePath);
    assert.ok(consumer.inputPaths.includes(target), "only declared authored inputs may change");
    fs.writeFileSync(target, file.bytes);
  }
  const valid = fs.readFileSync(consumer.entry, "utf8");
  assert.equal(valid.split("const value: number = 2;").length, 2);
  return {
    sources: {
      valid,
      error: valid.replace("const value: number = 2;", 'const value: number = "wrong";'),
      repair: valid,
      support: fs.readFileSync(path.join(consumer.configRoot, "src/support.ts"), "utf8"),
    },
    wireContract,
    initializeRequest: JSON.parse(initializeBytes.toString("utf8")),
    custody: {
      id: fixture.id,
      manifestSha256: loaded.manifestSha256,
      case: fixture.case,
      wireContractSha256: sha256(wireBytes),
      initializeRequestSha256: sha256(initializeBytes),
      inputFiles: captureConsumerInputs(consumer),
      sourceInspectedScope: raw.scope,
      noExecutionCreditFromMetadataAlone: true,
    },
  };
}

export function assertOwnedShutdownPackets(consumer, campaign, profile) {
  assert.deepEqual(
    profile.phases.map((phase) => phase.name),
    Object.keys(campaign.wireContract.basic),
  );
  const workspace = pathToFileURL(consumer.workspace).href;
  const relocate = (value) => {
    if (typeof value === "string") return value.replaceAll("${workspace}", workspace);
    if (Array.isArray(value)) return value.map(relocate);
    if (value && typeof value === "object")
      return Object.fromEntries(
        Object.entries(value).map(([key, child]) => [key, relocate(child)]),
      );
    return value;
  };
  for (const phase of profile.phases) {
    const expected = relocate(campaign.wireContract.basic[phase.name]);
    for (const [field, packet] of Object.entries(expected))
      assert.deepEqual(phase[field], packet, `whole owned-shutdown ${phase.name} ${field} packet`);
  }
}
