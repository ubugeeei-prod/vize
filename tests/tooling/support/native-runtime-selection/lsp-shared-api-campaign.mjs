import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { loadLspManifest } from "../../../differential/lsp-manifest.ts";
import { readPinnedArtifact } from "../../../differential/harness.mjs";
import { captureConsumerInputs, sha256 } from "./providers.mjs";

const ID = "lsp/regression/shared-api-snapshot-reopen-10730";

export function prepareSharedApiCampaign(repoRoot, consumer) {
  assert.ok(["canonical-702", "alias-702"].includes(consumer.kind));
  return prepareCampaignInputs(repoRoot, consumer);
}

/** Reuse authored packets only; beta provider/view qualification is independent. */
export function prepareNativeBetaDtoCampaign(repoRoot, consumer) {
  assert.equal(consumer.kind, "selected-config");
  return prepareCampaignInputs(repoRoot, consumer);
}

function prepareCampaignInputs(repoRoot, consumer) {
  const pack = path.join(repoRoot, "tests/_fixtures/differential/lsp");
  const loaded = loadLspManifest(path.join(pack, "manifest.json"));
  const fixture = loaded.cases.find((fixture) => fixture.id === ID);
  assert.ok(fixture, "ordinary pinned LSP fixture is required for general snapshot code");
  const rawCase = JSON.parse(readPinnedArtifact(pack, fixture.case).toString("utf8"));
  const transitionBytes = readPinnedArtifact(
    path.join(pack, fixture.inputs.root),
    rawCase.provenance.transitions,
  );
  const transitions = JSON.parse(transitionBytes.toString("utf8"));
  const wireBytes = readPinnedArtifact(
    path.join(pack, fixture.inputs.root),
    rawCase.provenance.wireContract,
  );
  const wireContract = JSON.parse(wireBytes.toString("utf8"));
  assert.equal(wireContract.version, 1);
  assert.equal(wireContract.id, "lsp/shared-editor-initial-snapshot-10730");
  assert.equal(transitions.version, 1);
  for (const file of fixture.files) {
    const target = path.join(consumer.configRoot, file.runtimePath);
    assert.ok(
      consumer.inputPaths.includes(target),
      "campaign changes only its declared fixture inputs",
    );
    fs.writeFileSync(target, file.bytes);
  }
  assert.equal(transitions.sourceValid, fs.readFileSync(consumer.entry, "utf8"));
  assert.equal(
    transitions.dependencyValid,
    fs.readFileSync(path.join(consumer.configRoot, "src/support.ts"), "utf8"),
  );
  assert.equal(
    transitions.sourceError,
    transitions.sourceValid.replace("const value: number = 2;", 'const value: number = "wrong";'),
  );
  assert.equal(
    transitions.dependencyError,
    "export const twice = (value: number): string => String(value);\n",
  );
  return {
    sources: {
      valid: transitions.sourceValid,
      error: transitions.sourceError,
      repair: transitions.sourceValid,
      support: transitions.dependencyValid,
    },
    transitions,
    wireContract,
    custody: {
      id: fixture.id,
      manifestSha256: loaded.manifestSha256,
      case: fixture.case,
      transitionsSha256: sha256(transitionBytes),
      wireContractSha256: sha256(wireBytes),
      inputFiles: captureConsumerInputs(consumer),
      manifestScope: rawCase.scope,
    },
  };
}

function diagnosticPacket(uri, version, diagnostics) {
  return {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri, version, diagnostics },
  };
}
function assignmentError(source, token) {
  const prefix = source.slice(0, source.indexOf(`const ${token}`) + 6).split("\n");
  const start = { line: prefix.length - 1, character: prefix.at(-1).length };
  return {
    code: 2322,
    message: "Type 'string' is not assignable to type 'number'.",
    range: { start, end: { line: start.line, character: start.character + token.length } },
    severity: 1,
    source: "vize/types",
  };
}
async function diagnostics(wire, uri, version, expected, from, name) {
  const packet = await wire.wait(
    (packet) =>
      packet.method === "textDocument/publishDiagnostics" &&
      packet.params?.uri === uri &&
      packet.params?.version === version,
    name,
    from,
  );
  assert.deepEqual(packet, diagnosticPacket(uri, version, expected), name);
  return packet;
}

/** Complete authored dependency and repeated-initial controls; no disk edits. */
export async function observeSharedApiTransitions({
  wire,
  consumer,
  campaign,
  profile,
  exactHover,
  navigation,
}) {
  const uri = pathToFileURL(consumer.entry).href;
  const dependencyUri = pathToFileURL(path.join(consumer.configRoot, "src/support.ts")).href;
  const record = async (name, operation, appDiagnostics, dependencyVersion) => {
    const from = wire.events.length;
    operation();
    const phase = {
      name,
      authoredDependencySha256: sha256(
        Buffer.from(
          dependencyVersion === 2
            ? campaign.transitions.dependencyError
            : campaign.transitions.dependencyValid,
        ),
      ),
    };
    if (dependencyVersion)
      phase.dependency = await diagnostics(
        wire,
        dependencyUri,
        dependencyVersion,
        [],
        from,
        `${name} complete dependency diagnostics`,
      );
    phase.importer = await diagnostics(
      wire,
      uri,
      4,
      appDiagnostics,
      from,
      `${name} complete importer diagnostics`,
    );
    assert.deepEqual(
      captureConsumerInputs(consumer),
      campaign.custody.inputFiles,
      "unsaved dependency controls preserve every authored disk byte",
    );
    if (navigation && dependencyVersion)
      phase.navigation = await navigation(
        wire,
        consumer,
        campaign,
        dependencyVersion === 2
          ? campaign.transitions.dependencyError
          : campaign.transitions.dependencyValid,
      );
    profile.snapshotTransitions.push(phase);
  };
  profile.snapshotTransitions = [];
  await record(
    "dependency-open",
    () =>
      wire.notify("textDocument/didOpen", {
        textDocument: {
          uri: dependencyUri,
          languageId: "typescript",
          version: 1,
          text: campaign.transitions.dependencyValid,
        },
      }),
    [],
    1,
  );
  await record(
    "dependency-error",
    () =>
      wire.notify("textDocument/didChange", {
        textDocument: { uri: dependencyUri, version: 2 },
        contentChanges: [{ text: campaign.transitions.dependencyError }],
      }),
    [assignmentError(campaign.sources.valid, "doubled")],
    2,
  );
  await record(
    "dependency-repair",
    () =>
      wire.notify("textDocument/didChange", {
        textDocument: { uri: dependencyUri, version: 3 },
        contentChanges: [{ text: campaign.transitions.dependencyValid }],
      }),
    [],
    3,
  );
  await record(
    "dependency-close",
    () => wire.notify("textDocument/didClose", { textDocument: { uri: dependencyUri } }),
    [],
  );
  for (const version of [5, 6]) {
    const from = wire.events.length;
    wire.notify("textDocument/didClose", { textDocument: { uri } });
    wire.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version, text: campaign.sources.valid },
    });
    const phase = {
      name: "repeated-initial",
      version,
      diagnostics: await diagnostics(
        wire,
        uri,
        version,
        [],
        from,
        `complete repeated initial ${version}`,
      ),
    };
    for (const token of ["value", "doubled"]) {
      const offset =
        token === "value"
          ? campaign.sources.valid.indexOf("const value") + 6
          : campaign.sources.valid.indexOf("{{ doubled }}") + 3;
      const prefix = campaign.sources.valid.slice(0, offset + 1).split("\n");
      phase[token] = await wire.request("textDocument/hover", {
        textDocument: { uri },
        position: { line: prefix.length - 1, character: prefix.at(-1).length },
      });
      exactHover(phase[token], campaign.sources.valid, token);
    }
    assert.deepEqual(captureConsumerInputs(consumer), campaign.custody.inputFiles);
    profile.snapshotTransitions.push(phase);
  }
  assertSharedApiWirePackets(consumer, campaign, profile);
}

export function assertSharedApiWirePackets(consumer, campaign, profile) {
  assert.deepEqual(
    profile.phases.map((phase) => phase.name),
    Object.keys(campaign.wireContract.basic),
    "every complete original phase is required",
  );
  const transitionKey = (phase) =>
    phase.name === "repeated-initial" ? `${phase.name}-${phase.version}` : phase.name;
  assert.deepEqual(
    profile.snapshotTransitions.map(transitionKey),
    Object.keys(campaign.wireContract.transitions),
    "every dependency and repeated-initial transition is required",
  );
  const workspace = pathToFileURL(consumer.workspace).href;
  const materialize = (value) => {
    if (typeof value === "string") return value.replaceAll("${workspace}", workspace);
    if (Array.isArray(value)) return value.map(materialize);
    if (value && typeof value === "object")
      return Object.fromEntries(
        Object.entries(value).map(([key, item]) => [key, materialize(item)]),
      );
    return value;
  };
  for (const phase of profile.phases) {
    const expected = materialize(campaign.wireContract.basic[phase.name]);
    assert.ok(expected, `registered whole packet phase: ${phase.name}`);
    for (const [key, packet] of Object.entries(expected))
      assert.deepEqual(phase[key], packet, `whole ${phase.name} ${key} packet`);
  }
  for (const phase of profile.snapshotTransitions) {
    const key = transitionKey(phase);
    const expected = materialize(campaign.wireContract.transitions[key]);
    assert.ok(expected, `registered whole transition: ${key}`);
    for (const [field, packet] of Object.entries(expected))
      assert.deepEqual(phase[field], packet, `whole ${key} ${field} packet`);
  }
}
