import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { prepareSharedApiCampaign } from "./support/native-runtime-selection/lsp-shared-api-campaign.mjs";

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
  } finally {
    fs.rmSync(workspace, { recursive: true, force: true });
  }
});
