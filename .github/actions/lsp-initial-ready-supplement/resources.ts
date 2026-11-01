import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { pathToFileURL } from "node:url";
import { object, hash, memoryWorkspace, type Obj, type Target } from "./fixtures.ts";
import { writeAll, open, firstPair, ns, errorText, type Session } from "./native-queries.ts";
type ProcessRow = {
  pid: number;
  parentPid: number;
  startTicks: string;
  state: string;
  rssKiB: number;
};
type Sampling = {
  parseStat: (pid: number, stat: string, pageSize: number) => ProcessRow;
  owned: (
    table: ReadonlyMap<number, ProcessRow>,
    root: number,
    ticks: string,
    known: ReadonlyMap<number, string>,
  ) => number[];
};
export function parsePss(source: string): number {
  const row = source.match(/^Pss:\s+(\d+) kB$/m);
  assert.ok(row);
  return Number(row[1]);
}
export function summarizeRows(rows: readonly ProcessRow[], root: number): Obj {
  assert.ok(rows.some((row) => row.pid === root));
  return {
    rootKiB: rows.find((row) => row.pid === root)!.rssKiB,
    treeKiB: rows.reduce((sum, row) => sum + row.rssKiB, 0),
    processes: rows.length,
    members: rows,
  };
}
export async function startResources(
  repo: string,
  root: number,
  directory: string,
  memory: boolean,
): Promise<{ snapshot: (milestone: string | number, pss: boolean) => Obj; stop: () => Obj }> {
  const module = object(
    await import(
      pathToFileURL(path.join(repo, "tests/performance/support/current-baseline/sample-rss.ts"))
        .href
    ),
  );
  assert.equal(typeof module.parseStat, "function");
  assert.equal(typeof module.owned, "function");
  const api = module as unknown as Sampling;
  const pageSize = Number(execFileSync("getconf", ["PAGESIZE"], { encoding: "utf8" }).trim());
  assert.ok(Number.isSafeInteger(pageSize) && pageSize > 0);
  const read = (pid: number): ProcessRow =>
    api.parseStat(pid, fs.readFileSync(`/proc/${pid}/stat`, "utf8"), pageSize);
  const initial = read(root);
  const known = new Map<number, string>([[root, initial.startTicks]]);
  let last: ProcessRow[] = [];
  let failure: string | null = null;
  const gaps: number[] = [];
  const executableByBirth = new Map<string, Obj>();
  let previous = process.hrtime.bigint();
  let samples = 0;
  let peakRoot = 0;
  let peakTree = 0;
  let peakProcesses = 0;
  const fd = fs.openSync(path.join(directory, "rss.ndjson"), "wx", 0o600);
  let written = 0;
  let quotaDroppedRows = 0;
  const emit = (row: Obj): void => {
    const line = JSON.stringify(row) + "\n";
    if (written + Buffer.byteLength(line) > 16 * 1024 * 1024) {
      failure ??= "RSS output quota exceeded; coverage incomplete";
      quotaDroppedRows++;
      return;
    }
    written += Buffer.byteLength(line);
    writeAll(fd, line);
  };
  const sample = (): Obj => {
    const now = process.hrtime.bigint();
    gaps.push(Number(now - previous) / 1e6);
    previous = now;
    const table = new Map<number, ProcessRow>();
    for (const name of fs.readdirSync("/proc")) {
      if (!/^\d+$/.test(name)) continue;
      try {
        table.set(Number(name), read(Number(name)));
      } catch (error) {
        const code = object(error).code;
        if (!mayIgnoreStatRead(String(code), Number(name), known)) throw error;
      }
    }
    const ids = api.owned(table, root, initial.startTicks, known);
    last = ids.map((pid) => table.get(pid)!).filter((row) => row.state !== "Z");
    for (const row of last) known.set(row.pid, row.startTicks);
    const total = last.reduce((sum, row) => sum + row.rssKiB, 0);
    peakRoot = Math.max(peakRoot, last.find((row) => row.pid === root)?.rssKiB ?? 0);
    peakTree = Math.max(peakTree, total);
    peakProcesses = Math.max(peakProcesses, last.length);
    samples++;
    const row = {
      ns: String(now),
      rootKiB: last.find((row) => row.pid === root)?.rssKiB ?? null,
      treeKiB: total,
      processes: last.length,
      members: last,
    };
    emit(row);
    const limits = memory
      ? { rootMiB: 2048, treeMiB: 8192, processes: 32 }
      : { rootMiB: 256, treeMiB: 512, processes: 4 };
    if (
      peakRoot > limits.rootMiB * 1024 ||
      peakTree > limits.treeMiB * 1024 ||
      peakProcesses > limits.processes
    )
      failure ??= `distinct ${memory ? "8-context safety" : "foreground existing Vben"} resource cap exceeded`;
    return row;
  };
  emit({
    kind: "start",
    root,
    rootBirth: initial.startTicks,
    intervalMs: 50,
    limits: memory
      ? { rootMiB: 2048, treeMiB: 8192, processes: 32 }
      : { rootMiB: 256, treeMiB: 512, processes: 4 },
    pssDuringTiming: false,
  });
  sample();
  const timer = setInterval(() => {
    try {
      sample();
    } catch (error) {
      failure ??= error instanceof Error ? error.message : String(error);
    }
  }, 50);
  return {
    snapshot(milestone, pss) {
      const values = sample();
      const memoryRows = pss
        ? last.map((row) => {
            try {
              return {
                pid: row.pid,
                startTicks: row.startTicks,
                pssKiB: parsePss(fs.readFileSync(`/proc/${row.pid}/smaps_rollup`, "utf8")),
              };
            } catch (error) {
              return {
                pid: row.pid,
                startTicks: row.startTicks,
                pssKiB: null,
                error: error instanceof Error ? error.message : String(error),
              };
            }
          })
        : null;
      const identities = last.map((row) => {
        const key = `${row.pid}:${row.startTicks}`;
        try {
          let identity = executableByBirth.get(key);
          if (!identity) {
            const executable = fs.readlinkSync(`/proc/${row.pid}/exe`);
            const stat = fs.statSync(`/proc/${row.pid}/exe`);
            identity = {
              executable,
              bytes: stat.size,
              sha256: hash(fs.readFileSync(`/proc/${row.pid}/exe`)),
            };
            executableByBirth.set(key, identity);
          }
          return {
            pid: row.pid,
            startTicks: row.startTicks,
            ...identity,
            cwd: fs.readlinkSync(`/proc/${row.pid}/cwd`),
            scope: "executable body hashed once/birth at untimed milestone",
          };
        } catch (error) {
          return {
            pid: row.pid,
            startTicks: row.startTicks,
            error: error instanceof Error ? error.message : String(error),
          };
        }
      });
      const result = {
        milestone,
        ...values,
        identities,
        pss: memoryRows,
        pssScope: pss ? "fixed untimed milestone; RSS sum is not PSS" : "not requested",
      };
      emit({ kind: "milestone", ...result });
      return result;
    },
    stop() {
      clearInterval(timer);
      try {
        sample();
      } catch (error) {
        failure ??= error instanceof Error ? error.message : String(error);
      }
      const survivors = last.map((row) => ({ pid: row.pid, startTicks: row.startTicks }));
      const result = {
        samples,
        quotaDroppedRows,
        peakRootKiB: peakRoot,
        sampledPeakTreeKiB: peakTree,
        peakProcesses,
        gapsMs: gaps,
        failure,
        survivors,
        coverage:
          "50ms requested; actual gaps retained; sampled peak, not true peak; no PID-to-config inference",
      };
      emit({ kind: "finish", ...result });
      fs.closeSync(fd);
      return result;
    },
  };
}

export async function memory(
  args: { repo: string; output: string },
  api: { Session: new () => Session; linkVue: (root: string) => void },
): Promise<Obj> {
  const directory = path.join(
    args.repo,
    "target/vize-tests/initial-ready-supplement",
    `${process.env.GITHUB_RUN_ID}-memory-${path.basename(args.output)}`,
  );
  const fixture = memoryWorkspace(directory);
  for (const root of fixture.roots) api.linkVue(root);
  const errors: string[] = [];
  const session = new api.Session();
  const fd = fs.openSync(path.join(args.output, "memory-events.ndjson"), "wx", 0o600);
  let eventBytes = 0;
  const emit = (row: Obj): void => {
    const line = JSON.stringify(row) + "\n";
    eventBytes += Buffer.byteLength(line);
    if (eventBytes > 16 * 1024 * 1024) {
      if (!errors.includes("memory event evidence cap exceeded"))
        errors.push("memory event evidence cap exceeded");
      return;
    }
    writeAll(fd, line);
  };
  let stderrOffset = 0;
  session.stderrObservers.push((text) => {
    emit({ kind: "stderrReceipt", ns: ns(), text: text.slice(stderrOffset) });
    stderrOffset = text.length;
  });
  session.notificationObservers.push((method, params) =>
    emit({ kind: "notification", ns: ns(), method, params }),
  );
  session.responseObservers.push((message) => emit({ kind: "response", ns: ns(), message }));
  let resource;
  try {
    resource = await startResources(args.repo, session.processId, args.output, true);
  } catch (error) {
    await session.shutdown();
    await session.waitForClosed();
    fs.closeSync(fd);
    throw error;
  }
  const milestones: Obj[] = [];
  const queries: Obj[] = [];
  const row: Obj = {
    fixture: "distinct-configs",
    milestones,
    queries,
    errors,
    scope: "trace-on normal-release resource observer; no ordinary-memory or 8-context latency win",
  };
  const waitClean = async (t: Target, version: number): Promise<void> => {
    const packet = await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (params) => {
        const p = object(params);
        return (
          p.uri === t.uri &&
          p.version === version &&
          Array.isArray(p.diagnostics) &&
          p.diagnostics.length === 0
        );
      },
      60_000,
    );
    assert.deepEqual(packet, { uri: t.uri, version, diagnostics: [] });
  };
  const ownerFacts = (targets: Target[]): Obj[] => selectedOwnerFacts(session.stderrText, targets);
  try {
    await session.initialize(fixture.roots, {
      editor: true,
      lint: false,
      ecosystem: false,
      typecheck: true,
      hover: true,
      completion: true,
    });
    for (const [i, t] of fixture.targets.entries()) {
      open(session, t, t.invalid, 1);
      try {
        queries.push({ uri: t.uri, version: 1, ...(await firstPair(session, t, false, emit)) });
        await waitClean(t, 1);
        const deadline = Date.now() + 60_000;
        while (
          !session.stderrText.includes(`finished initial type diagnostics for ${t.uri} version 1`)
        ) {
          if (Date.now() > deadline) throw Error("native memory collector did not finish");
          await sleep(5);
        }
      } catch (error) {
        errors.push(errorText(error));
      }
      if ([1, 4, 8].includes(i + 1)) {
        const facts = ownerFacts(fixture.targets.slice(0, i + 1));
        assert.equal(
          new Set(facts.map((fact) => fact.selectedNativeConfig)).size,
          i + 1,
          "distinct actually selected configs required",
        );
        assert.ok(
          facts.every(
            (value) =>
              Array.isArray(value.selectedNativeRecords) && value.selectedNativeRecords.length > 0,
          ),
          "selected native config records required, no source-only owner count",
        );
        milestones.push({
          ...resource.snapshot(i + 1, true),
          expectedContextInputs: fixture.roots.slice(0, i + 1),
          observedOwnerFacts: facts,
        });
      }
    }
    for (const t of fixture.targets)
      session.notify("textDocument/didClose", { textDocument: { uri: t.uri } });
    emit({ kind: "closeAllAttempt", ns: ns(), uris: fixture.targets.map((t) => t.uri) });
    await sleep(250);
    milestones.push({
      ...resource.snapshot("closed", true),
      expectedContextInputs: fixture.roots,
      observedOwnerFacts: ownerFacts(fixture.targets),
      closeScope: "250ms quiet observation only; no close ACK/native quiescence/eviction claim",
    });
    for (const t of fixture.targets) open(session, t, t.repair, 3);
    emit({ kind: "reopenAttempt", ns: ns(), version: 3 });
    for (const t of fixture.targets)
      try {
        queries.push({ uri: t.uri, version: 3, ...(await firstPair(session, t, true, emit)) });
        await waitClean(t, 3);
        const deadline = Date.now() + 60_000;
        while (
          !session.stderrText.includes(`finished initial type diagnostics for ${t.uri} version 3`)
        ) {
          if (Date.now() > deadline) throw Error("reopened collector did not finish");
          await sleep(5);
        }
      } catch (error) {
        errors.push(errorText(error));
      }
    milestones.push({
      ...resource.snapshot("reopened8", true),
      expectedContextInputs: fixture.roots,
      observedOwnerFacts: ownerFacts(fixture.targets),
    });
  } catch (error) {
    errors.push(errorText(error));
  } finally {
    try {
      await session.shutdown();
      await session.waitForClosed();
      assert.equal(session.exitedGracefully, true);
    } catch (error) {
      errors.push(errorText(error));
    }
    const resources = resource.stop();
    row.resources = resources;
    if (resources.failure) errors.push(String(resources.failure));
    fs.closeSync(fd);
    fs.writeFileSync(path.join(args.output, "memory.json"), JSON.stringify(row, null, 2) + "\n", {
      flag: "wx",
      mode: 0o600,
    });
  }
  return row;
}

export function selectedOwnerFacts(text: string, targets: Target[]): Obj[] {
  return targets.map((t) => {
    const records = text
      .replace(/\u001b\[[0-9;]*m/g, "")
      .split("\n")
      .filter(
        (line) =>
          line.includes("initial_api_read") && line.includes(t.uri) && line.includes("config="),
      );
    const configs = records
      .map((line) => {
        const match = line.match(/\bconfig=("(?:[^"\\]|\\.)*"|\S+)/);
        return match
          ? match[1]!.startsWith('"')
            ? String(JSON.parse(match[1]!))
            : match[1]!
          : null;
      })
      .filter((value): value is string => typeof value === "string" && value.length > 0);
    assert.ok(configs.length > 0, "no actual selected native config record");
    return {
      uri: t.uri,
      sourceConfig: path.join(path.dirname(t.file), "tsconfig.json"),
      sourceConfigSha256: hash(fs.readFileSync(path.join(path.dirname(t.file), "tsconfig.json"))),
      selectedNativeRecords: records,
      selectedNativeConfig: configs.at(-1),
      pidToSelectedProject:
        "UNKNOWN unless explicit cwd facts establish relation; no one-process/config assumption",
    };
  });
}

export function mayIgnoreStatRead(
  code: string,
  pid: number,
  known: ReadonlyMap<number, string>,
): boolean {
  return (
    code === "ENOENT" ||
    code === "ESRCH" ||
    ((code === "EACCES" || code === "EPERM") && !known.has(pid))
  );
}
