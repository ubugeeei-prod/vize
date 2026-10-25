import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  prepareSharedApiCampaign,
  assertSharedApiWirePackets,
} from "./support/native-runtime-selection/lsp-shared-api-campaign.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

void test("shared snapshot campaign uses the ordinary pinned language fixture and explicit transitions", () => {
  const workspace = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-shared-api-fixture-")),
  );
  try {
    const inputPaths = ["src/App.vue", "src/support.ts", "tsconfig.json", "vite.config.mjs"].map(
      (file) => path.join(workspace, file),
    );
    for (const file of inputPaths) {
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, "");
    }
    const consumer = {
      kind: "canonical-702",
      workspace,
      configRoot: workspace,
      entry: inputPaths[0],
      inputPaths,
    };
    const campaign = prepareSharedApiCampaign(repoRoot, consumer);
    assert.equal(campaign.custody.id, "lsp/regression/shared-api-snapshot-reopen-10730");
    assert.equal(campaign.custody.inputFiles.length, 4);
    assert.ok(campaign.sources.valid.includes("const doubled: number = twice(value);"));
    assert.equal(campaign.sources.repair, campaign.sources.valid);
    assert.ok(campaign.sources.error.includes('const value: number = "wrong";'));
    assert.equal(
      campaign.transitions.dependencyError,
      "export const twice = (value: number): string => String(value);\n",
    );
    assert.throws(() =>
      prepareSharedApiCampaign(repoRoot, { ...consumer, kind: "canonical-beta" }),
    );
    assert.throws(() => prepareSharedApiCampaign(repoRoot, { ...consumer, inputPaths: [] }));
    const materialize = (value) =>
      JSON.parse(JSON.stringify(value).replaceAll("${workspace}", pathToFileURL(workspace).href));
    const profile = {
      phases: Object.entries(campaign.wireContract.basic).map(([name, packets]) => ({
        name,
        ...materialize(packets),
      })),
      snapshotTransitions: Object.entries(campaign.wireContract.transitions).map(
        ([key, packets]) => ({
          name: key.startsWith("repeated-initial-") ? "repeated-initial" : key,
          ...(key.startsWith("repeated-initial-") ? { version: Number(key.at(-1)) } : {}),
          ...materialize(packets),
        }),
      ),
    };
    assertSharedApiWirePackets(consumer, campaign, profile);
    for (const mutate of [
      (p) => p.phases.pop(),
      (p) => p.snapshotTransitions.shift(),
      (p) => {
        p.phases[3].diagnostics.params.extraOpaque = true;
      },
      (p) => {
        p.phases[1].diagnostics.params.diagnostics[0].range.start.character++;
      },
      (p) => {
        p.phases[0].value.id++;
      },
      (p) => {
        p.phases[0].doubled.result.extraOpaque = true;
      },
      (p) => {
        p.snapshotTransitions[1].importer.params.uri = "file:///foreign/App.vue";
      },
      (p) => {
        p.snapshotTransitions[3].importer.params.version = 5;
      },
    ]) {
      const changed = structuredClone(profile);
      mutate(changed);
      assert.throws(() => assertSharedApiWirePackets(consumer, campaign, changed));
    }
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
