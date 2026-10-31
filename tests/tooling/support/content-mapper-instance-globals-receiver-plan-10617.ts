import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import {
  assertHostCase,
  assertMapperCase,
  type AdditionalHost,
} from "./content-mapper-instance-globals-hosts-10617.ts";
import {
  assertReceiverNativeCase,
  hasReceiverNativeExpectation,
} from "./content-mapper-instance-globals-receiver-native-10617.ts";
import {
  assertReceiverSymbols,
  type ReceiverProgram,
} from "./content-mapper-instance-globals-receiver-symbols-10617.ts";
import {
  assertProtocolAdmission,
  assertTransformEnvelope,
  decodeFrames,
  frame,
  request,
  sha,
  type Packet,
} from "./content-mapper-instance-globals-10617.ts";
type Run = (label: string, command: string, args: string[], cwd: string, input?: Buffer) => Packet;
type Independent = {
  id: string;
  host: string;
  status: number;
  signal: string | null;
  stdout: string;
  stderr: string;
};
const exactProcess = (packet: Packet | Independent) => ({
  status: packet.status,
  signal: packet.signal,
  stdout: packet.stdout,
  stderr: packet.stderr,
});

export function runReceiverPlanControls(
  files: Map<string, string>,
  source: string,
  published: string,
  output: string,
  binary: string,
  publicCli: string,
  beta: string,
  ts5: string,
  additionalHosts: AdditionalHost[],
  run: Run,
) {
  const data = JSON.parse(files.get("receiver-plan-controls.json")!) as {
    schema: string;
    version: number;
    inventory: { programs: number };
    programs: ReceiverProgram[];
  };
  assert.equal(data.schema, "vize.instance-global-receiver-plan-controls");
  assert.equal(data.version, 1);
  assert.deepEqual(data.inventory, { programs: 4 });
  assert.equal(data.programs.length, 4);
  assert.equal(new Set(data.programs.map((row) => sha(row.source))).size, 4);
  const frozen = JSON.parse(files.get("receiver-plan-independent-packets.json")!) as Independent[];
  assert.equal(frozen.length, 8);
  const root = path.join(output, "receiver-plan-controls");
  fs.mkdirSync(root, { recursive: true });
  const checks: Array<{ id: string; run: () => void }> = [];
  const wires = (handle: string, directory: string) => [
    frame(request(1, "initialize", { positionEncodings: ["utf-8"] })),
    frame(
      request(2, "openProject", {
        projectHandle: handle,
        configFileName: path.join(directory, "tsconfig.json"),
        compilerOptions: JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
      }),
    ),
  ];
  const sourceWire = wires("receiver-plan-source", source),
    publicWire = wires("receiver-plan-public", published);
  const inventory = [];
  for (const [index, entry] of data.programs.entries()) {
    const id = index + 3,
      label = `receiver-plan/${entry.id}`;
    for (const directory of [source, published]) {
      fs.writeFileSync(path.join(directory, "src/Globals.vue"), entry.source);
      fs.writeFileSync(path.join(directory, "src/globals.d.ts"), entry.globals);
      fs.writeFileSync(path.join(directory, "oracle/Context.ts"), entry.oracle);
    }
    const stageRoot = path.join(root, entry.id);
    fs.mkdirSync(stageRoot, { recursive: true });
    for (const [name, text] of [
      ["Globals.vue", entry.source],
      ["globals.d.ts", entry.globals],
      ["Context.ts", entry.oracle],
    ])
      fs.writeFileSync(path.join(stageRoot, `${name}.txt`), text);
    const reference = run(
      `${label}/independent-5.9-context`,
      process.execPath,
      [ts5, "--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
      source,
    );
    const expectedReference = frozen.find((row) => row.id === entry.id && row.host === "ts5");
    assert.ok(expectedReference);
    checks.push({
      id: `${label}/frozen-independent-5.9`,
      run() {
        assert.deepEqual(exactProcess(reference), exactProcess(expectedReference));
      },
    });
    for (const host of [{ id: "reported-beta", command: beta }, ...additionalHosts]) {
      const hostLabel = `${label}/${host.id}`;
      const context = run(
        `${hostLabel}/independent-context`,
        host.command,
        ["--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
        source,
      );
      run(
        `${hostLabel}/published-official-host`,
        host.command,
        ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
        published,
      );
      const after = run(
        `${hostLabel}/source-official-host`,
        host.command,
        ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
        source,
      );
      const cli = run(
        `${hostLabel}/source-check`,
        binary,
        [
          "check",
          "src",
          "--tsconfig",
          "tsconfig.json",
          "--format",
          "json",
          "--quiet",
          "--corsa-path",
          host.command,
        ],
        source,
      );
      checks.push({
        id: hostLabel,
        run() {
          if (host.id === "reported-beta") {
            const expected = frozen.find((row) => row.id === entry.id && row.host === "beta");
            assert.ok(expected);
            assert.deepEqual(exactProcess(context), exactProcess(expected));
          }
          const compilerOptions = JSON.parse(
            files.get("project/tsconfig.json.txt")!,
          ).compilerOptions;
          if (hasReceiverNativeExpectation(entry.id)) {
            assertMapperCase(entry.source, entry, context, after, hostLabel);
            assertReceiverNativeCase(entry.source, entry, context, cli, compilerOptions, hostLabel);
          } else {
            assertHostCase(entry.source, entry, context, after, cli, compilerOptions, hostLabel);
          }
        },
      });
    }
    sourceWire.push(
      frame(
        request(id, "transform", {
          projectHandle: "receiver-plan-source",
          fileName: path.join(source, "src/Globals.vue"),
          content: entry.source,
        }),
      ),
    );
    publicWire.push(
      frame(
        request(id, "transform", {
          projectHandle: "receiver-plan-public",
          fileName: path.join(published, "src/Globals.vue"),
          content: entry.source,
        }),
      ),
    );
    inventory.push({
      id,
      program: entry.id,
      sourceSha256: sha(entry.source),
      globalsSha256: sha(entry.globals),
      oracleSha256: sha(entry.oracle),
      receiver: entry.receiver,
      candidate: entry.candidate,
    });
  }
  const publicPacket = run(
    "receiver-plan/published/complete-protocol",
    publicCli,
    ["content-mapper"],
    published,
    Buffer.concat(publicWire),
  );
  const sourcePacket = run(
    "receiver-plan/source/complete-protocol",
    binary,
    ["content-mapper"],
    source,
    Buffer.concat(sourceWire),
  );
  const responses = decodeFrames(sourcePacket.stdout);
  fs.writeFileSync(
    path.join(root, "source-protocol.json"),
    `${JSON.stringify(responses, null, 2)}\n`,
  );
  fs.writeFileSync(
    path.join(root, "published-protocol.json"),
    `${JSON.stringify(decodeFrames(publicPacket.stdout), null, 2)}\n`,
  );
  fs.writeFileSync(
    path.join(root, "inventory.json"),
    `${JSON.stringify({ inventory, actualHosts: ["reported-beta", ...additionalHosts.map((row) => row.id)], credit: "Unqualified until actual semantic and symbol-chain controls pass; independent packets were frozen before source execution" }, null, 2)}\n`,
  );
  checks.push({
    id: "receiver-plan/closed-protocol",
    run() {
      for (const packet of [sourcePacket, publicPacket])
        assert.deepEqual(
          {
            status: packet.status,
            signal: packet.signal,
            error: packet.error,
            stderr: packet.stderr,
          },
          { status: 0, signal: null, error: null, stderr: "" },
        );
      assert.equal(responses.length, 6);
      assertProtocolAdmission(responses, path.join(source, "tsconfig.json"));
      assertProtocolAdmission(
        decodeFrames(publicPacket.stdout),
        path.join(published, "tsconfig.json"),
      );
    },
  });
  const facts: unknown[] = [];
  for (const [index, entry] of data.programs.entries())
    checks.push({
      id: `receiver-plan/${entry.id}/symbols-and-geometry`,
      run() {
        const response = responses.find((row) => row.id === index + 3);
        assert.ok(response);
        assertTransformEnvelope(response, index + 3);
        facts.push({
          id: entry.id,
          proof: assertReceiverSymbols(
            entry,
            response.result!,
            source,
            path.join(output, "reference/package.json"),
          ),
        });
        fs.writeFileSync(
          path.join(root, "symbol-chains.json"),
          `${JSON.stringify(facts, null, 2)}\n`,
        );
      },
    });
  return checks;
}
