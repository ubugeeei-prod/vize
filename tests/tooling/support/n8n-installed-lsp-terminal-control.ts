// Actual public installed LSP terminal custody; this is not slot-policy acceptance.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { errorPacket } from "./n8n-cli-config-oracle.mjs";
import { writeJson } from "./n8n-cli-config-workspace.mjs";
import { installedLspRuntime } from "./n8n-installed-lsp.ts";

assert.ok(process.argv[2] && process.argv[3], "new output and reviewed public plan required");
const output = path.resolve(process.argv[2]);
assert.equal(fs.existsSync(output), false, "terminal capture must be new");
const session = installedLspRuntime(output, process.argv[3]);
let failure: unknown;
try {
  const initialized = await session.wire.request("initialize", {
    processId: process.pid,
    rootUri: pathToFileURL(session.identity.cwd).href,
    capabilities: {},
    initializationOptions: { editor: false, lint: false, typecheck: false },
  });
  writeJson(path.join(output, "initialize.json"), initialized);
  assert.equal(initialized.error, undefined);
  assert.ok(initialized.result);
  session.wire.notify("initialized", {});
  await session.finish();
} catch (error) {
  failure = errorPacket(error);
  writeJson(path.join(output, "failure.json"), failure);
} finally {
  await session.wire.stop();
  session.capture();
  session.recheck();
}
const terminal = session.wire.observation();
writeJson(path.join(output, "terminal.json"), terminal);
assert.equal(failure, undefined, "complete installed terminal failure retained");
assert.deepEqual(
  {
    exitStatus: terminal.exitStatus,
    signal: terminal.signal,
    processError: terminal.processError,
  },
  { exitStatus: 0, signal: null, processError: null },
);
writeJson(path.join(output, "qualified.json"), {
  identity: session.identity,
  terminal: { exitStatus: terminal.exitStatus, signal: terminal.signal },
  qualification: "public installed initialize/shutdown custody only; no slot or adoption credit",
});
