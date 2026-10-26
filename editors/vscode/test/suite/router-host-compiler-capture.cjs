const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const { isDeepStrictEqual } = require("node:util");

// Bind an in-memory compiler snapshot to the actual native completion response
// and its preceding request. A same-spelling file opened by another owner does
// not supply authority. This observer never changes either protocol stream.
function captureCompilerDocument(events, completion) {
  assert.ok(completion.length > 0);
  const envelope = completion[0].data.vizeCompletion;
  const items = completion.map(({ data }) => data.vizeCompletion.item);
  let response = -1;
  for (let index = events.length - 1; index >= 0; index--) {
    const event = events[index];
    const result = event.packet?.result;
    const value = Array.isArray(result) ? result : result?.items;
    if (event.kind === "frame" && event.direction === "server" && isDeepStrictEqual(value, items)) {
      response = index;
      break;
    }
  }
  assert.ok(response >= 0, "complete native item vector must match a retained compiler response");
  const owner = events[response];
  const request = events.findLastIndex(
    (event, index) =>
      index < response &&
      event.kind === "frame" &&
      event.pid === owner.pid &&
      event.direction === "client" &&
      event.packet.id === owner.packet.id &&
      event.packet.method === "textDocument/completion" &&
      event.packet.params.textDocument.uri === envelope.requestUri,
  );
  assert.ok(request >= 0, "the native response must have the matching same-owner request");
  let source;
  let version;
  const snapshotFrames = [];
  for (let index = 0; index < request; index++) {
    const event = events[index];
    if (event.kind !== "frame" || event.pid !== owner.pid || event.direction !== "client") continue;
    const { method, params } = event.packet;
    if (params?.textDocument?.uri !== envelope.requestUri) continue;
    if (method === "textDocument/didOpen") {
      source = params.textDocument.text;
      version = params.textDocument.version;
      snapshotFrames.push(index);
    } else if (method === "textDocument/didChange") {
      assert.equal(typeof source, "string");
      assert.ok(params.textDocument.version > version);
      for (const change of params.contentChanges) {
        assert.equal(
          change.range,
          undefined,
          "this closed adapter admits compiler full snapshots only",
        );
        source = change.text;
      }
      version = params.textDocument.version;
      snapshotFrames.push(index);
    } else if (method === "textDocument/didClose") {
      source = undefined;
      snapshotFrames.length = 0;
    }
  }
  assert.equal(
    typeof source,
    "string",
    "the same compiler owner must have an open source snapshot",
  );
  const bytes = Buffer.from(source);
  return {
    file: items[0].data.fileName,
    source,
    bytes: bytes.length,
    sha256: createHash("sha256").update(bytes).digest("hex"),
    observation: {
      pid: owner.pid,
      request,
      response,
      snapshotFrames,
      version,
      requestUri: envelope.requestUri,
    },
  };
}

module.exports = { captureCompilerDocument };
