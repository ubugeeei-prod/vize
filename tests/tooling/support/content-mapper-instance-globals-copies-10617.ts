import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import {
  assertHostCase,
  type AdditionalHost,
} from "./content-mapper-instance-globals-hosts-10617.ts";
import {
  assertErrorMappings,
  assertProtocolAdmission,
  assertTransformEnvelope,
  decodeFrames,
  frame,
  instanceReceiverReads,
  request,
  sha,
  type GlobalCase,
  type Packet,
} from "./content-mapper-instance-globals-10617.ts";

type AuxProgram = GlobalCase & { binder?: { source: string; normalized: string } };
type AuxStage = { label: string; program: AuxProgram; id: number };
type AuxRun = (
  label: string,
  command: string,
  args: string[],
  cwd: string,
  input?: Buffer,
) => Packet;

export function runCopyAndCollisionControls(
  files: Map<string, string>,
  source: string,
  published: string,
  output: string,
  binary: string,
  publicCli: string,
  beta: string,
  ts5: string,
  additionalHosts: AdditionalHost[],
  run: AuxRun,
) {
  const copy = JSON.parse(files.get("copy-controls.json")!) as {
    schema: string;
    inventory: { families: number; uniqueAuthoredPrograms: number; stages: number };
    programs: AuxProgram[];
    families: Array<{ id: string; stages: Array<{ id: string; program: string }> }>;
  };
  const collisions = JSON.parse(files.get("collision-controls.json")!) as {
    schema: string;
    inventory: { probes: number };
    programs: AuxProgram[];
  };
  assert.equal(copy.schema, "vize.instance-global-copy-controls");
  assert.equal(collisions.schema, "vize.instance-global-collision-controls");
  assert.deepEqual(copy.inventory, { families: 6, uniqueAuthoredPrograms: 10, stages: 14 });
  assert.equal(copy.programs.length, 10);
  assert.equal(new Set(copy.programs.map((row) => row.id)).size, 10);
  assert.equal(new Set(copy.programs.map((row) => sha(row.source))).size, 10);
  assert.equal(copy.families.length, 6);
  assert.equal(collisions.programs.length, 3);
  assert.equal(new Set(collisions.programs.map((row) => sha(row.source))).size, 3);
  assert.equal(collisions.inventory.probes, 3);
  const root = path.join(output, "auxiliary-controls");
  fs.mkdirSync(root, { recursive: true });
  const stages: AuxStage[] = [];
  let id = 3;
  for (const family of copy.families)
    for (const stage of family.stages) {
      const program = copy.programs.find((row) => row.id === stage.program);
      assert.ok(program);
      stages.push({ label: `copy/${family.id}/${stage.id}`, program, id: id++ });
    }
  assert.equal(stages.length, 14);
  for (const program of collisions.programs)
    stages.push({ label: `collision/${program.id}`, program, id: id++ });
  const checks: Array<{ id: string; run: () => void }> = [];
  const sourceWire = [
    frame(request(1, "initialize", { positionEncodings: ["utf-8"] })),
    frame(
      request(2, "openProject", {
        projectHandle: "aux-source",
        configFileName: path.join(source, "tsconfig.json"),
        compilerOptions: JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
      }),
    ),
  ];
  const publicWire = [
    frame(request(1, "initialize", { positionEncodings: ["utf-8"] })),
    frame(
      request(2, "openProject", {
        projectHandle: "aux-public",
        configFileName: path.join(published, "tsconfig.json"),
        compilerOptions: JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
      }),
    ),
  ];
  const hosts = [{ id: "reported-beta", command: beta }, ...additionalHosts];
  const facts: unknown[] = [];
  for (const stage of stages) {
    const { program: entry, label } = stage;
    for (const directory of [source, published]) {
      fs.writeFileSync(path.join(directory, "src/Globals.vue"), entry.source);
      fs.writeFileSync(path.join(directory, "src/globals.d.ts"), entry.globals);
      fs.writeFileSync(path.join(directory, "oracle/Context.ts"), entry.oracle);
    }
    const stageRoot = path.join(root, label);
    fs.mkdirSync(stageRoot, { recursive: true });
    for (const [name, content] of [
      ["Globals.vue", entry.source],
      ["globals.d.ts", entry.globals],
      ["Context.ts", entry.oracle],
    ])
      fs.writeFileSync(path.join(stageRoot, `${name}.txt`), content);
    const reference = run(
      `aux/${label}/independent-5.9-context`,
      process.execPath,
      [ts5, "--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
      source,
    );
    for (const host of hosts) {
      const hostLabel = `aux/${label}/${host.id}`;
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
          if (host.id === "reported-beta")
            assert.deepEqual(
              {
                status: reference.status,
                signal: reference.signal,
                error: reference.error,
                stdout: reference.stdout,
                stderr: reference.stderr,
              },
              {
                status: context.status,
                signal: context.signal,
                error: context.error,
                stdout: context.stdout,
                stderr: context.stderr,
              },
              "independent compiler packets are retained whole",
            );
          assertHostCase(
            entry.source,
            entry,
            context,
            after,
            cli,
            JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
            hostLabel,
          );
        },
      });
    }
    sourceWire.push(
      frame(
        request(stage.id, "transform", {
          projectHandle: "aux-source",
          fileName: path.join(source, "src/Globals.vue"),
          content: entry.source,
        }),
      ),
    );
    publicWire.push(
      frame(
        request(stage.id, "transform", {
          projectHandle: "aux-public",
          fileName: path.join(published, "src/Globals.vue"),
          content: entry.source,
        }),
      ),
    );
    facts.push({
      label,
      program: entry.id,
      sourceSha256: sha(entry.source),
      globalsSha256: sha(entry.globals),
      oracleSha256: sha(entry.oracle),
      binder: entry.binder,
      credit: "discovery only; no full copy-ownership acceptance credit",
    });
  }
  const publicPacket = run(
    "aux/published/complete-protocol",
    publicCli,
    ["content-mapper"],
    published,
    Buffer.concat(publicWire),
  );
  const sourcePacket = run(
    "aux/source/complete-protocol",
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
    `${JSON.stringify({ copy: copy.inventory, collisions: collisions.inventory, totalStages: stages.length, facts, qualification: "discovery only; whole raw packets retained pending source ownership qualification" }, null, 2)}\n`,
  );
  checks.push({
    id: "aux/closed-protocol",
    run() {
      assert.deepEqual(
        {
          status: sourcePacket.status,
          signal: sourcePacket.signal,
          error: sourcePacket.error,
          stderr: sourcePacket.stderr,
        },
        { status: 0, signal: null, error: null, stderr: "" },
      );
      assert.deepEqual(
        {
          status: publicPacket.status,
          signal: publicPacket.signal,
          error: publicPacket.error,
          stderr: publicPacket.stderr,
        },
        { status: 0, signal: null, error: null, stderr: "" },
      );
      assert.equal(responses.length, stages.length + 2);
      assertProtocolAdmission(responses, path.join(source, "tsconfig.json"));
      assertProtocolAdmission(
        decodeFrames(publicPacket.stdout),
        path.join(published, "tsconfig.json"),
      );
    },
  });
  const mappedReads: unknown[] = [];
  for (const stage of stages) {
    const response = responses.find((row) => row.id === stage.id);
    checks.push({
      id: `aux/protocol/${stage.label}`,
      run() {
        assert.ok(response);
        assertTransformEnvelope(response, stage.id);
        const binderProof = stage.program.binder
          ? {
              oracle: escapedBinderFact(
                stage.program.oracle,
                stage.program.binder,
                path.join(output, "reference/package.json"),
              ),
              generated: escapedBinderFact(
                response.result!.text,
                stage.program.binder,
                path.join(output, "reference/package.json"),
              ),
            }
          : undefined;
        mappedReads.push({
          label: stage.label,
          reads: instanceReceiverReads(
            response.result!,
            path.join(output, "reference/package.json"),
          ),
          mappings: response.result!.mappings,
        });
        fs.writeFileSync(
          path.join(root, "mapped-reads.json"),
          `${JSON.stringify(mappedReads, null, 2)}\n`,
        );
      },
    });
  }
  for (const family of copy.families) {
    const valid = stages.find((row) => row.label === `copy/${family.id}/valid`);
    const repaired = stages.find((row) => row.label === `copy/${family.id}/repaired`);
    if (valid && repaired)
      checks.push({
        id: `aux/${family.id}/whole-repair-determinism`,
        run() {
          assert.deepEqual(
            responses.find((row) => row.id === valid.id)!.result,
            responses.find((row) => row.id === repaired.id)!.result,
          );
        },
      });
  }
  return checks;
}

// Ask the pinned parser to normalize escaped parameter names. No private decoder.
function escapedBinderFact(
  text: string,
  binder: { source: string; normalized: string },
  referencePackage: string,
) {
  type Node = {
    end: number;
    text?: string;
    parameters?: Array<{ name: Node }>;
    getStart(file: Node): number;
  };
  const ts = createRequire(referencePackage)("typescript") as {
    version: string;
    ScriptTarget: { Latest: number };
    ScriptKind: { TSX: number };
    createSourceFile(
      name: string,
      text: string,
      target: number,
      parents: boolean,
      kind: number,
    ): Node;
    isArrowFunction(node: Node): boolean;
    isIdentifier(node: Node): boolean;
    forEachChild(node: Node, visit: (node: Node) => void): void;
  };
  assert.equal(ts.version, "5.9.3");
  const file = ts.createSourceFile(
    "escaped-binder.tsx",
    text,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TSX,
  );
  const facts: Array<{ raw: string; normalized: string }> = [];
  const visit = (node: Node) => {
    if (ts.isArrowFunction(node))
      for (const parameter of node.parameters ?? [])
        if (ts.isIdentifier(parameter.name) && parameter.name.text === binder.normalized)
          facts.push({
            raw: text.slice(parameter.name.getStart(file), parameter.name.end),
            normalized: parameter.name.text,
          });
    ts.forEachChild(node, visit);
  };
  visit(file);
  assert.deepEqual(facts, [{ raw: binder.source, normalized: binder.normalized }]);
  return { compiler: ts.version, facts };
}
