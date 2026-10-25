import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { assertSharedApiWirePackets } from "./support/native-runtime-selection/lsp-shared-api-campaign.mjs";

const wireContract = JSON.parse(fs.readFileSync(new URL("../_fixtures/differential/lsp/shared-api-snapshot-reopen-10730/wire-contract.json", import.meta.url), "utf8"));
const consumer = { workspace: "/outer", configRoot: "/outer/selected" };
const profile = JSON.parse(JSON.stringify({
  phases: Object.entries(wireContract.basic).map(([name, packets]) => ({ name, ...packets })),
  snapshotTransitions: Object.entries(wireContract.transitions).map(([key, packets]) => {
    const match = /^repeated-initial-(\d+)$/u.exec(key);
    return { name: match ? "repeated-initial" : key, ...(match ? { version: Number(match[1]) } : {}), ...packets };
  }),
}).replaceAll("${workspace}", "file:///outer/selected"));

void test("selected config packet root is the exact authored input root, with whole nested packets retained", () => {
  const campaign = { wireContract, packetWorkspace: consumer.configRoot };
  assertSharedApiWirePackets(consumer, campaign, profile);
  assert.throws(() => assertSharedApiWirePackets(consumer, { wireContract }, profile), /whole valid diagnostics packet/u);
  const altered = structuredClone(profile);
  altered.phases[0].diagnostics.params.version = 100;
  assert.throws(() => assertSharedApiWirePackets(consumer, campaign, altered), /whole valid diagnostics packet/u);
});
