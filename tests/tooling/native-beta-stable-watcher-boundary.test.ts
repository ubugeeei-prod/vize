import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import {
  verifyNativeRelations,
  type WatcherContext,
} from "./support/native-beta-dto/stable-watcher-boundary.ts";

const directory = new URL(
  "../_fixtures/differential/typechecker/native-beta-diagnostic-dto-10731/stable-post-shutdown-41099824102/",
  import.meta.url,
);
type Packet = Record<string, unknown>;
const record = (value: unknown) => value as Packet;
function fixture() {
  const identity = JSON.parse(fs.readFileSync(new URL("identity.json", directory), "utf8")) as {
    context: WatcherContext;
    qualified: boolean;
    files: Array<{ path: string; bytes: number; sha256: string }>;
  };
  assert.equal(identity.qualified, false);
  const streams = identity.files.map((file) => {
    const raw = fs.readFileSync(new URL(file.path, directory));
    assert.equal(raw.length, file.bytes);
    assert.equal(createHash("sha256").update(raw).digest("hex"), file.sha256);
    let offset = 0;
    const packets: Packet[] = [];
    while (offset < raw.length) {
      const end = raw.indexOf("\r\n\r\n", offset);
      assert.ok(end >= 0);
      const header = raw.subarray(offset, end).toString("ascii");
      assert.match(header, /^Content-Length: [0-9]+$/u);
      const size = Number(header.slice("Content-Length: ".length));
      const next = end + 4 + size;
      assert.ok(next <= raw.length);
      packets.push(JSON.parse(raw.subarray(end + 4, next).toString("utf8")) as Packet);
      offset = next;
    }
    assert.equal(offset, raw.length);
    return packets;
  });
  return { client: streams[0]!, server: streams[1]!, context: structuredClone(identity.context) };
}
const program = (f: ReturnType<typeof fixture>) => f.server.find((row) => row.id === "ts3")!;
const registration = (f: ReturnType<typeof fixture>) =>
  record((record(program(f).params).registrations as unknown[])[0]);
const watcher = (f: ReturnType<typeof fixture>) =>
  record((record(registration(f).registerOptions).watchers as unknown[])[0]);

test("actual before trace observes a distinct SDK watcher after shutdown and grants no qualification", () => {
  const f = fixture(),
    observation = verifyNativeRelations(f.client, f.server, f.context);
  assert.ok(observation);
  assert.equal(observation.boundary, "post-shutdown-sdk-program-files");
  assert.equal(observation.requestIndex, 26);
  assert.equal(observation.shutdownAckIndex, 21);
  assert.deepEqual(observation.request, program(f));
  assert.equal(observation.nativeQualification, false);
  assert.equal(observation.lifecyclePass, false);
  assert.equal(observation.registrationPass, false);
  assert.deepEqual(observation.clientPackets, f.client);
  assert.deepEqual(observation.serverPackets, f.server);
  assert.equal(observation.request.id, "ts3", "raw IDs remain unchanged");
  assert.ok(
    f.client.some((row) => row.id === "ts2" && row.result === null),
    "different root watcher was actually answered",
  );
});

test("wrong role, actor, project, payload, ordering or pair closure cannot become a baseline observation", () => {
  const mutations: Array<[string, (f: ReturnType<typeof fixture>) => void]> = [
    [
      "after stable",
      (f) => {
        f.context.role = "after";
      },
    ],
    [
      "before beta",
      (f) => {
        f.context.sdkVersion = "7.1.0-beta";
      },
    ],
    [
      "after beta",
      (f) => {
        f.context.role = "after";
        f.context.sdkVersion = "7.1.0-beta";
      },
    ],
    [
      "other before source",
      (f) => {
        f.context.source = "0".repeat(40);
      },
    ],
    [
      "other native image",
      (f) => {
        f.context.sdkSha256 = "0".repeat(64);
      },
    ],
    [
      "other SDK path",
      (f) => {
        f.context.sdkExecutable = "/foreign/node_modules/@typescript/typescript-linux-x64/lib/tsc";
      },
    ],
    [
      "foreign observed project",
      (f) => {
        f.context.projectRoots = ["/tmp/unrelated"];
      },
    ],
    [
      "wrong native actor",
      (f) => {
        record(
          record(f.server.find((row) => row.id === 1 && "result" in row)!.result).serverInfo,
        ).version = "7.1.0-beta";
      },
    ],
    [
      "wrong project reply",
      (f) => {
        record(f.server.find((row) => row.id === 3 && "result" in row)!.result).configFilePath =
          "/tmp/unrelated/tsconfig.json";
      },
    ],
    [
      "foreign glob",
      (f) => {
        watcher(f).globPattern = "/tmp/**/*";
      },
    ],
    [
      "wrong watcher kind",
      (f) => {
        watcher(f).kind = 1;
      },
    ],
    [
      "different watcher",
      (f) => {
        registration(f).id = String(registration(f).id).replace("watcher 7.0", "watcher 8.0");
      },
    ],
    [
      "extra registration field",
      (f) => {
        registration(f).other = true;
      },
    ],
    [
      "multiple watchers",
      (f) => {
        (record(registration(f).registerOptions).watchers as unknown[]).push({ ...watcher(f) });
      },
    ],
    [
      "pre-shutdown request",
      (f) => {
        const request = program(f);
        f.server.splice(f.server.indexOf(request), 1);
        f.server.unshift(request);
      },
    ],
    [
      "missing shutdown ACK",
      (f) => {
        f.server = f.server.filter((row) => !(row.id === 4 && "result" in row));
      },
    ],
    [
      "error shutdown ACK",
      (f) => {
        f.server.find((row) => row.id === 4 && "result" in row)!.error = { code: -1 };
      },
    ],
    [
      "unanswered root watcher",
      (f) => {
        f.client = f.client.filter((row) => row.id !== "ts2");
      },
    ],
    [
      "extra unmatched request",
      (f) => {
        f.server.push({ jsonrpc: "2.0", id: "ts999", method: "workspace/other" });
      },
    ],
    [
      "reply without request",
      (f) => {
        f.client.push({ jsonrpc: "2.0", id: "other", result: null });
      },
    ],
    [
      "duplicate reply",
      (f) => {
        f.client.push({ ...f.client.find((row) => row.id === "ts2")! });
      },
    ],
    [
      "duplicate request",
      (f) => {
        f.server.push({ ...program(f) });
      },
    ],
  ];
  for (const [name, mutate] of mutations) {
    const f = fixture();
    mutate(f);
    assert.throws(() => verifyNativeRelations(f.client, f.server, f.context), name);
  }
});

test("fully replied channels need no baseline exception in any role", () => {
  for (const [role, sdkVersion] of [
    ["before", "7.0.2"],
    ["after", "7.0.2"],
    ["before", "7.1.0-beta"],
    ["after", "7.1.0-beta"],
  ]) {
    const f = fixture();
    f.context.role = role!;
    f.context.sdkVersion = sdkVersion!;
    f.client.push({ jsonrpc: "2.0", id: "ts3", result: null });
    assert.equal(verifyNativeRelations(f.client, f.server, f.context), null);
  }
});
