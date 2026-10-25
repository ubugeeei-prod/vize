import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { test } from "node:test";
import { repoRoot } from "../_helpers/realworld-patch.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import {
  admitAdditionalHosts,
  assertHostCase,
  assertPublishedControl,
} from "./support/content-mapper-instance-globals-hosts-10617.ts";
import {
  assertErrorMappings,
  assertCheckedReadOwnership,
  assertProtocolAdmission,
  assertWholeTransform,
  assertTransformEnvelope,
  fullPacketMutations,
  type PacketGoldens,
  compilerRows,
  decodeFrames,
  expectedHost,
  frame,
  location,
  makeSourceProject,
  readGlobalsFixture,
  request,
  sha,
  writeInputs,
  type Packet,
} from "./support/content-mapper-instance-globals-10617.ts";

test("10617 retains the original reported files, exact beta host and closed framing", () => {
  const { files, cases } = readGlobalsFixture();
  assert.equal(
    cases.find(({ id }) => id === "original")!.source,
    files.get("project/src/Globals.vue.txt"),
  );
  assert.equal(
    cases.find(({ id }) => id === "original")!.globals,
    files.get("project/src/globals.d.ts.txt"),
  );
  assert.equal(cases.length, 40);
  const packet = { jsonrpc: "2.0", id: 1, result: { text: "日本語🍣\r\n", mappings: [] } };
  assert.deepEqual(decodeFrames(frame(packet).toString()), [packet]);
  assert.throws(() => decodeFrames(frame(packet).subarray(0, -1).toString()));
  if (files.has("source-packets.json")) {
    const golden = JSON.parse(files.get("source-packets.json")!) as PacketGoldens;
    assert.equal(golden.schema, "vize.instance-global-complete-transforms");
    fullPacketMutations(
      golden.packets[0].result,
      golden.packets.find((row) => row.label === "lf/loop-shadow")!.result,
    );
    const directivePacket = golden.packets.find((row) => "diagnosticDirectives" in row.result)!;
    fullPacketMutations(directivePacket.result, golden.packets[0].result);
    const response = { id: 3, jsonrpc: "2.0", result: golden.packets[0].result };
    assertTransformEnvelope(response, 3);
    assert.throws(() => assertTransformEnvelope({ ...response, id: 4 }, 3));
    assert.throws(() => assertTransformEnvelope({ ...response, error: null }, 3));
    assert.throws(() => assertTransformEnvelope({ ...response, jsonrpc: "1.0" }, 3));
  }
});

await test(
  "real beta mapper rejects undeclared instance globals without widening declared or lexical owners",
  {
    skip:
      !process.env.VIZE_TEST_BIN &&
      !process.env.VIZE_LSP_BIN &&
      process.env.VIZE_TEST_GLOBALS_REQUIRED !== "1" &&
      process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD !== "1"
        ? "requires the exact-source consumer campaign"
        : false,
    timeout: 900_000,
  },
  async () => {
    const output =
      process.env.VIZE_TEST_GLOBALS_OUTPUT ??
      path.join(repoRoot, "target/differential/instance-globals-10617");
    fs.mkdirSync(output, { recursive: true });
    const requestedBinary = process.env.VIZE_TEST_BIN ?? process.env.VIZE_LSP_BIN;
    assert.ok(requestedBinary, "source CLI required; no fallback or silent acceptance");
    const [binary] = resolveVizeLaunchCommand(undefined, requestedBinary, {
      required: true,
      repoRoot,
      onVerifiedLaunch(launch) {
        fs.writeFileSync(
          path.join(output, "source-launch.json"),
          `${JSON.stringify(launch, null, 2)}\n`,
        );
      },
    });
    const { authority, files, cases } = readGlobalsFixture();
    const golden = files.has("source-packets.json")
      ? (JSON.parse(files.get("source-packets.json")!) as PacketGoldens)
      : undefined;
    const published = path.join(output, "published"),
      reference = path.join(output, "reference"),
      source = path.join(output, "source");
    writeInputs(published, files);
    fs.mkdirSync(reference, { recursive: true });
    for (const name of ["package.json", "package-lock.json"])
      fs.writeFileSync(path.join(reference, name), files.get(`reference/${name}.txt`)!);
    let additionalHostRoot: string | undefined;
    const records: Packet[] = [],
      checks: Array<{ id: string; run: () => string | void }> = [],
      sizes: unknown[] = [],
      discovery: unknown[] = [];
    const run = (label: string, command: string, args: string[], cwd: string, input?: Buffer) => {
      const started = performance.now();
      const raw = spawnSync(command, args, {
        cwd,
        input,
        encoding: "utf8",
        timeout: 120_000,
        maxBuffer: 32 * 1024 * 1024,
        env: { ...process.env, LANG: "C", LC_ALL: "C" },
      });
      const packet: Packet = {
        label,
        command,
        args,
        cwd,
        status: raw.status,
        signal: raw.signal,
        error: raw.error?.message ?? null,
        stdout: raw.stdout ?? "",
        stderr: raw.stderr ?? "",
        elapsedMs: performance.now() - started,
      };
      records.push(packet);
      fs.writeFileSync(
        path.join(output, "processes.json"),
        `${JSON.stringify(records, null, 2)}\n`,
      );
      assert.equal(packet.error, null, label);
      assert.equal(packet.signal, null, label);
      return packet;
    };
    try {
      for (const root of [published, reference])
        assert.equal(
          run(
            `${path.basename(root)}/locked-install`,
            process.platform === "win32" ? "npm.cmd" : "npm",
            [
              "ci",
              "--ignore-scripts",
              "--no-audit",
              "--no-fund",
              "--prefer-offline",
              "--registry=https://registry.npmjs.org",
            ],
            root,
          ).status,
          0,
        );
      const publicCli = path.join(published, "node_modules/vize/bin/vize"),
        host = path.join(published, "node_modules/typescript/bin/tsc"),
        ts5 = path.join(reference, "node_modules/typescript/bin/tsc");
      assert.equal(
        run("official-beta-version", host, ["--version"], published).stdout,
        "Version 7.1.0-beta\n",
      );
      assert.equal(
        run("published-vize-version", publicCli, ["--version"], published).stdout,
        "vize 0.566.0\n",
      );
      const hostMatrix = admitAdditionalHosts(output, files, published, repoRoot, run);
      additionalHostRoot = hostMatrix.cleanupRoot;
      const identity = makeSourceProject(source, published, repoRoot, binary, files);
      fs.writeFileSync(
        path.join(output, "source-adapter.json"),
        `${JSON.stringify({ authority, ...identity }, null, 2)}\n`,
      );
      for (const root of [published, source]) {
        fs.mkdirSync(path.join(root, "oracle"), { recursive: true });
        const config = JSON.parse(files.get("project/tsconfig.json.txt")!);
        delete config.contentMappers;
        delete config.include;
        config.files = ["src/globals.d.ts", "oracle/Context.ts"];
        fs.writeFileSync(path.join(root, "oracle.tsconfig.json"), JSON.stringify(config));
      }
      const publicWires: Buffer[] = [
        frame(request(1, "initialize", { positionEncodings: ["utf-8"] })),
        frame(
          request(2, "openProject", {
            projectHandle: "published",
            configFileName: path.join(published, "tsconfig.json"),
            compilerOptions: JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
          }),
        ),
      ];
      const faithful: Array<() => void> = [];
      const wires: Buffer[] = [
        frame(request(1, "initialize", { positionEncodings: ["utf-8"] })),
        frame(
          request(2, "openProject", {
            projectHandle: "source",
            configFileName: path.join(source, "tsconfig.json"),
            compilerOptions: JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
          }),
        ),
      ];
      const sourceCases: Array<{
        id: number;
        label: string;
        source: string;
        entry: (typeof cases)[number];
      }> = [];
      let id = 3;
      for (const [variant, newline, unicode] of [
        ["lf", "\n", false],
        ["crlf-unicode", "\r\n", true],
      ] as const) {
        for (const entry of cases) {
          const label = `${variant}/${entry.id}`;
          const content = (
            unicode ? entry.source.replace("{{ ", '{{ "日本語🍣" + ') : entry.source
          ).replaceAll("\n", newline);
          for (const root of [published, source]) {
            fs.writeFileSync(path.join(root, "src/Globals.vue"), content);
            fs.writeFileSync(path.join(root, "src/globals.d.ts"), entry.globals);
            fs.writeFileSync(path.join(root, "oracle/Context.ts"), entry.oracle);
          }
          fs.writeFileSync(path.join(output, `${variant}-${entry.id}.vue.txt`), content);
          const beta = run(
            `${label}/independent-beta-context`,
            host,
            ["--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
            source,
          );
          const independent = run(
            `${label}/independent-5.9-context`,
            process.execPath,
            [ts5, "--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
            source,
          );
          const before = run(
            `${label}/published-official-host`,
            host,
            ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
            published,
          );
          const after = run(
            `${label}/source-official-host`,
            host,
            ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
            source,
          );
          const checkArgs = [
            "check",
            "src",
            "--tsconfig",
            "tsconfig.json",
            "--format",
            "json",
            "--quiet",
            "--corsa-path",
            host,
          ];
          if (label === "lf/original")
            checkArgs.push("--profile-json", path.join(output, "source-profile.json"));
          const cli = run(`${label}/source-check`, binary, checkArgs, source);
          if (label === "lf/original")
            run(
              "reported-sfc-reference-unqualified",
              process.execPath,
              [
                path.join(reference, "node_modules/vue-tsc/bin/vue-tsc.js"),
                "--noEmit",
                "--pretty",
                "false",
                "-p",
                "tsconfig.json",
              ],
              published,
            );
          wires.push(
            frame(
              request(id, "transform", {
                projectHandle: "source",
                fileName: path.join(source, "src/Globals.vue"),
                content,
              }),
            ),
          );
          publicWires.push(
            frame(
              request(id, "transform", {
                projectHandle: "published",
                fileName: path.join(published, "src/Globals.vue"),
                content,
              }),
            ),
          );
          sourceCases.push({ id: id++, source: content, entry, label });
          if (["original", "valid", "undeclared-only", "repaired"].includes(entry.id))
            faithful.push(() => assertPublishedControl(content, entry, beta, before, label));
          for (const additional of hostMatrix.hosts) {
            const hostLabel = `${label}/${additional.id}`;
            const context = run(
              `${hostLabel}/independent-context`,
              additional.command,
              ["--noEmit", "--pretty", "false", "-p", "oracle.tsconfig.json"],
              source,
            );
            const publicPacket = run(
              `${hostLabel}/published-official-host`,
              additional.command,
              ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
              published,
            );
            const sourcePacket = run(
              `${hostLabel}/source-official-host`,
              additional.command,
              ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"],
              source,
            );
            const args = [
              "check",
              "src",
              "--tsconfig",
              "tsconfig.json",
              "--format",
              "json",
              "--quiet",
              "--corsa-path",
              additional.command,
            ];
            if (label === "lf/original")
              args.push(
                "--profile-json",
                path.join(output, `${additional.id}-source-profile.json`),
              );
            const native = run(`${hostLabel}/source-check`, binary, args, source);
            checks.push({
              id: hostLabel,
              run() {
                if (entry.discovery) {
                  discovery.push({
                    label: hostLabel,
                    scope: "Options API own method discovery",
                    before: publicPacket,
                    after: sourcePacket,
                    cli: native,
                  });
                  if (publicPacket.status === 0 && sourcePacket.status !== 0)
                    throw new Error("new Options API own-dollar regression");
                  return "discovery-unqualified";
                }
                assertHostCase(
                  content,
                  entry,
                  context,
                  sourcePacket,
                  native,
                  JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
                  hostLabel,
                );
              },
            });
            if (["original", "valid", "undeclared-only", "repaired"].includes(entry.id))
              faithful.push(() =>
                assertPublishedControl(content, entry, context, publicPacket, hostLabel),
              );
          }
          checks.push({
            id: label,
            run() {
              const rows = compilerRows(beta.stdout);
              assert.equal(beta.status, rows.length ? 2 : 0, label);
              assert.equal(beta.stderr, "", label);
              assert.equal(independent.status, beta.status, label);
              assert.equal(
                independent.stdout,
                beta.stdout,
                `full independent context messages ${label}`,
              );
              assert.equal(independent.stderr, beta.stderr, label);
              const expected = expectedHost(content, entry, rows);
              if (entry.discovery) {
                discovery.push({
                  label,
                  scope: "Options API own method discovery",
                  before,
                  after,
                  cli,
                  expected,
                });
                if (before.status === 0 && after.status !== 0)
                  throw new Error("new Options API own-dollar regression");
                return "discovery-unqualified";
              }
              assertHostCase(
                content,
                entry,
                beta,
                after,
                cli,
                JSON.parse(files.get("project/tsconfig.json.txt")!).compilerOptions,
                label,
              );
            },
          });
        }
      }
      const publicWire = run(
        "published/same-process-complete-protocol",
        publicCli,
        ["content-mapper"],
        published,
        Buffer.concat(publicWires),
      );
      fs.writeFileSync(
        path.join(output, "published-protocol.json"),
        `${JSON.stringify(decodeFrames(publicWire.stdout), null, 2)}\n`,
      );
      const wire = run(
        "source/same-process-complete-protocol",
        binary,
        ["content-mapper"],
        source,
        Buffer.concat(wires),
      );
      assert.equal(wire.status, 0);
      assert.equal(wire.stderr, "");
      const responses = decodeFrames(wire.stdout);
      fs.writeFileSync(
        path.join(output, "source-protocol.json"),
        `${JSON.stringify(responses, null, 2)}\n`,
      );
      assert.deepEqual(
        responses.map(({ jsonrpc, id: responseId, error }) => [jsonrpc, responseId, error ?? null]),
        Array.from({ length: id - 1 }, (_, index) => ["2.0", index + 1, null]),
      );
      assertProtocolAdmission(responses, path.join(source, "tsconfig.json"));
      assertProtocolAdmission(
        decodeFrames(publicWire.stdout),
        path.join(published, "tsconfig.json"),
      );
      for (const row of sourceCases) {
        const response = responses.find((packet) => packet.id === row.id)!;
        assertTransformEnvelope(response, row.id);
        const result = response.result!;
        checks.push({
          id: `protocol/${row.id}/${row.entry.id}`,
          run() {
            assertErrorMappings(row.source, row.entry, result);
            assertCheckedReadOwnership(
              row.source,
              row.entry,
              result,
              path.join(reference, "package.json"),
            );
          },
        });
        checks.push({
          id: `whole-packet/${row.label}`,
          run() {
            const expected = golden?.packets.find((item) => item.label === row.label);
            assert.ok(expected, `complete transform golden required for ${row.label}`);
            assert.equal(expected.sourceSha256, sha(row.source));
            assert.equal(expected.globalsSha256, sha(row.entry.globals));
            assertWholeTransform(result, expected.result);
          },
        });
        sizes.push({
          id: row.id,
          case: row.entry.id,
          sourceBytes: Buffer.byteLength(row.source),
          generatedBytes: Buffer.byteLength(result.text),
          mappingCount: result.mappings.length,
          generatedSha256: sha(result.text),
        });
      }
      const first = sourceCases.find((row) => row.entry.id === "valid")!,
        repair = sourceCases.find((row) => row.entry.id === "repaired")!;
      assert.deepEqual(
        responses.find((row) => row.id === first.id)!.result,
        responses.find((row) => row.id === repair.id)!.result,
        "whole generated packet valid/repair determinism",
      );
      fs.writeFileSync(path.join(output, "sizes.json"), `${JSON.stringify(sizes, null, 2)}\n`);
      const outcomes = checks.map(({ id: label, run: check }) => {
        try {
          const state = check();
          return { id: label, outcome: state ?? "pass" };
        } catch (error) {
          return { id: label, outcome: "gap", detail: String(error) };
        }
      });
      fs.writeFileSync(
        path.join(output, "outcomes.json"),
        `${JSON.stringify(outcomes, null, 2)}\n`,
      );
      fs.writeFileSync(
        path.join(output, "discovery.json"),
        `${JSON.stringify(discovery, null, 2)}\n`,
      );
      assert.ok(
        fs.existsSync(path.join(output, "source-profile.json")),
        "actual source instrumentation must be retained",
      );
      for (const check of faithful) check();
      const discoveryOnly = process.env.VIZE_TEST_GLOBALS_DISCOVERY === "1";
      const failures = outcomes.filter(({ outcome }) => outcome === "gap");
      const wholePacketQualification =
        !discoveryOnly &&
        golden?.complete === true &&
        golden.packets.length === sourceCases.length &&
        failures.length === 0;
      const hostMatrixQualification = wholePacketQualification && hostMatrix.sourceHostAdmitted;
      fs.writeFileSync(
        path.join(output, "qualification.json"),
        `${JSON.stringify({ status: discoveryOnly ? "discovery-only; no acceptance credit" : wholePacketQualification ? "qualified" : "unqualified", fullPacketQualification: wholePacketQualification, hostMatrixQualification, hostMatrixRequired: hostMatrix.required, outcomes }, null, 2)}\n`,
      );
      if (!discoveryOnly) {
        if (hostMatrix.required)
          assert.equal(hostMatrixQualification, true, "all three real hosts must qualify");
        assert.equal(
          wholePacketQualification,
          true,
          "all complete transform goldens, semantic and mapping proofs are required",
        );
        assert.deepEqual(failures, [], "whole beta/native/protocol gaps; raw packets retained");
      }
    } finally {
      for (const root of [
        published,
        reference,
        source,
        ...(additionalHostRoot ? [additionalHostRoot] : []),
      ])
        fs.rmSync(path.join(root, "node_modules"), { recursive: true, force: true });
    }
  },
);
