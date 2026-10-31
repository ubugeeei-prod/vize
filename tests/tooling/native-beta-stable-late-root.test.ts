import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  verifyNativeRelations,
  type WatcherContext,
} from "./support/native-beta-dto/stable-watcher-boundary.ts";
import {
  observeLateRootProgramWatcher,
  verifyHistoricalWatcherRelations,
} from "./support/native-beta-dto/stable-watcher-late-root.ts";

type Packet = Record<string, unknown>;
const object = (value: unknown) => value as Packet;
const directory = new URL(
  "../_fixtures/differential/typechecker/native-beta-diagnostic-dto-10731/stable-post-shutdown-41099824102/",
  import.meta.url,
);
function packets(name: string): Packet[] {
  const raw = fs.readFileSync(new URL(name, directory)),
    rows: Packet[] = [];
  let offset = 0;
  while (offset < raw.length) {
    const end = raw.indexOf("\r\n\r\n", offset);
    assert.ok(end >= 0);
    const header = raw.subarray(offset, end).toString("ascii");
    assert.match(header, /^Content-Length: [0-9]+$/u);
    const next = end + 4 + Number(header.slice(16));
    assert.ok(next <= raw.length);
    rows.push(JSON.parse(raw.subarray(end + 4, next).toString("utf8")) as Packet);
    offset = next;
  }
  assert.equal(offset, raw.length);
  return rows;
}
function fixture(late = true) {
  const identity = JSON.parse(fs.readFileSync(new URL("identity.json", directory), "utf8")) as {
    context: WatcherContext;
  };
  const client = packets("client.bin"),
    server = packets("server.bin");
  if (late) [server[20], server[21]] = [server[21]!, server[20]!];
  return { client, server, context: structuredClone(identity.context) };
}
type Fixture = ReturnType<typeof fixture>;
const call = (f: Fixture, id: string | number) =>
  f.server.find((row) => row.id === id && "method" in row)!;
const registration = (f: Fixture, id: string) =>
  object((object(call(f, id).params).registrations as unknown[])[0]);
const watcher = (f: Fixture, id: string) =>
  object((object(registration(f, id).registerOptions).watchers as unknown[])[0]);

void test("late root observer keeps every packet and grants no baseline qualification", () => {
  const f = fixture();
  assert.throws(() => verifyNativeRelations(f.client, f.server, f.context));
  const result = verifyHistoricalWatcherRelations(f.client, f.server, f.context);
  assert.ok(result && result.boundary === "post-shutdown-root-and-sdk-program-files");
  assert.equal(result.shutdownAckIndex, 20);
  assert.equal(result.rootRequestIndex, 21);
  assert.equal(result.requestIndex, 26);
  assert.equal(result.rootAckIndex, 6);
  assert.equal(result.observedRootRegistrationAcknowledged, true);
  assert.deepEqual(
    [
      result.nativeQualification,
      result.dtoFunctionalPass,
      result.lifecyclePass,
      result.registrationPass,
    ],
    [false, false, false, false],
  );
  assert.strictEqual(result.clientPackets, f.client);
  assert.strictEqual(result.serverPackets, f.server);
});

void test("strict default and fully acknowledged streams retain their original behavior", () => {
  const f = fixture(false);
  assert.deepEqual(
    verifyHistoricalWatcherRelations(f.client, f.server, f.context),
    verifyNativeRelations(f.client, f.server, f.context),
  );
  assert.throws(() => observeLateRootProgramWatcher(f.client, f.server, f.context));
  const answered = fixture();
  answered.client.push({ jsonrpc: "2.0", id: "ts3", result: null });
  for (const role of ["before", "after"]) {
    answered.context.role = role;
    assert.equal(
      verifyHistoricalWatcherRelations(answered.client, answered.server, answered.context),
      null,
    );
  }
});

void test("late-root variant refuses wrong actor, scope, payload, ACK, ordering and closure", () => {
  const mutations: Array<[string, (f: Fixture) => void]> = [
    [
      "AFTER",
      (f) => {
        f.context.role = "after";
      },
    ],
    [
      "source",
      (f) => {
        f.context.source = "0".repeat(40);
      },
    ],
    [
      "beta",
      (f) => {
        f.context.sdkVersion = "7.1.0-beta";
      },
    ],
    [
      "image",
      (f) => {
        f.context.sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "SDK path",
      (f) => {
        f.context.sdkExecutable = "/foreign/node_modules/@typescript/typescript-linux-x64/lib/tsc";
      },
    ],
    [
      "project",
      (f) => {
        f.context.projectRoots = ["/tmp/foreign"];
      },
    ],
    [
      "actor",
      (f) => {
        object(
          object(f.server.find((row) => row.id === 1 && "result" in row)!.result).serverInfo,
        ).version = "7.1.0-beta";
      },
    ],
    [
      "owned config",
      (f) => {
        object(f.server.find((row) => row.id === 3 && "result" in row)!.result).configFilePath =
          "/tmp/foreign/tsconfig.json";
      },
    ],
    [
      "root glob",
      (f) => {
        watcher(f, "ts2").globPattern = "/tmp/**/*";
      },
    ],
    [
      "program glob",
      (f) => {
        watcher(f, "ts3").globPattern = "/tmp/**/*";
      },
    ],
    [
      "root kind",
      (f) => {
        watcher(f, "ts2").kind = 1;
      },
    ],
    [
      "program id",
      (f) => {
        registration(f, "ts3").id = "different watcher";
      },
    ],
    [
      "configuration",
      (f) => {
        registration(f, "ts1").other = true;
      },
    ],
    [
      "header",
      (f) => {
        object(f.server[23]!.params).message = "Added new watch: foreign";
      },
    ],
    [
      "root ACK missing",
      (f) => {
        f.client = f.client.filter((row) => row.id !== "ts2");
      },
    ],
    [
      "root ACK error",
      (f) => {
        f.client.find((row) => row.id === "ts2")!.error = { code: -1 };
      },
    ],
    [
      "root ACK early",
      (f) => {
        [f.client[5], f.client[6]] = [f.client[6]!, f.client[5]!];
      },
    ],
    [
      "root before shutdown",
      (f) => {
        [f.server[20], f.server[21]] = [f.server[21]!, f.server[20]!];
      },
    ],
    [
      "program before root",
      (f) => {
        [f.server[21], f.server[26]] = [f.server[26]!, f.server[21]!];
      },
    ],
    [
      "duplicate request",
      (f) => {
        f.server.push(structuredClone(call(f, "ts3")));
      },
    ],
    [
      "duplicate ACK",
      (f) => {
        f.client.push(structuredClone(f.client[6]!));
      },
    ],
    [
      "other unmatched",
      (f) => {
        f.server[2] = { jsonrpc: "2.0", id: "extra", method: "unknown" };
      },
    ],
    [
      "missing frame",
      (f) => {
        f.server.splice(2, 1);
      },
    ],
    [
      "error response",
      (f) => {
        f.server.find((row) => row.id === 2 && "result" in row)!.error = { code: -1 };
      },
    ],
  ];
  for (const [label, mutate] of mutations) {
    const f = fixture();
    mutate(f);
    assert.throws(() => observeLateRootProgramWatcher(f.client, f.server, f.context), label);
  }
});
