import assert from "node:assert/strict";

/** Bind whole native frame bytes to conservative bounds of their actual syscalls. */
export function timedFrames(events, operation, fd, decode) {
  let offset = 0;
  const segments = events
    .filter((row) => row.operation === operation && row.fd === fd && row.bytes)
    .map((row) => {
      const start = offset;
      offset += row.bytes.length;
      return { start, end: offset, interval: row.interval };
    });
  const bytes = Buffer.concat(
    events
      .filter((row) => row.operation === operation && row.fd === fd && row.bytes)
      .map((row) => row.bytes),
  );
  return decode(bytes).map((frame) => {
    const covered = segments.filter(
      (row) => row.start < frame.offset + frame.bytes && row.end > frame.offset,
    );
    assert(covered.length > 0);
    return {
      ...frame,
      interval: {
        startNanoseconds: covered
          .reduce(
            (minimum, row) =>
              BigInt(row.interval.startNanoseconds) < minimum
                ? BigInt(row.interval.startNanoseconds)
                : minimum,
            BigInt(covered[0].interval.startNanoseconds),
          )
          .toString(),
        endNanoseconds: covered
          .reduce(
            (maximum, row) =>
              BigInt(row.interval.endNanoseconds) > maximum
                ? BigInt(row.interval.endNanoseconds)
                : maximum,
            BigInt(covered[0].interval.endNanoseconds),
          )
          .toString(),
      },
    };
  });
}

/** Every actual DerivedSnapshot handle must receive its own release ACK before native shutdown. */
export function assertSnapshotDrainBeforeOwnerShutdown(input, output, shutdown) {
  assert.equal(shutdown.packet.method, "shutdown");
  const before = BigInt(shutdown.interval.startNanoseconds);
  const requests = input.filter((frame) => frame.packet.id !== undefined && frame.packet.method);
  const response = (request) => {
    const matches = output.filter(
      (frame) => frame.packet.id === request.packet.id && !frame.packet.method,
    );
    assert.equal(matches.length, 1, "whole same-ID API response");
    assert.equal(matches[0].packet.error, undefined);
    return matches[0];
  };
  const heads = requests
    .filter((frame) =>
      ["createSnapshot", "updateSnapshot", "deriveSnapshot"].includes(frame.packet.method),
    )
    .map((frame) => {
      const reply = response(frame);
      const handle = reply.packet.result?.snapshot;
      assert(
        Number.isSafeInteger(handle) && handle > 0,
        "exact known beta numeric snapshot handle",
      );
      return { request: frame, response: reply, handle };
    });
  assert(heads.length > 0, "actual API-owned snapshot heads are required");
  assert.equal(
    new Set(heads.map((row) => String(row.handle))).size,
    heads.length,
    "each derived head has a distinct observed handle",
  );
  const releases = requests.filter((frame) => frame.packet.method === "release");
  assert.equal(releases.length, heads.length, "no missing or extra release RPC");
  return heads.map((head) => {
    const matches = releases.filter(
      (frame) => String(frame.packet.params?.snapshot) === String(head.handle),
    );
    assert.equal(matches.length, 1, `one release for actual snapshot ${head.handle}`);
    const request = matches[0],
      reply = response(request);
    assert.deepEqual(
      request.packet.params,
      { handle: String(head.handle), snapshot: head.handle },
      "whole exact beta release parameters",
    );
    assert.equal(reply.packet.result, true, "known beta API release ACK is the whole true result");
    assert(
      BigInt(head.response.interval.endNanoseconds) < BigInt(request.interval.startNanoseconds),
      "snapshot creation completes before its release",
    );
    assert(
      BigInt(reply.interval.endNanoseconds) < before,
      `snapshot ${head.handle} release ACK precedes owner shutdown`,
    );
    return {
      snapshot: head.handle,
      created: head,
      released: { request, response: reply },
      nativeShutdown: shutdown,
    };
  });
}
