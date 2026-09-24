import assert from "node:assert/strict";
import { test } from "node:test";
import { gunzipSync, gzipSync } from "node:zlib";
import { shallowRef } from "vue";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import {
  compress,
  decompress,
  decompressText,
  useCompressionStream,
} from "./use-compression-stream.ts";
import type { CompressionStreamConstructor } from "./use-compression-stream.ts";

// Node's global streams, injected explicitly: default resolution goes through `window`.
const NodeCompressionStream: CompressionStreamConstructor = globalThis.CompressionStream;
const NodeDecompressionStream: CompressionStreamConstructor = globalThis.DecompressionStream;
const streams = {
  CompressionStream: NodeCompressionStream,
  DecompressionStream: NodeDecompressionStream,
};

const text = "vize ".repeat(200);

void test("round-trips text through every format", async () => {
  for (const format of ["gzip", "deflate", "deflate-raw"] as const) {
    const packed = await compress(text, format, streams);
    assert.ok(packed.byteLength < text.length, `${format} shrinks repetitive text`);
    assert.equal(await decompressText(packed, format, streams), text);
  }
});

void test("interoperates with zlib gzip", async () => {
  const packed = await compress(text, "gzip", streams);
  assert.equal(gunzipSync(packed).toString("utf8"), text);
  const fromZlib = gzipSync(Buffer.from("hello"));
  assert.equal(await decompressText(fromZlib, "gzip", streams), "hello");
});

void test("accepts buffers, views, blobs and streams", async () => {
  const bytes = new TextEncoder().encode("payload");
  const expected = [...bytes];
  const inputs = [
    bytes.buffer,
    new DataView(bytes.buffer),
    new Blob([bytes]),
    new ReadableStream<Uint8Array<ArrayBuffer>>({
      start(controller) {
        controller.enqueue(bytes.slice(0, 3));
        controller.enqueue(bytes.slice(3));
        controller.close();
      },
    }),
  ];
  for (const input of inputs) {
    const packed = await compress(input, "deflate", streams);
    assert.deepEqual([...(await decompress(packed, "deflate", streams))], expected);
  }
  const offsetView = new Uint8Array([0, ...bytes, 0]).subarray(1, bytes.length + 1);
  const packed = await compress(offsetView, "gzip", streams);
  assert.deepEqual([...(await decompress(packed, "gzip", streams))], expected);
});

void test("rejects invalid formats, missing constructors and corrupt input", async () => {
  await assert.rejects(
    // @ts-expect-error runtime validation of an unknown format.
    compress("x", "brotli", streams),
    /VIZE_COMPOSE_COMPRESSION_INVALID_FORMAT/,
  );
  await assert.rejects(
    compress("x", "gzip", { CompressionStream: null }),
    /VIZE_COMPOSE_COMPRESSION_UNSUPPORTED/,
  );
  await assert.rejects(decompress("x", "gzip"), /DecompressionStream is not available/);
  await assert.rejects(decompress(new Uint8Array([1, 2, 3]), "gzip", streams));
});

void test("the composable tracks pending and error", async () => {
  const controls = useCompressionStream({ ...streams, format: "deflate-raw" });
  assert.equal(controls.supported.value, true);
  const running = controls.compress(text);
  assert.equal(controls.pending.value, true);
  const packed = await running;
  assert.equal(controls.pending.value, false);
  assert.equal(await controls.decompressText(packed), text);
  assert.deepEqual(
    [...(await controls.decompress(await controls.compress("ab", "gzip"), "gzip"))],
    [97, 98],
  );

  await assert.rejects(controls.decompress(new Uint8Array([9, 9, 9])));
  assert.ok(controls.error.value instanceof Error);
  assert.equal(controls.pending.value, false);
  await controls.compress("ok");
  assert.equal(controls.error.value, undefined);
});

void test("supported follows injected constructor refs", async () => {
  const Compression = shallowRef<CompressionStreamConstructor | null>(null);
  const controls = useCompressionStream({
    CompressionStream: Compression,
    DecompressionStream: NodeDecompressionStream,
  });
  assert.equal(controls.supported.value, false);
  await assert.rejects(controls.compress("x"), /VIZE_COMPOSE_COMPRESSION_UNSUPPORTED/);
  Compression.value = NodeCompressionStream;
  assert.equal(controls.supported.value, true);
  assert.equal(await controls.decompressText(await controls.compress("x")), "x");
});

void test("server rendering reports unsupported", async () => {
  const state = await renderComposableOnServer(() => {
    const controls = useCompressionStream();
    return { supported: controls.supported, pending: controls.pending, error: controls.error };
  });
  assert.equal(state, '{"supported":false,"pending":false}');
});
