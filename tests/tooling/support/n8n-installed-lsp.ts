import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { decodeFrames, LspWire } from "../../differential/lsp-wire.ts";
import { root } from "./lsp/paths.ts";
import { writeJson } from "./n8n-cli-config-workspace.mjs";
import {
  exactPath,
  installedAuthority,
  sha256,
  type InstalledCampaignPlan,
} from "./n8n-installed-authority.ts";
import { installedInvocationPreflight, validateNativeJournal } from "./n8n-installed-cli.ts";

/** Use the existing wire transport with an authenticated public npm launch. */
export function installedLspRuntime(output: string, planPath: string) {
  const plan = JSON.parse(fs.readFileSync(exactPath(planPath), "utf8")) as InstalledCampaignPlan;
  const custody = installedAuthority(plan, root);
  const { authority } = custody;
  assert.ok(path.isAbsolute(output));
  exactPath(path.dirname(output), undefined, true);
  assert.equal(fs.existsSync(output), false, "installed LSP capture must be new");
  assert.equal(output.startsWith(authority.installRoot + path.sep), false);
  fs.mkdirSync(output);
  const journalPath = path.join(output, "native-journal.jsonl");
  const observer = {
    schema: "vize-public-native-custody-v1",
    installRoot: authority.installRoot,
    nativePath: authority.native.path,
    nativeSha256: authority.native.sha256,
    journalPath,
  };
  const argv = ["--require", authority.custodyHook.path, authority.cli.binPath, "lsp"];
  installedInvocationPreflight(custody.recheck);
  const wire = new LspWire(authority.node.path, argv, authority.installRoot, {
    ...process.env,
    NO_COLOR: "1",
    VIZE_PUBLIC_NATIVE_CUSTODY: JSON.stringify(observer),
  });
  const identity = {
    authority: "public-npm-install",
    source: plan.source,
    receiptPath: custody.receiptPath,
    receiptSha256: custody.receiptSha256,
    collectorAuthority: authority.collectorAuthority,
    node: authority.node,
    cli: authority.cli,
    native: authority.native,
    command: authority.node.path,
    argv,
    cwd: authority.installRoot,
    pid: wire.child.pid,
  };
  writeJson(path.join(output, "producer.json"), identity);
  let observedMessages = 0;
  const capture = () => {
    // Match the existing source client for requests it does not implement.
    for (const message of wire.messages.slice(observedMessages)) {
      if (typeof message.id === "number" && message.method != null) {
        wire.send({
          jsonrpc: "2.0",
          id: message.id,
          error: { code: -32601, message: `client does not implement ${message.method}` },
        });
      }
    }
    observedMessages = wire.messages.length;
    const journal = fs.existsSync(journalPath) ? fs.readFileSync(journalPath, "utf8") : "";
    const observation = {
      identity,
      wire: wire.observation(),
      nativeJournal: { path: journalPath, sha256: sha256(journal), raw: journal },
    };
    writeJson(path.join(output, "observation.json"), observation);
    return observation;
  };
  // Persist the complete existing transport, even when a later packet fails.
  wire.child.stdout.on("data", capture);
  wire.child.stderr.on("data", capture);
  wire.child.on("close", capture);
  const finish = async () => {
    try {
      await wire.finish();
    } finally {
      await wire.stop();
      capture();
      custody.recheck();
    }
    assert.ok(wire.child.pid);
    const journal = fs.readFileSync(journalPath, "utf8");
    validateNativeJournal(journal, {
      ...observer,
      node: authority.node.path,
      argv: [authority.node.path, authority.cli.binPath, "lsp"],
      pid: wire.child.pid,
      status: 0,
      corsaPath: authority.bundledCorsa.path,
    });
    const observation = capture();
    for (const stream of ["clientWireBase64", "serverWireBase64"] as const)
      decodeFrames(Buffer.from(observation.wire[stream], "base64"));
  };
  return { wire, identity, capture, finish, recheck: custody.recheck };
}
