import assert from "node:assert/strict";
import { ChildProcess, execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { setTimeout as sleep } from "node:timers/promises";
import {
  object,
  hash,
  expectedDiagnostics,
  auditPublications,
  diagnosticReceipts,
  writeHuge,
  vbenFixture,
  VBEN_REVISION,
  type Obj,
  type Fixture,
  type Target,
} from "./fixtures.ts";
import { startResources, memory } from "./resources.ts";
import {
  writeAll,
  ns,
  errorText,
  phaseEvents,
  classifyInterval,
  firstPair,
  open,
  change,
  collectCaptures,
  resolveIssued,
  type Session,
} from "./native-queries.ts";
type Mode = "timing" | "observe" | "memory";
type Modules = {
  Session: new () => Session;
  withFixture: (options: Obj, run: (fixture: Obj) => Promise<void>) => Promise<void>;
  linkPackages: (root: string, paths: string[]) => string[];
  linkVue: (root: string) => void;
  linkDirectory: (source: string, target: string) => void;
};
const save = (file: string, value: unknown): void =>
  fs.writeFileSync(file, JSON.stringify(value, null, 2) + "\n", { flag: "wx", mode: 0o600 });
export function parseArgs(args: readonly string[]): {
  mode: Mode;
  repo: string;
  output: string;
  source: string;
  attempt: number;
} {
  assert.equal(args.length, 10);
  const values: Record<string, string> = {};
  for (let i = 0; i < args.length; i += 2) {
    const key = args[i]!;
    assert.ok(["--mode", "--repo", "--output", "--source", "--attempt"].includes(key));
    assert.equal(values[key], undefined);
    values[key] = args[i + 1]!;
  }
  assert.equal(values["--mode"], "timing", "focused comparison is timing only");
  assert.match(values["--source"]!, /^[a-f0-9]{40}$/);
  assert.ok([1, 2, 3, 4].includes(Number(values["--attempt"])));
  assert.ok(path.isAbsolute(values["--repo"]!) && path.isAbsolute(values["--output"]!));
  return {
    mode: values["--mode"] as Mode,
    repo: values["--repo"]!,
    output: values["--output"]!,
    source: values["--source"]!,
    attempt: Number(values["--attempt"]),
  };
}
async function modules(repo: string): Promise<Modules> {
  const session = object(
    await import(pathToFileURL(path.join(repo, "tests/tooling/support/lsp/session.ts")).href),
  );
  const fixture = object(
    await import(pathToFileURL(path.join(repo, "tests/_helpers/realworld-patch.ts")).href),
  );
  const types = object(
    await import(pathToFileURL(path.join(repo, "tests/_helpers/realworld-typecheck.ts")).href),
  );
  return {
    Session: session.LspSession,
    withFixture: fixture.withPinnedFixtureWorkspace,
    linkPackages: fixture.linkWorkspacePackages,
    linkDirectory: fixture.symlinkDirectory,
    linkVue: types.symlinkVueTypes,
  } as unknown as Modules;
}
async function diagnostic(
  session: Session,
  t: Target,
  kind: Fixture["id"],
  version: number,
): Promise<unknown> {
  const packet = await session.waitForNotification(
    "textDocument/publishDiagnostics",
    (params) => {
      const row = object(params);
      return (
        row.uri === t.uri &&
        row.version === version &&
        Array.isArray(row.diagnostics) &&
        !row.diagnostics.some((value) => object(value).code === "typecheck-pending") &&
        (version !== 1 ||
          (Array.isArray(row.diagnostics) &&
            row.diagnostics.some((value) => [2322, 2339].includes(Number(object(value).code)))))
      );
    },
    60_000,
  );
  assert.deepEqual(packet, {
    uri: t.uri,
    version,
    diagnostics: expectedDiagnostics(version === 1 ? t.invalid : t.repair, kind, version !== 1),
  });
  return packet;
}
async function scenario(
  args: ReturnType<typeof parseArgs>,
  api: Modules,
  fixture: Fixture,
  arrival: number | string,
  index: number,
): Promise<Obj> {
  const dir = path.join(args.output, `case-${index}`);
  fs.mkdirSync(dir);
  const events: Obj[] = [];
  const publications: Obj[] = [];
  const publicationReceipts: Obj[] = [];
  const queryAttempts: Obj[] = [];
  let bytes = 0;
  const fd = fs.openSync(path.join(dir, "events.ndjson"), "wx", 0o600);
  const errors: string[] = [];
  const emit = (row: Obj): void => {
    if (row.kind === "requestAttempt") queryAttempts.push(row);
    const line = JSON.stringify(row) + "\n";
    bytes += Buffer.byteLength(line);
    if (bytes > 16 * 1024 * 1024) {
      if (!errors.includes("event evidence cap exceeded"))
        errors.push("event evidence cap exceeded");
      return;
    }
    writeAll(fd, line);
  };
  save(path.join(dir, "fixture-inputs.json"), {
    manifest: fixture.manifest,
    targets: fixture.targets.map((t) => ({
      ...t,
      closedSupport: fs.readFileSync(
        path.join(
          path.dirname(t.file),
          fixture.id === "huge4096" ? "support.ts" : "__native_invoice.ts",
        ),
        "utf8",
      ),
    })),
  });
  const constructorStartedNs = ns();
  const session = new api.Session();
  const child = Reflect.get(session, "process");
  assert.ok(child instanceof ChildProcess);
  assert.equal(child.pid, session.processId);
  let spawnEventNs: string | null = null;
  child.once("spawn", () => {
    spawnEventNs = ns();
    emit({ kind: "spawnEventReceipt", ns: spawnEventNs });
  });
  let stderrOffset = 0;
  let stderrTail = "";
  session.stderrObservers.push((text) => {
    const delta = text.slice(stderrOffset);
    stderrOffset = text.length;
    const received = ns();
    emit({ kind: "stderrReceipt", ns: received, text: delta });
    const lines = (stderrTail + delta).split("\n");
    stderrTail = lines.pop()!;
    for (const row of phaseEvents(lines.join("\n"), fixture.targets[0]!.uri, events)) {
      const value = { ...row, ns: received, sequence: events.length };
      events.push(value);
      emit({ kind: "sourceEvent", ...value });
    }
  });
  session.notificationObservers.push((method, params) => {
    if (method === "textDocument/publishDiagnostics") {
      publications.push(object(params));
      publicationReceipts.push({ ns: ns(), params });
    }
    emit({ kind: "notification", ns: ns(), method, params });
  });
  session.responseObservers.push((message) => emit({ kind: "response", ns: ns(), message }));
  let resource;
  try {
    resource = await startResources(args.repo, session.processId, dir, false);
  } catch (error) {
    await session.shutdown();
    await session.waitForClosed();
    fs.closeSync(fd);
    throw error;
  }
  const result: Obj = {
    fixture: fixture.id,
    arrival,
    manifest: fixture.manifest,
    constructorStartedNs,
    targets: fixture.targets.map((t) => ({
      uri: t.uri,
      invalidSha256: hash(t.invalid),
      repairSha256: hash(t.repair),
    })),
    errors,
    queryAttempts,
  };
  try {
    const initialized = await session.initialize(fixture.directory, {
      editor: true,
      lint: false,
      ecosystem: false,
      typecheck: true,
      hover: true,
      completion: true,
    });
    const readyNs = ns();
    Object.assign(result, {
      initialized,
      readyNs,
      spawnEventNs,
      spawnScope:
        "actual child spawn-event receipt→initialize-return; constructor/probe lower bound retained",
    });
    const openedNs = ns();
    for (const t of fixture.targets) open(session, t, t.invalid, 1);
    emit({ kind: "didOpenAttempt", ns: openedNs, targets: fixture.targets.map((t) => t.uri) });
    result.openedNs = openedNs;
    if (typeof arrival === "number" && arrival > 0)
      await sleep(Math.max(0, arrival - Number(BigInt(ns()) - BigInt(openedNs)) / 1e6));
    else if (typeof arrival === "string" && arrival !== "beforePrep") {
      const wanted =
        arrival === "duringPrep"
          ? "preparing"
          : arrival === "collectorWait"
            ? "collectorSelected"
            : "apiEntered";
      const deadline = Date.now() + 60_000;
      while (!events.some((row) => row.event === wanted)) {
        if (Date.now() > deadline) throw Error(`no actual ${wanted} event`);
        await sleep(1);
      }
    }
    try {
      result.firstNative = await firstPair(session, fixture.targets[0]!, false, emit);
    } catch (error) {
      errors.push(errorText(error));
    }
    const firstNativeEnd = ns();
    result.firstNativeEndNs = firstNativeEnd;
    const initial = await Promise.allSettled(
      fixture.targets.map((t) => diagnostic(session, t, fixture.id, 1)),
    );
    result.initialDiagnostics = initial.map((row) =>
      row.status === "fulfilled" ? row.value : { error: errorText(row.reason) },
    );
    for (const row of initial) if (row.status === "rejected") errors.push(errorText(row.reason));
    result.allDeclaredTargetQualificationNs = ns();
    try {
      const clock = diagnosticReceipts(publicationReceipts, fixture.targets, fixture.id);
      result.nativeDiagnosticReceipts = clock.rows;
      result.firstExactNativeDiagnosticReceiveNs = clock.firstNs;
      result.allDeclaredTargetDiagnosticReceiveNs = clock.allNs;
    } catch (error) {
      errors.push(errorText(error));
    }
    if (result.firstNative)
      try {
        result.untimedResolve = await resolveIssued(session, object(result.firstNative), emit);
      } catch (error) {
        errors.push(errorText(error));
      }
    for (const t of fixture.targets) change(session, t, t.repair, 2);
    emit({ kind: "repairAttempt", ns: ns(), version: 2 });
    try {
      result.currentNative = await firstPair(session, fixture.targets[0]!, true, emit);
      if (result.firstNative)
        assert.ok(
          Number(object(object(result.currentNative).completion).revision) >
            Number(object(object(result.firstNative).completion).revision),
        );
    } catch (error) {
      errors.push(errorText(error));
    }
    const repaired = await Promise.allSettled(
      fixture.targets.map((t) => diagnostic(session, t, fixture.id, 2)),
    );
    result.repairedDiagnostics = repaired.map((row) =>
      row.status === "fulfilled" ? row.value : { error: errorText(row.reason) },
    );
    for (const row of repaired) if (row.status === "rejected") errors.push(errorText(row.reason));
    const endDeadline = Date.now() + 60_000;
    while (
      !fixture.targets.every((t) =>
        session.stderrText.includes(`finished initial type diagnostics for ${t.uri} version 2`),
      )
    ) {
      if (Date.now() > endDeadline)
        throw Error("full repair collector completion was not observed");
      await sleep(5);
    }
    result.fullRepairCollectorsCompleteNs = ns();
    auditPublications(publications, fixture.targets, fixture.id);
    result.publications = publications;
    result.untimedResources = resource.snapshot("afterCompleteDeclaredTargets", false);
    result.events = events;
    result.droppedRetry = {
      state: "UNKNOWN",
      reason: "no dedicated source counter; full stderr and all versioned publications retained",
    };
    if (args.mode === "observe") {
      if (arrival === "apiWait") {
        result.apiWait = classifyInterval(
          events,
          String(queryAttempts[0]?.ns),
          result.firstNative ? String(object(result.firstNative).receivedNs) : undefined,
        );
        assert.equal(object(result.apiWait).state, "matchedCallerApiWait");
      }
      if (arrival === "collectorWait") {
        const selected = events.find((row) => row.event === "collectorSelected");
        const finished = events.find((row) => row.event === "collectorFinished");
        const sent = BigInt(String(queryAttempts[0]?.ns));
        result.collectorWait = {
          selected,
          finished,
          requestSentNs: String(sent),
          apiWait: "UNAVAILABLE",
          nativeCpuEntered: "UNKNOWN",
          scope:
            "worker selection through finish only; includes admission/configuration and is not an API/CPU interval",
        };
        assert.ok(
          selected &&
            finished &&
            BigInt(String(selected.ns)) < sent &&
            sent < BigInt(String(finished.ns)),
          "actual worker collector interval must contain request attempt",
        );
      }
      if (arrival === "duringPrep") {
        const sent = BigInt(String(queryAttempts[0]?.ns));
        const begin = events.find((row) => row.event === "preparing");
        const end = events.find((row) => row.event === "prepared");
        assert.ok(
          begin && end && BigInt(String(begin.ns)) < sent && sent < BigInt(String(end.ns)),
          "actual successful-preparation interval must contain the request attempt",
        );
      }
    }
  } catch (error) {
    errors.push(errorText(error));
  } finally {
    for (const t of fixture.targets)
      session.notify("textDocument/didClose", { textDocument: { uri: t.uri } });
    try {
      await session.shutdown();
      await session.waitForClosed();
      assert.equal(session.exitedGracefully, true);
    } catch (error) {
      errors.push(errorText(error));
    }
    const resources = resource.stop();
    result.resources = resources;
    if (resources.failure) errors.push(String(resources.failure));
    fs.closeSync(fd);
    save(path.join(dir, "case.json"), result);
  }
  return result;
}
async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  assert.equal(fs.realpathSync(args.repo), fs.realpathSync(process.cwd()));
  assert.equal(
    execFileSync("git", ["rev-parse", "HEAD"], { cwd: args.repo, encoding: "utf8" }).trim(),
    args.source,
  );
  assert.equal(process.env.VIZE_LSP_REQUIRE_SOURCE_BUILD, "1");
  assert.equal(process.env.VIZE_LSP_BUILD_RECIPE, "cargo build --release -p vize");
  assert.equal(process.env.VIZE_TRACE_EDITOR_PREPARATION, args.mode === "timing" ? undefined : "1");
  assert.equal(process.env.VIZE_PERF_BUDGET_SCALE, undefined);
  const captureRoot = path.join(args.repo, "target/differential/lsp-sessions");
  assert.ok(!fs.existsSync(captureRoot));
  const binary = process.env.VIZE_LSP_BIN;
  assert.ok(binary);
  const receipt = object(JSON.parse(fs.readFileSync(`${binary}.differential-build.json`, "utf8")));
  assert.equal(receipt.sourceRevision, args.source);
  const image = String(receipt.binarySha256);
  assert.equal(hash(fs.readFileSync(binary)), image);
  const api = await modules(args.repo);
  const rows: Obj[] = [];
  const errors: string[] = [];
  const outcome: Obj = {
    schemaVersion: 1,
    source: args.source,
    mode: args.mode,
    attempt: args.attempt,
    complete: false,
    errors,
    rows,
    rawCaptures: [],
    scope: {
      firstArmRerun: false,
      osCacheCold: "UNKNOWN",
      initialCompletionProjectionAuthority: "UNAVAILABLE",
      droppedRetry: "UNKNOWN",
      nativeCpuEntered: "UNKNOWN",
    },
  };
  try {
    if (args.mode === "memory") rows.push(await memory(args, api));
    else {
      const arrivals = args.mode === "timing" ? [0] : ["beforePrep", "duringPrep", "apiWait"];
      for (const [index, arrival] of arrivals.entries()) {
        const directory = path.join(
          args.repo,
          "target/vize-tests/initial-ready-supplement",
          `${process.env.GITHUB_RUN_ID}-${args.mode}-${args.source.slice(0, 8)}-${args.attempt}-huge-${index}`,
        );
        const fixture = writeHuge(directory);
        api.linkVue(directory);
        try {
          rows.push(await scenario(args, api, fixture, arrival!, index));
        } catch (error) {
          errors.push(errorText(error));
        }
      }
      await api.withFixture(
        {
          fixtureId: "vue-vben-admin",
          includePaths: [
            "apps",
            "packages",
            "internal",
            "playground",
            "package.json",
            "pnpm-workspace.yaml",
          ],
        },
        async (value) => {
          const directory = String(value.workspaceDir);
          const entry = object(value.entry);
          assert.equal(entry.revision, VBEN_REVISION);
          const sourceFiles = [
            "apps/web-antd/src/views/_core/fallback/offline.vue",
            "apps/web-ele/src/views/_core/fallback/offline.vue",
          ].map((file) => fs.readFileSync(path.join(directory, file), "utf8"));
          api.linkPackages(directory, ["apps", "packages", "internal"]);
          api.linkVue(directory);
          api.linkDirectory(
            path.join(args.repo, "node_modules/vite"),
            path.join(directory, "node_modules/vite"),
          );
          for (const [index, arrival] of arrivals.entries()) {
            const fixture = vbenFixture(directory, sourceFiles);
            const selectedArrival = arrival === "apiWait" ? "collectorWait" : arrival;
            try {
              rows.push(await scenario(args, api, fixture, selectedArrival!, index + 3));
            } catch (error) {
              errors.push(errorText(error));
            }
          }
        },
      );
    }
  } catch (error) {
    errors.push(errorText(error));
  } finally {
    for (const row of rows) for (const error of object(row).errors as string[]) errors.push(error);
    try {
      outcome.rawCaptures = collectCaptures(args, image);
      assert.equal(
        (outcome.rawCaptures as Obj[]).length,
        args.mode === "memory" ? 1 : args.mode === "timing" ? 2 : 6,
      );
    } catch (error) {
      errors.push(errorText(error));
    }
    outcome.complete =
      rows.length === (args.mode === "memory" ? 1 : args.mode === "timing" ? 2 : 6) &&
      errors.length === 0;
    save(path.join(args.output, "outcome.json"), outcome);
  }
  assert.equal(
    outcome.complete,
    true,
    "all supplemental cases/raw/source/resource laws required; failures are retained",
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
