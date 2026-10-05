import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { LspSession } from "../../tooling/support/lsp/session.ts";
import { decodeFrames } from "../../differential/lsp-wire.ts";
import {
  awaitRetired,
  nativeIdentities,
  type ProcessIdentity,
} from "./warm-type-backed-processes.ts";
import { controls } from "./warm-type-backed-controls.ts";
import { QueryRecorder } from "./warm-type-backed-measure.ts";
import {
  assertTypedPackets,
  finishWire,
  requests,
  waitForInitialTypes,
} from "./warm-type-backed-packets.ts";
import {
  driverRoot,
  generateWorkspace,
  inputAuthority,
  runtimeIdentity,
  sourceIdentity,
} from "./warm-type-backed-source.ts";

async function runSide(
  repoRoot: string,
  workspace: string,
  runtime: ReturnType<typeof runtimeIdentity>,
  output: string,
) {
  const binary = path.join(repoRoot, "target/ci/vize");
  const captureRoot = path.join(repoRoot, "target/differential/lsp-sessions");
  fs.mkdirSync(captureRoot, { recursive: true });
  const previous = fs.readdirSync(captureRoot);
  const inputs = generateWorkspace(workspace, runtime);
  const source = inputs.source["src/c/Comp0.vue"];
  const uri = pathToFileURL(path.join(workspace, "src/c/Comp0.vue")).href;
  let session: LspSession | undefined;
  let recorder: QueryRecorder | undefined;
  let processId: number | null = null;
  let retirement: unknown;
  const natives = new Map<string, ProcessIdentity>();
  const failures: string[] = [];
  let initialization: unknown;
  let changedInputs: unknown;
  let resource: unknown;
  let wire: unknown;
  const remember = (error: unknown) =>
    failures.push(error instanceof Error ? (error.stack ?? error.message) : String(error));
  try {
    session = new LspSession({ repoRoot, binary });
    processId = session.processId;
    recorder = new QueryRecorder(session, runtime.executable, binary);
    initialization = await session.initialize(workspace, { editor: true, typecheck: true });
    session.notify("textDocument/didOpen", {
      textDocument: { uri, languageId: "vue", version: 1, text: source },
    });
    await waitForInitialTypes(session, uri);
    // One complete priming sweep is retained separately, then every one of
    // the five reported four-request rounds is measured without case selection.
    try {
      assertTypedPackets(await recorder.sweep("prime", requests(uri, source)), workspace);
    } catch (error) {
      remember(error);
    }
    for (let round = 1; round <= 5; round++) {
      try {
        assertTypedPackets(await recorder.sweep(`warm-${round}`, requests(uri, source)), workspace);
      } catch (error) {
        remember(error);
      }
    }
    changedInputs = await controls(recorder, workspace, source, runtime);
  } catch (error) {
    remember(error);
  } finally {
    if (recorder) {
      try {
        for (const row of recorder.rows) {
          for (const entry of row.processes as Array<
            ProcessIdentity & { executable: string | null }
          >) {
            if (entry.executable === runtime.executable)
              natives.set(`${entry.pid}:${entry.birth_ticks}`, entry);
          }
        }
      } catch (error) {
        remember(error);
      }
      try {
        for (const entry of nativeIdentities(recorder.sampler.sample(), runtime.executable)) {
          natives.set(`${entry.pid}:${entry.birth_ticks}`, entry);
        }
      } catch (error) {
        remember(error);
      }
      try {
        resource = recorder.sampler.stop();
      } catch (error) {
        remember(error);
      }
    }
    if (session) {
      try {
        await session.shutdown();
      } catch (error) {
        remember(error);
        try {
          await session.kill();
        } catch (cleanup) {
          remember(cleanup);
        }
      }
    }
    try {
      retirement = await awaitRetired([...natives.values()]);
    } catch (error) {
      remember(error);
    }
    const created = fs.readdirSync(captureRoot).filter((name) => !previous.includes(name));
    if (created.length) {
      try {
        wire = await finishWire(repoRoot, previous);
      } catch (error) {
        remember(error);
      }
      for (const name of created) {
        try {
          fs.cpSync(path.join(captureRoot, name), path.join(output, name), { recursive: true });
        } catch (error) {
          remember(error);
        }
      }
    }
    if (wire && recorder) {
      try {
        const captured = wire as { capture: string };
        const client = decodeFrames(
          fs.readFileSync(path.join(captured.capture, "client.bin")),
        ).messages;
        const server = decodeFrames(
          fs.readFileSync(path.join(captured.capture, "server.bin")),
        ).messages;
        const responses = server.filter(
          (message) => typeof message.id === "number" && message.method == null,
        );
        assert.deepEqual(
          responses,
          recorder.responses,
          "passive response packets must match complete captured frames",
        );
        assert.equal(
          new Set(responses.map((message) => message.id)).size,
          responses.length,
          "duplicate response IDs are refused",
        );
        for (const row of recorder.rows) {
          const sent = client.filter((message) => message.id === row.requestId);
          const received = responses.filter((message) => message.id === row.requestId);
          assert.equal(sent.length, 1);
          assert.equal(received.length, 1);
          assert.deepEqual(sent[0], {
            jsonrpc: "2.0",
            id: row.requestId,
            method: row.method,
            params: row.params,
          });
          assert.deepEqual(
            received[0],
            row.response,
            "unknown envelope/error fields and their presence remain whole",
          );
        }
      } catch (error) {
        remember(error);
      }
    }
  }
  return {
    source: sourceIdentity(repoRoot),
    inputs,
    changedInputs,
    initialization,
    processId,
    rows: recorder?.rows ?? [],
    failures: [...failures, ...(recorder?.failures ?? [])],
    resource,
    wire,
    notifications: recorder?.notifications,
    responses: recorder?.responses,
    sessionRoots: recorder && {
      physical: recorder.roots.roots,
      spellings: [...recorder.roots.spellings],
    },
    observedNativeIdentities: [...natives.values()],
    retirement,
  };
}

export async function pairedWarmRequests(beforeRoot: string, outputRoot: string) {
  assert.equal(process.platform, "linux", "actual native CPU qualification requires Linux /proc");
  assert.ok(process.env.RUNNER_TEMP);
  assert.equal(beforeRoot, path.join(process.env.RUNNER_TEMP, "warm-before"));
  assert.equal(outputRoot, path.join(process.env.RUNNER_TEMP, "warm-pair"));
  const sourceBinding = JSON.parse(
    fs.readFileSync(path.join(outputRoot, "workflow-source.json"), "utf8"),
  );
  assert.equal(sourceIdentity(beforeRoot).revision, sourceBinding.baseline);
  assert.equal(sourceIdentity(driverRoot).revision, sourceBinding.head);
  fs.copyFileSync(
    path.join(driverRoot, "target/ci/vize.differential-build.json"),
    path.join(outputRoot, "after-build.json"),
  );
  const runtime = runtimeIdentity();
  const originals = inputAuthority();
  const workspace = path.join(outputRoot, "workspace");
  fs.mkdirSync(outputRoot, { recursive: true });
  const before = await runSide(beforeRoot, workspace, runtime, path.join(outputRoot, "before"));
  fs.writeFileSync(path.join(outputRoot, "before.json"), `${JSON.stringify(before, null, 2)}\n`);
  const after = await runSide(driverRoot, workspace, runtime, path.join(outputRoot, "after"));
  fs.writeFileSync(path.join(outputRoot, "after.json"), `${JSON.stringify(after, null, 2)}\n`);
  const packet = {
    originals,
    runtime,
    before,
    after,
    protocol:
      "original400+134; same worker/workspace/locks/runtime/ci build; completed initial types+10s idle; retained prime; all5x4 warm requests",
    timingScope:
      "request wall time; inclusive sampled Linux Maestro/native-descendant CPU; observational, no numeric ceiling or Program count",
    pendingDelivery:
      "source-qualified observation only; protected full suites/instruction gates, actual merge and release remain separate",
  };
  fs.writeFileSync(path.join(outputRoot, "paired.json"), `${JSON.stringify(packet, null, 2)}\n`);
  assert.deepEqual(before.failures, [], "every before outcome is retained and required");
  assert.deepEqual(after.failures, [], "every current outcome is retained and required");
  assert.deepEqual(before.inputs.source, after.inputs.source);
  assert.equal(before.inputs.sourceSha256, after.inputs.sourceSha256);
  assert.deepEqual(before.source.locks, after.source.locks);
  assert.deepEqual(before.initialization, after.initialization);
  assert.equal(before.source.dirty, "");
  assert.equal(after.source.dirty, "");
  const publicPackets = (side: typeof before) =>
    side.rows.map((row) => ({
      stage: row.stage,
      name: row.name,
      method: row.method,
      params: row.comparableParams,
      requestId: row.requestId,
      response: row.comparableResponse,
    }));
  // Physical random session roots are recorded and checked while live, then
  // substituted bijectively. No public field, native data or response is dropped.
  const completePackets = publicPackets;
  assert.deepEqual(completePackets(after), completePackets(before));
  for (const side of [before, after]) {
    assert.equal(side.rows.filter((row) => /^warm-[1-5]$/u.test(String(row.stage))).length, 20);
  }
  assert.notEqual(before.processId, after.processId, "two source builds own two real processes");
  fs.writeFileSync(
    path.join(outputRoot, "qualified.json"),
    `${JSON.stringify(
      {
        before: before.source,
        after: after.source,
        wholePacketsEqual: true,
        warmRequestsPerSide: 20,
        observations: [before, after].map((side) =>
          side.rows.filter((row) => /^warm-[1-5]$/u.test(String(row.stage))),
        ),
      },
      null,
      2,
    )}\n`,
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.ok(
    process.argv[2] && process.argv[3],
    "usage: warm-type-backed-requests.ts BEFORE_ROOT OUTPUT_ROOT",
  );
  await pairedWarmRequests(path.resolve(process.argv[2]), path.resolve(process.argv[3]));
}
