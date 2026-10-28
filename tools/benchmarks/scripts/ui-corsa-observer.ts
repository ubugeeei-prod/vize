#!/usr/bin/env node
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  worker,
  echo,
  calibrate,
  identity,
  readRows,
  stamp,
  sha,
  sleep,
  outputDirectory,
  sourceRoot,
  sourceRevision,
  filename,
  realGo,
  rawFile,
  rpcFile,
  type Row,
} from "./case.js";
const self = fileURLToPath(import.meta.url);
function sample(pid: number): Row | null {
  try {
    const raw = fs.readFileSync("/proc/" + pid + "/stat", "utf8");
    const fields = raw
      .slice(raw.lastIndexOf(")") + 2)
      .trim()
      .split(/\s+/);
    const status = fs.readFileSync("/proc/" + pid + "/status", "utf8");
    const kb = (key: string) =>
      Number(new RegExp("^" + key + ":\\s+(\\d+) kB$", "m").exec(status)?.[1] ?? NaN);
    return {
      pid,
      ns: stamp(),
      state: fields[0],
      parentPid: Number(fields[1]),
      userTicks: fields[11],
      systemTicks: fields[12],
      startTicks: fields[19],
      vmRssKiB: kb("VmRSS"),
      vmHwmKiB: kb("VmHWM"),
      argv: fs
        .readFileSync("/proc/" + pid + "/cmdline", "utf8")
        .split("\0")
        .filter(Boolean),
      cwd: fs.readlinkSync("/proc/" + pid + "/cwd"),
    };
  } catch {
    return null;
  }
}
async function supervise() {
  fs.mkdirSync(outputDirectory, { recursive: true });
  const revision = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: sourceRoot,
    encoding: "utf8",
  }).trim();
  assert.equal(revision, sourceRevision);
  const calibration = await calibrate(self);
  const beforeGo = await identity(realGo);
  const samplePath = path.join(outputDirectory, "process-samples.jsonl");
  const child = spawn(process.execPath, [self, "--worker"], {
    cwd: path.join(sourceRoot, "npm/ui"),
    env: { ...process.env, VIZE_UI_RPC_REAL_EXE: realGo, VIZE_UI_RPC_LOG: rpcFile },
    stdio: ["ignore", "pipe", "pipe"],
  });
  assert(child.pid !== undefined);
  const workerPid = child.pid;
  const stdout = fs.createWriteStream(path.join(outputDirectory, "worker.stdout"));
  const stderr = fs.createWriteStream(path.join(outputDirectory, "worker.stderr"));
  child.stdout.pipe(stdout);
  child.stderr.pipe(stderr);
  let done = false;
  const completion = new Promise<{ code: number | null; signal: string | null }>(
    (resolve, reject) => {
      child.once("error", reject);
      child.once("close", (code, signal) => {
        done = true;
        resolve({ code, signal });
      });
    },
  );
  const timeout = setTimeout(() => {
    fs.appendFileSync(
      samplePath,
      JSON.stringify({
        event: "observer-deadline",
        ns: stamp(),
        pid: workerPid,
        milliseconds: 90000,
      }) + "\n",
    );
    child.kill("SIGTERM");
  }, 90_000);
  const seenGo = new Map<number, string>(),
    seenProxy = new Set<number>();
  const sampleOnce = () => {
    for (const row of readRows(rpcFile).filter((r) => r.event === "spawn")) {
      if (typeof row.proxyPid === "number") seenProxy.add(row.proxyPid);
      if (typeof row.goPid === "number") {
        const info = sample(row.goPid);
        if (info && typeof info.startTicks === "string") seenGo.set(row.goPid, info.startTicks);
      }
    }
    for (const pid of [workerPid, ...seenProxy, ...seenGo.keys()]) {
      const info = sample(pid);
      if (info)
        fs.appendFileSync(
          samplePath,
          JSON.stringify({
            role: pid === workerPid ? "native-node" : seenGo.has(pid) ? "go" : "observer-proxy",
            ...info,
          }) + "\n",
        );
    }
  };
  while (!done) {
    sampleOnce();
    await sleep(100);
  }
  const exit = await completion;
  clearTimeout(timeout);
  await sleep(2200);
  sampleOnce();
  const observerCleanup: Row[] = [];
  for (const [pid, startTicks] of seenGo) {
    const info = sample(pid);
    if (info?.startTicks === startTicks && info.state !== "Z") {
      process.kill(pid, "SIGTERM");
      observerCleanup.push({
        pid,
        startTicks,
        action: "SIGTERM",
        reason: "observer proxy may be killed before descendant cleanup; not natural backend exit",
      });
    }
  }
  const afterGo = await identity(realGo);
  assert.deepEqual(afterGo, beforeGo);
  const rpc = readRows(rpcFile),
    workerEvents = readRows(path.join(outputDirectory, "worker.jsonl"));
  const allSamples = readRows(samplePath);
  const frames = rpc.filter((r) => r.event === "frame");
  const requests = frames.filter((r) => r.direction === "to-go" && typeof r.method === "string");
  const responses = frames.filter((r) => r.direction === "from-go" && r.response === true);
  const exchange = requests.map((request) => {
    const response = responses.find((r) => request.id !== null && r.id === request.id);
    return {
      request,
      response: response ?? null,
      wireDurationNs:
        response && typeof response.ns === "string" && typeof request.ns === "string"
          ? (BigInt(response.ns) - BigInt(request.ns)).toString()
          : null,
    };
  });
  const loaded = fs.existsSync(path.join(outputDirectory, "loaded-images.json"))
    ? (JSON.parse(
        fs.readFileSync(path.join(outputDirectory, "loaded-images.json"), "utf8"),
      ) as unknown)
    : null;
  const end = workerEvents.find((r) => r.event === "native-call-end");
  const raw = fs.existsSync(rawFile)
    ? (JSON.parse(fs.readFileSync(rawFile, "utf8")) as unknown)
    : null;
  const buildFile = path.join(
    sourceRoot,
    "npm/native/.artifacts/native/formatter-history/build-receipt.json",
  );
  const build = fs.existsSync(buildFile)
    ? (JSON.parse(fs.readFileSync(buildFile, "utf8")) as Row)
    : null;
  const maxima = ["native-node", "go", "observer-proxy"].map((role) => {
    const rows = allSamples.filter((r) => r.role === role);
    const values = rows.flatMap((r) =>
      typeof r.vmRssKiB === "number" && Number.isFinite(r.vmRssKiB) ? [r.vmRssKiB] : [],
    );
    return {
      role,
      observedSamples: rows.length,
      sampledMaxRssKiB: values.length ? Math.max(...values) : null,
    };
  });
  const emitted = build?.emitted as
    | {
        cargo?: { fresh?: boolean; features?: string[]; profile?: unknown; manifest_path?: string };
        filename?: string;
      }
    | undefined;
  const builtHash = (build?.generated as { sha256?: string } | undefined)?.sha256;
  const imageRows = Array.isArray(loaded) ? (loaded as Array<{ sha256?: string }>) : [];
  const imageQualified =
    build?.schema === "vize.public-native-formatter-ci-build" &&
    emitted?.cargo?.fresh === false &&
    emitted.cargo.features?.includes("napi") === true &&
    emitted.cargo.features.includes("legacy") &&
    !emitted.cargo.features.some((name) => /profiling/.test(name)) &&
    imageRows.length === 1 &&
    typeof builtHash === "string" &&
    imageRows[0]?.sha256 === builtHash;
  const receipt = {
    diagnosticImageQualified: imageQualified,
    sourceRevision,
    originalRun: 40603481864,
    originalJob: 122537512822,
    originalNapiIdentity: null,
    originalGoIdentity: null,
    exactOriginalImageQualified: false,
    scope:
      "one same-source newly built diagnostic image; observer overhead; not cold/normal speed or eight-shard reproduction",
    originalTimeoutMilliseconds: 30000,
    observerDeadlineMilliseconds: 90000,
    exit,
    calibration,
    loadedNapi: loaded,
    go: beforeGo,
    goVersionAfterCase: execFileSync(realGo, ["--version"], { encoding: "utf8" }).trim(),
    source: { filename, bytes: fs.statSync(filename).size, sha256: sha(fs.readFileSync(filename)) },
    workerEvents,
    exchanges: exchange,
    backendSpawnExit: rpc.filter((r) => r.event !== "frame"),
    diagnosticCount: typeof end?.diagnosticsCount === "number" ? end.diagnosticsCount : null,
    nativeResult: raw,
    cleanSingleCase: end?.diagnosticsCount === 0 && exit.code === 0,
    originalFailureFixed: false,
    sampledMemory: maxima,
    memoryScope:
      "separate process RSS samples, not allocator totals; observer separate; do not sum independent peaks",
    samplesEveryMilliseconds: 100,
    clockTicksPerSecond: execFileSync("getconf", ["CLK_TCK"], { encoding: "utf8" }).trim(),
    observerCleanup,
    buildReceipt: build
      ? {
          schema: build.schema,
          source: build.source,
          emitted: build.emitted,
          environment: build.environment,
          process:
            typeof build.process === "object" && build.process !== null
              ? Object.fromEntries(
                  Object.entries(build.process).filter(
                    ([key]) => !["stdout", "stderr"].includes(key),
                  ),
                )
              : null,
        }
      : null,
    runtime: {
      node: process.version,
      platform: process.platform,
      architecture: process.arch,
      rustc: execFileSync("rustc", ["-vV"], { encoding: "utf8" }),
    },
    rawFiles: Object.fromEntries(
      fs
        .readdirSync(outputDirectory)
        .filter((name) => fs.statSync(path.join(outputDirectory, name)).isFile())
        .map((name) => {
          const bytes = fs.readFileSync(path.join(outputDirectory, name));
          return [name, { bytes: bytes.length, sha256: sha(bytes) }];
        }),
    ),
  };
  const bytes = JSON.stringify(receipt, null, 2) + "\n";
  assert(Buffer.byteLength(bytes) <= 95000, "only bounded text metadata may be downloaded");
  fs.writeFileSync(path.join(outputDirectory, "receipt.json"), bytes);
  console.log(bytes);
  assert(
    imageQualified && requests.some((r) => r.method === "updateSnapshot"),
    "diagnostic image or original failing RPC was not observed",
  );
}
if (process.argv[2] === "--worker") await worker();
else if (process.argv[2] === "--echo") await echo();
else await supervise();
