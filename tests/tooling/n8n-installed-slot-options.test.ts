// Transport and early-refusal laws only; public product acceptance runs separately.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { decodeFrames, LspWire } from "../differential/lsp-wire.ts";
import { sha256, type InstalledCampaignPlan } from "./support/n8n-installed-authority.ts";
import { installedLspRuntime } from "./support/n8n-installed-lsp.ts";
import { runtimeOverrides } from "../../tools/support/release/public_acceptance/installed.ts";

test("existing whole-wire transport forwards explicit child env without changing its parent", async () => {
  const key = "VIZE_WIRE_ENV_PROBE";
  const parent = process.env[key];
  const child = `const message = JSON.stringify({jsonrpc:"2.0",method:"ready",params:{probe:process.env.${key}}}); process.stdout.write("Content-Length: " + Buffer.byteLength(message) + "\\r\\n\\r\\n" + message);`;
  const wire = new LspWire(process.execPath, ["-e", child], process.cwd(), {
    ...process.env,
    [key]: "owned-child-value",
  });
  try {
    const packet = { jsonrpc: "2.0", method: "ready", params: { probe: "owned-child-value" } };
    assert.deepEqual(await wire.waitFor((message) => message.method === "ready"), packet);
    await wire.stop(false);
    assert.equal(wire.exitStatus, 0);
    const raw = Buffer.from(wire.observation().serverWireBase64, "base64");
    assert.deepEqual(decodeFrames(raw).messages, [packet]);
    assert.equal(process.env[key], parent);
  } finally {
    await wire.stop();
  }
});

test("installed LSP refuses changed receipt and source tuple before creating output or starting a provider", (t) => {
  const ambient = runtimeOverrides.map((key) => [key, process.env[key]] as const);
  const temporary = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "n8n-public-lsp-refusal-")),
  );
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  try {
    for (const key of runtimeOverrides) delete process.env[key];
    const receiptPath = path.join(temporary, "inert-receipt.json");
    const planPath = path.join(temporary, "inert-plan.json");
    const output = path.join(temporary, "must-not-exist");
    const plan: InstalledCampaignPlan = {
      schema: "vize.n8n.installed-campaign-v1",
      source: {
        C: "26031a4fbb30a1e86511919bafc7035b08440ef3",
        H: "a26243855bcf81005049252a6e71b6dd55e457f1",
        tag: "v0.439.0",
        R: "38011924629",
        sourcePr: "8351",
      },
      installReceipt: { path: receiptPath, sha256: "0".repeat(64) },
      collectorSha256: "a26411442a3fde2590d969160fd83b97685e9019d2de4decb0c103a376f10786",
    };
    const { C, H, tag, R } = plan.source;
    const wrongSource = { C, H: "1".repeat(40), tag, R };
    fs.writeFileSync(
      receiptPath,
      JSON.stringify({
        schema: "vize-public-registry-install-v1",
        success: true,
        version: "0.439.0",
        source: wrongSource,
      }),
    );
    fs.writeFileSync(planPath, JSON.stringify(plan));
    assert.throws(() => installedLspRuntime(output, planPath), /reviewed install receipt changed/u);
    plan.installReceipt.sha256 = sha256(fs.readFileSync(receiptPath));
    fs.writeFileSync(planPath, JSON.stringify(plan));
    assert.throws(() => installedLspRuntime(output, planPath), {
      name: "AssertionError",
      actual: wrongSource,
      expected: { C, H, tag, R },
    });
    assert.equal(fs.existsSync(output), false);
  } finally {
    for (const [key, value] of ambient) {
      if (value === undefined) delete process.env[key];
      else process.env[key] = value;
    }
  }
});
