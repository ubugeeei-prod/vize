import assert from "node:assert/strict";
import { Buffer } from "node:buffer";
import { Readable } from "node:stream";
import { test } from "node:test";
import { crc32 } from "node:zlib";

import { downloadArtifactEntries } from "../../legacy-tools/github/release-preflight-artifact-entries.mjs";

const artifact = {
  name: "real-project-matrix-0",
  archive_download_url: "https://example.test/artifacts/0.zip",
};

test("artifact download reads a bounded archive into text entries", async () => {
  const entries = await downloadArtifactEntries({
    artifact,
    token: "token",
    fetchImpl: async () => zipResponse([["summary.json", '{"ok":true}\n']]),
  });
  assert.deepEqual([...entries.keys()], ["summary.json"]);
  assert.equal(entries.get("summary.json"), '{"ok":true}\n');
});

test("artifact download stops before buffering an oversized response", async () => {
  let delivered = 0;
  await assert.rejects(
    downloadArtifactEntries({
      artifact,
      token: "token",
      limits: { maxBytes: 64 },
      fetchImpl: async () =>
        streamResponse(
          (function* chunks() {
            for (let index = 0; index < 8; index += 1) {
              delivered += 1;
              yield Buffer.alloc(32, index);
            }
          })(),
        ),
    }),
    /exceeds 64 bytes/,
  );
  // The limit must trip while streaming, not after the whole body is in memory.
  assert.ok(delivered <= 3, `streamed ${delivered} chunks past the limit`);
});

test("artifact download rejects archives that exceed entry or size budgets", async () => {
  const payload: [string, string][] = [
    ["a.json", "1"],
    ["b.json", "2"],
    ["c.json", "3"],
  ];
  await assert.rejects(
    downloadArtifactEntries({
      artifact,
      token: "token",
      limits: { maxEntries: 2 },
      fetchImpl: async () => zipResponse(payload),
    }),
    /declares 3 entries; the limit is 2/,
  );
  await assert.rejects(
    downloadArtifactEntries({
      artifact,
      token: "token",
      limits: { maxUncompressedBytes: 2 },
      fetchImpl: async () => zipResponse(payload),
    }),
    /declares 3 uncompressed bytes; the limit is 2/,
  );
});

test("artifact download rejects empty and unreadable archives", async () => {
  await assert.rejects(
    downloadArtifactEntries({
      artifact,
      token: "token",
      fetchImpl: async () => streamResponse([]),
    }),
    /downloaded no bytes/,
  );
  await assert.rejects(
    downloadArtifactEntries({
      artifact,
      token: "token",
      fetchImpl: async () => streamResponse([Buffer.from("not a zip archive\n")]),
    }),
    /Failed to inspect Real Project Matrix artifact real-project-matrix-0/,
  );
});

test("artifact download fails closed on missing URLs and error responses", async () => {
  await assert.rejects(
    downloadArtifactEntries({
      artifact: { name: "real-project-matrix-0" },
      token: "token",
      fetchImpl: async () => zipResponse([["summary.json", "{}"]]),
    }),
    /has no download URL/,
  );
  await assert.rejects(
    downloadArtifactEntries({
      artifact,
      token: "token",
      fetchImpl: async () => ({ ok: false, status: 500, statusText: "Server Error" }),
    }),
    /500 Server Error/,
  );
});

function streamResponse(chunks: Iterable<Buffer>) {
  return { ok: true, status: 200, statusText: "OK", body: Readable.from(chunks) };
}

function zipResponse(files: [string, string][]) {
  return streamResponse([storedZip(files)]);
}

// Minimal stored (uncompressed) ZIP writer so the download path is exercised
// without depending on an external `zip` binary.
function storedZip(files: [string, string][]) {
  const locals: Buffer[] = [];
  const central: Buffer[] = [];
  let offset = 0;
  for (const [name, text] of files) {
    const nameBytes = Buffer.from(name, "utf8");
    const data = Buffer.from(text, "utf8");
    const checksum = crc32(data);
    const local = Buffer.alloc(30 + nameBytes.byteLength);
    local.writeUInt32LE(0x04034b50, 0);
    local.writeUInt16LE(20, 4);
    local.writeUInt32LE(checksum, 14);
    local.writeUInt32LE(data.byteLength, 18);
    local.writeUInt32LE(data.byteLength, 22);
    local.writeUInt16LE(nameBytes.byteLength, 26);
    nameBytes.copy(local, 30);
    locals.push(local, data);

    const header = Buffer.alloc(46 + nameBytes.byteLength);
    header.writeUInt32LE(0x02014b50, 0);
    header.writeUInt16LE(20, 4);
    header.writeUInt16LE(20, 6);
    header.writeUInt32LE(checksum, 16);
    header.writeUInt32LE(data.byteLength, 20);
    header.writeUInt32LE(data.byteLength, 24);
    header.writeUInt16LE(nameBytes.byteLength, 28);
    header.writeUInt32LE(offset, 42);
    nameBytes.copy(header, 46);
    central.push(header);
    offset += local.byteLength + data.byteLength;
  }
  const directory = Buffer.concat(central);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(files.length, 8);
  end.writeUInt16LE(files.length, 10);
  end.writeUInt32LE(directory.byteLength, 12);
  end.writeUInt32LE(offset, 16);
  return Buffer.concat([...locals, directory, end]);
}
