import assert from "node:assert/strict";
import fs from "node:fs";
import type { NativeOwner } from "./lsp-strace.ts";
import type { NativeRuntimeConsumer } from "./providers.ts";

export function validateNativeLifecycle(
  owners: readonly NativeOwner[],
  consumer: Pick<NativeRuntimeConsumer, "expectedExecutable" | "kind">,
  knownContract: unknown,
) {
  const positive = consumer.expectedExecutable !== null;
  if (!positive) {
    assert.equal(owners.length, 0, "no undeclared, bundled, cached or global compiler may run");
    return { selectedProvider: false, nativeCleanExitQualified: false, semanticCredit: false };
  }
  assert.ok(
    owners.length > 0,
    "real compiler use must be observed independently of outer responses",
  );
  const sdk = consumer.expectedExecutable;
  assert.ok(sdk);
  for (const owner of owners) {
    assert.equal(owner.executable.physicalPath, sdk.path);
    assert.equal(owner.executable.sha256, sdk.sha256);
    assert.equal(owner.exit.signal, null);
  }
  const lsp = owners.filter((owner) => owner.argv.includes("--lsp"));
  assert.ok(lsp.length > 0, "the selected actual provider must serve LSP");
  const packets = (owner: NativeOwner) => ({
    client: owner.streams.client.frames.map((frame) => frame.packet),
    server: owner.streams.server.frames.map((frame) => frame.packet),
  });
  let realWrongBuffer = false,
    realTypeError = false;
  for (const owner of lsp) {
    const { client, server } = packets(owner);
    realWrongBuffer ||= client.some(
      (packet) =>
        (packet.method === "textDocument/didOpen" &&
          packet.params?.textDocument?.text?.includes('"wrong"')) ||
        (packet.method === "textDocument/didChange" &&
          packet.params?.contentChanges?.some((change: { text?: string }) =>
            change.text?.includes('"wrong"'),
          )),
    );
    realTypeError ||= server.some((packet) =>
      packet.result?.items?.some((diagnostic: { code?: unknown }) => diagnostic.code === 2322),
    );
    const initialized = server.filter(
      (packet) =>
        packet.id === client.find((request) => request.method === "initialize")?.id &&
        !packet.method,
    );
    assert.equal(initialized.length, 1);
    assert.equal(initialized[0].error, undefined);
    if (consumer.kind.endsWith("702")) {
      assert.deepEqual(
        initialized[0].result,
        knownContract,
        "exact qualified image retains its complete source-owned Initialize contract",
      );
      const shutdown = client.filter((packet) => packet.method === "shutdown");
      assert.equal(shutdown.length, 1);
      assert.deepEqual(
        server.find((packet) => !packet.method && packet.id === shutdown[0].id),
        { jsonrpc: "2.0", id: shutdown[0].id, result: null },
      );
      assert.ok(
        server.some(
          (packet) =>
            packet.method === "window/logMessage" &&
            packet.params?.type === 3 &&
            packet.params?.message?.startsWith(`handled method 'shutdown' (${shutdown[0].id}) in `),
        ),
        "actual current shutdown completion",
      );
      assert.equal(
        client.some((packet) => packet.method === "exit"),
        false,
        "qualified EOF closure does not borrow the standard Exit path",
      );
      assert.deepEqual(owner.exit, { code: 0, signal: null });
      assert.equal(fs.readFileSync(owner.streams.stderr.path).length, 0);
    } else {
      const stderr = fs.readFileSync(owner.streams.stderr.path, "utf8");
      assert.ok(
        (owner.exit.code === 0 && stderr === "") ||
          (owner.exit.code === 1 && stderr === "context canceled\n") ||
          (owner.exit.code === 0 && stderr === "error handling method 'exit': EOF\n"),
        `unrecognized beta terminal behavior: ${owner.exit.code} ${JSON.stringify(stderr)}`,
      );
    }
  }
  assert.ok(realWrongBuffer, "actual selected LSP receives the genuine unsaved error text");
  assert.ok(realTypeError, "actual selected native LSP returns the genuine type error");
  const apiErrors = owners
    .filter((owner) => owner.argv.includes("--api"))
    .flatMap((owner) =>
      owner.streams.server.frames
        .filter((frame) => frame.packet.error)
        .map((frame) => frame.packet),
    );
  for (const owner of owners.filter((owner) => owner.argv.includes("--api"))) {
    assert.deepEqual(owner.exit, { code: 0, signal: null });
    assert.equal(fs.readFileSync(owner.streams.stderr.path).length, 0);
  }
  return {
    selectedProvider: true,
    nativeCleanExitQualified: consumer.kind.endsWith("702"),
    betaStandardExitFrontier: !consumer.kind.endsWith("702"),
    apiCompatibilityCredit: false,
    actualApiErrors: apiErrors,
    actualNativeSemanticTraffic: true,
    semanticCredit: false,
  };
}
