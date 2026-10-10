import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { root, testOutputRoot } from "./paths.ts";
import type { JsonRpcMessage } from "./protocol.ts";
import type { LspSession } from "./session.ts";

export const freshPolicyFixtureRoot = path.join(
  root,
  "tests/_fixtures/differential/lsp/config-defaults-8371",
);

export function readPolicyFixture(name: string): string {
  return fs.readFileSync(path.join(freshPolicyFixtureRoot, name), "utf8");
}

export const FRESH_EDITOR_CAPABILITIES = JSON.parse(readPolicyFixture("capabilities.fresh.json"));

/** Keep whole initialize responses separate from the independently authored oracle. */
export function observeInitializePacket(session: LspSession, label: string): () => JsonRpcMessage {
  let packet: JsonRpcMessage | undefined;
  session.responseObservers.push((message) => {
    if (message.id !== 1) return;
    packet = message;
    const output = path.join(testOutputRoot, "lsp-fresh-capability-policy");
    fs.mkdirSync(output, { recursive: true });
    fs.writeFileSync(
      path.join(output, `${label}.initialize.json`),
      JSON.stringify(message, null, 2) + "\n",
    );
  });
  return () => {
    assert.ok(packet, "the complete initialize response must be observed");
    return packet;
  };
}

export function assertInitializePacket(packet: JsonRpcMessage, capabilities: unknown): void {
  const { version } = JSON.parse(fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf8"));
  assert.deepEqual(packet, {
    jsonrpc: "2.0",
    id: 1,
    result: { capabilities, serverInfo: { name: "vize-maestro", version } },
  });
}
