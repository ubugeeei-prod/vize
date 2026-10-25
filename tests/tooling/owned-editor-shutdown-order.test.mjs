import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  decodeNativeIo,
  decodeNativeFrames,
} from "./support/native-runtime-selection/lsp-strace.mjs";
import { assertOwnedShutdownOrder } from "./support/native-runtime-selection/lsp-owned-outer-shutdown-order.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const initializeRequest = JSON.parse(
  fs.readFileSync(
    path.join(
      root,
      "tests/_fixtures/differential/lsp/owned-outer-shutdown-10760/initialize-request.json",
    ),
    "utf8",
  ),
);
const nativeSource = fs.readFileSync(
  path.join(root, "crates/vize_canon/src/lsp_client/editor_lsp/native_shutdown.rs"),
  "utf8",
);
const knownContract = JSON.parse(
  /fn known_contract\(\) -> Value \{\s*serde_json::json!\(([\s\S]*?)\)\s*\}/u.exec(nativeSource)[1],
);
const expectedExecutable = {
  path: "/unit/typescript/lib/tsc",
  sha256: "4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
};
const workspace = "/unit/workspace";
const frame = (packet) => {
  const bytes = Buffer.from(JSON.stringify(packet));
  return Buffer.concat([Buffer.from(`Content-Length: ${bytes.length}\r\n\r\n`), bytes]);
};
const quoted = (bytes) =>
  `"${[...bytes].map((byte) => `\\x${byte.toString(16).padStart(2, "0")}`).join("")}"`;
const row = (tid, time, call) => ({
  tid,
  at: `${time}.000000`,
  completedAt: `${time}.000000`,
  duration: "0.000001",
  interval: {
    startNanoseconds: String(BigInt(time) * 1_000_000_000n),
    endNanoseconds: String(BigInt(time) * 1_000_000_000n + 1_000n),
  },
  call,
});
const io = (tid, time, operation, fd, bytes) =>
  row(
    tid,
    time,
    `${operation}(${fd}, ${quoted(bytes)}, ${bytes.length || 4096}) = ${bytes.length} <0.000001>`,
  );

// Deliberately synthetic pure-oracle rows: never source/provider acceptance.
function seed() {
  const init = structuredClone(initializeRequest);
  init.params.processId = 100;
  init.params.rootPath = workspace;
  init.params.rootUri = pathToFileURL(workspace).href;
  init.params.workspaceFolders = [
    { name: path.basename(workspace), uri: pathToFileURL(workspace).href },
  ];
  const shutdown = { jsonrpc: "2.0", id: 2, method: "shutdown" };
  const completion = {
    jsonrpc: "2.0",
    method: "window/logMessage",
    params: { type: 3, message: "handled method 'shutdown' (2) in 0ms" },
  };
  const native = {
    process: 200,
    path: expectedExecutable.path,
    argv: [expectedExecutable.path, "--lsp", "--stdio"],
    executable: { physicalPath: expectedExecutable.path, sha256: expectedExecutable.sha256 },
    exit: { code: 0, signal: null },
    rows: [
      io(200, 2, "read", 0, frame(init)),
      io(200, 3, "write", 1, frame({ jsonrpc: "2.0", id: 1, result: knownContract })),
      io(200, 4, "read", 0, frame(shutdown)),
      io(
        200,
        5,
        "write",
        1,
        Buffer.concat([frame({ jsonrpc: "2.0", id: 2, result: null }), frame(completion)]),
      ),
      io(200, 6, "read", 0, Buffer.alloc(0)),
      row(200, 7, "exit_group(0) = ?"),
      row(200, 8, "+++ exited with 0 +++"),
    ],
  };
  const outerShutdown = { jsonrpc: "2.0", id: 10, result: null };
  const source = {
    process: 100,
    exit: { code: 0, signal: null },
    rows: [
      io(100, 9, "read", 0, frame({ jsonrpc: "2.0", id: 10, method: "shutdown" })),
      io(100, 10, "write", 1, frame(outerShutdown)),
      row(100, 14, "+++ exited with 0 +++"),
    ],
  };
  return {
    sourceOwner: decodeNativeIo(source),
    nativeOwners: [decodeNativeIo(native)],
    outerShutdown,
    knownContract,
    initializeRequest,
    workspace,
    expectedExecutable,
  };
}

void test("whole native ACK and completion may share a write, with actual EOF and every terminal before outer ACK", () => {
  assert.doesNotThrow(() => assertOwnedShutdownOrder(seed()));
});

void test("missing Shutdown, EOF, current completion, whole Initialize or exact image never borrows closure credit", () => {
  for (const change of [
    (data) => {
      data.nativeOwners[0].rows.splice(2, 1);
    },
    (data) => {
      data.nativeOwners[0].rows.splice(4, 1);
    },
    (data) => {
      data.nativeOwners[0].executable.sha256 = "0".repeat(64);
    },
    (data) => {
      data.nativeOwners[0].executable.physicalPath = "/foreign/compiler";
    },
    (data) => {
      data.nativeOwners[0].exit.code = 1;
    },
    (data) => {
      data.nativeOwners[0].rows[3] = io(
        200,
        5,
        "write",
        1,
        frame({ jsonrpc: "2.0", id: 2, result: null }),
      );
    },
    (data) => {
      data.nativeOwners[0].rows[3] = io(
        200,
        5,
        "write",
        1,
        frame({ jsonrpc: "2.0", id: 99, result: null }),
      );
    },
    (data) => {
      data.nativeOwners[0].rows[1] = io(
        200,
        3,
        "write",
        1,
        frame({ jsonrpc: "2.0", id: 1, result: {} }),
      );
    },
    (data) => {
      data.nativeOwners[0].rows.push(row(201, 8, "+++ exited with 1 +++"));
    },
  ]) {
    const data = seed();
    change(data);
    for (const native of data.nativeOwners) decodeNativeIo(native);
    assert.throws(() => assertOwnedShutdownOrder(data), String(change));
  }
});

void test("late, equal or overlapping terminal intervals and any unclosed additional SDK fail", () => {
  for (const time of [10, 11]) {
    const data = seed();
    data.nativeOwners[0].rows[6] = row(200, time, "+++ exited with 0 +++");
    assert.throws(() => assertOwnedShutdownOrder(data));
  }
  const overlap = seed();
  overlap.nativeOwners[0].rows[6].interval.endNanoseconds =
    overlap.sourceOwner.rows[1].interval.startNanoseconds;
  assert.throws(() => assertOwnedShutdownOrder(overlap));
  const extra = seed(),
    unclosed = seed().nativeOwners[0];
  unclosed.process = 300;
  unclosed.rows = unclosed.rows.filter((item) => !item.call.startsWith("read(0,"));
  decodeNativeIo(unclosed);
  extra.nativeOwners.push(unclosed);
  assert.throws(() => assertOwnedShutdownOrder(extra));
});

void test("foreign outer response or unproved stdin EOF and completion order fail", () => {
  for (const change of [
    (data) => {
      data.outerShutdown = { ...data.outerShutdown, id: 99 };
    },
    (data) => {
      data.outerShutdown = { ...data.outerShutdown, extra: true };
    },
    (data) => {
      data.nativeOwners[0].rows[4] = row(200, 6, "read(0,  <unfinished ...>");
    },
    (data) => {
      data.nativeOwners[0].rows[4] = io(200, 4, "read", 0, Buffer.alloc(0));
    },
  ]) {
    const data = seed();
    change(data);
    assert.throws(() => {
      for (const native of data.nativeOwners) decodeNativeIo(native);
      return assertOwnedShutdownOrder(data);
    });
  }
});

void test("full Initialize custody refuses foreign parent, undeclared root relocation and unknown fields", () => {
  for (const change of [
    (packet) => {
      packet.params.processId = 999;
    },
    (packet) => {
      packet.params.rootPath = "/foreign/project";
    },
    (packet) => {
      packet.params.workspaceFolders[0].name = "foreign";
    },
    (packet) => {
      packet.params.extra = true;
    },
    (packet) => {
      packet.id = 999;
    },
  ]) {
    const data = seed(),
      owner = data.nativeOwners[0];
    const packet = decodeNativeFrames(owner.events[0].bytes)[0].packet;
    change(packet);
    owner.rows[0] = io(200, 2, "read", 0, frame(packet));
    decodeNativeIo(owner);
    assert.throws(() => assertOwnedShutdownOrder(data));
  }
});
