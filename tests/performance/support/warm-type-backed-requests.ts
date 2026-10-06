import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { LspSession } from "../../tooling/support/lsp/session.ts";
import type { PublishedSessionBinding } from "../../tooling/support/lsp/session-process.ts";
import { decodeFrames } from "../../differential/lsp-wire.ts";
import {
  awaitRetired,
  nativeIdentities,
  type ProcessIdentity,
} from "./warm-type-backed-processes.ts";
import { controls } from "./warm-type-backed-controls.ts";
import { assertQueryFrames } from "./warm-type-backed-query-frames.ts";
import { startup } from "./warm-type-backed-startup.ts";
import { QueryRecorder } from "./warm-type-backed-measure.ts";
import {
  assertOriginalFrames,
  assertTypedPackets,
  finishWire,
  object,
  requests,
  waitForInitialTypes,
} from "./warm-type-backed-packets.ts";
import {
  driverRoot,
  generateWorkspace,
  inputAuthority,
  runtimeIdentity,
  sha256,
  sourceIdentity,
} from "./warm-type-backed-source.ts";

export async function runSide(
  repoRoot: string,
  workspace: string,
  runtime: ReturnType<typeof runtimeIdentity>,
  output: string,
  published?: PublishedSessionBinding & { binary: string },
  coldStart = false,
) {
  const binary = published?.binary ?? path.join(repoRoot, "target/ci/vize");
  const captureRoot =
    published?.captureRoot ?? path.join(repoRoot, "target/differential/lsp-sessions");
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
  let startupTimings: unknown;
  let changedInputs: unknown;
  let resource: unknown;
  let wire: unknown;
  const remember = (error: unknown) =>
    failures.push(error instanceof Error ? (error.stack ?? error.message) : String(error));
  try {
    session = new LspSession({ repoRoot, binary, ...(published ? { published } : {}) });
    processId = session.processId;
    recorder = new QueryRecorder(session, runtime.executable, binary);
    ({ initialization, startupTimings } = await startup(
      recorder,
      workspace,
      uri,
      source,
      coldStart,
    ));
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
        wire = await finishWire(repoRoot, previous, captureRoot);
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
        const responses = assertOriginalFrames(
          client,
          server,
          initialization,
          coldStart ? { repoRoot, workspace, source } : 0,
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
        assertQueryFrames(recorder.rows, client, responses);
      } catch (error) {
        remember(error);
      }
    }
  }
  return {
    source: sourceIdentity(repoRoot),
    ...(published
      ? {
          sourceMeaning:
            "driver checkout only; actual installed release source is bound by publicationReceipt",
          launchAuthority: published.authority,
        }
      : {}),
    inputs,
    changedInputs,
    initialization,
    startupTimings,
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
  const afterRoot = sourceBinding.afterRoot ?? driverRoot;
  const releaseCut = sourceBinding.authority === "root-frozen-release-cut";
  assert.deepEqual(sourceIdentity(driverRoot), sourceBinding.driverSource);
  assert.equal(sourceIdentity(beforeRoot).revision, sourceBinding.baseline);
  assert.equal(sourceIdentity(afterRoot).revision, sourceBinding.head);
  const buildCustody = [beforeRoot, afterRoot].map((root, index) => {
    const side = index === 0 ? "before" : "after";
    const custody = JSON.parse(
      fs.readFileSync(path.join(outputRoot, `${side}-cargo-custody.json`), "utf8"),
    );
    const binary = path.join(root, "target/ci/vize");
    assert.equal(custody.side, side);
    assert.equal(custody.launchBinary, binary);
    assert.deepEqual(custody.source, sourceIdentity(root));
    assert.equal(custody.binarySha256, sha256(fs.readFileSync(binary)));
    const receipt = JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8"));
    assert.equal(receipt.binarySha256, custody.binarySha256);
    assert.equal(receipt.sourceRevision, custody.source.revision);
    return custody;
  });
  const runtime = runtimeIdentity();
  const originals = inputAuthority();
  const workspace = path.join(outputRoot, "workspace");
  fs.mkdirSync(outputRoot, { recursive: true });
  const run = (root: string, side: string) =>
    runSide(root, workspace, runtime, path.join(outputRoot, side), undefined, true);
  const before = await run(beforeRoot, "before");
  fs.writeFileSync(path.join(outputRoot, "before.json"), `${JSON.stringify(before, null, 2)}\n`);
  const after = await run(afterRoot, "after");
  fs.writeFileSync(path.join(outputRoot, "after.json"), `${JSON.stringify(after, null, 2)}\n`);
  const packet = {
    buildCustody,
    sourceBinding,
    originals,
    runtime,
    before,
    after,
    protocol: releaseCut
      ? "original400+134; same worker/workspace/runtime/attested shipping build; exact per-source locks; four immediate cold requests+one hover during actual native collection; completed initial types+10s idle; retained prime; all5x4 warm requests"
      : "original400+134; same worker/workspace/locks/runtime/attested shipping build; four immediate cold requests+one hover during actual native collection; completed initial types+10s idle; retained prime; all5x4 warm requests",
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
  if (!releaseCut) {
    assert.deepEqual(before.source.locks, after.source.locks);
    assert.deepEqual(before.initialization, after.initialization);
  } else {
    const expected = structuredClone(before.initialization);
    for (const [index, side] of [before, after].entries()) {
      const info = object(object(side.initialization).serverInfo);
      assert.equal(info.name, "vize-maestro");
      assert.equal(info.version, buildCustody[index].cliVersion.slice("vize ".length));
    }
    object(object(expected).serverInfo).version = buildCustody[1].cliVersion.slice("vize ".length);
    assert.deepEqual(
      after.initialization,
      expected,
      "only the attested setup release version differs",
    );
  }
  assert.equal(before.source.dirty, "");
  assert.equal(after.source.dirty, "");
  for (const [index, side] of [before, after].entries()) {
    const observed = (
      side.wire as { observation: { binary: { binarySha256: string; sourceRevision: string } } }
    ).observation.binary;
    assert.equal(observed.binarySha256, buildCustody[index].binarySha256);
    assert.equal(observed.sourceRevision, buildCustody[index].source.revision);
    assert.equal(
      buildCustody[index].binarySha256,
      sha256(fs.readFileSync(buildCustody[index].launchBinary)),
    );
  }
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
  if (!releaseCut) assert.deepEqual(after.notifications, before.notifications);
  for (const side of [before, after]) {
    assert.equal(
      side.rows.length,
      79,
      "four cold, one background and all74 original whole rows are required",
    );
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
        coldRequestsPerSide: 4,
        backgroundRequestsPerSide: 1,
        startupTimings: [before.startupTimings, after.startupTimings],
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
