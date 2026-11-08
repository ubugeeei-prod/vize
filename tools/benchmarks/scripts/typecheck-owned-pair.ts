/** Opt-in paired source experiment. Never invoked by the stock corpus driver. */
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, readdirSync, realpathSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import {
  ownedNativeAuthority,
  providerClosure,
  sourceRevision,
} from "./typecheck-owned-pair-authority.ts";
import {
  assertSameEvidence,
  assertSameInputs,
  batchCommand,
  diagnosticEvidence,
  sha256,
  snapshotInputs,
  writeJson,
} from "./typecheck-owned-pair-evidence.ts";
import {
  clearProjectCaches,
  installRole,
  prepareOwnedWorkload,
  selectOwnedWorkloads,
} from "./typecheck-owned-pair-projects.ts";
import {
  compareOwnedProjections,
  ownedProbeRows,
  prepareOwnedProbe,
} from "./typecheck-owned-pair-projection.ts";
import { ownedProbeAuthority, runtimeProviderView } from "./typecheck-owned-pair-runtime.ts";
import { requireMeasurementRunner, runOwnedInvocation } from "./typecheck-owned-pair-run.ts";
import { hostArgs } from "./typecheck-corpus-pair-host.ts";
import { checkerBusyMs, checkMsByFile } from "./typecheck-corpus-pair-projection.ts";
import { hostCorsaPath } from "./typecheck-corpus-pair-batch.ts";
import { checkPacket, assertFileCoverage } from "./pr-performance-observation-check.ts";
import { median } from "./typecheck-corpus-pair-parse.ts";
import { gate, parseRun } from "./typecheck-scaling-gate.ts";
import { MAPPER_HOSTS, TYPECHECK_HOSTS_DIR, resolveMapperHosts } from "./typecheck-mapper-hosts.ts";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
export const PAIR_LANES = [
  { id: "tsc", format: "text", mapper: "vize", host: "tsc" },
  { id: "tsrs", format: "text", mapper: "vize", host: "tsrs" },
  { id: "batch-8", format: "json", mapper: "vize", host: "tsc" },
] as const;
export function selectPairLanes(requested?: string) {
  if (!requested) return [...PAIR_LANES];
  const ids = requested.split(",").map((id) => id.trim());
  assert.ok(ids.every(Boolean), "pair lane IDs must be non-empty");
  assert.equal(new Set(ids).size, ids.length, "pair lane IDs must be unique");
  return ids.map((id) => {
    const lane = PAIR_LANES.find((candidate) => candidate.id === id);
    assert.ok(lane, `unknown pair lane: ${id}`);
    return lane;
  });
}
export function sampleOrder(index: number) {
  return index % 2 === 0 ? (["base", "head"] as const) : (["head", "base"] as const);
}
export function requiredSamples(real: boolean) {
  return real
    ? ["cold", "warm-1", "warm-2", "warm-3", "warm-4", "warm-5"]
    : ["cold", "warm-1", "warm-2", "warm-3"];
}

async function main() {
  requireMeasurementRunner();
  const { values } = parseArgs({
    options: Object.fromEntries(
      ["base", "head", "base-sha", "head-sha", "work", "out", "workloads", "lanes"].map((name) => [
        name,
        { type: "string" as const },
      ]),
    ),
  });
  for (const name of ["base", "head", "base-sha", "head-sha", "work", "out"])
    assert.ok(values[name], `missing --${name}`);
  const builds = { base: resolve(values.base!), head: resolve(values.head!) };
  const sources = { base: values["base-sha"]!, head: values["head-sha"]! };
  assert.notEqual(sources.base, sources.head);
  const work = resolve(values.work!),
    out = resolve(values.out!);
  const workloads = selectOwnedWorkloads(values.workloads);
  const lanes = selectPairLanes(values.lanes);
  mkdirSync(work, { recursive: true });
  mkdirSync(out, { recursive: true });
  assert.deepEqual(readdirSync(work), [], "work directory must be fresh");
  const boundary = JSON.parse(readFileSync(join(out, "build-boundary.json"), "utf8"));
  for (const role of ["base", "head"] as const) {
    assert.equal(boundary[role].directory, builds[role]);
    assert.equal(boundary[role].source, sources[role]);
    for (const key of ["targetAbsent", "nativeOutputAbsent", "loaderImagesAbsent"])
      assert.equal(boundary[role][key], true);
  }
  const pins: Record<string, string> = {
    typescript: "7.1.0-beta",
    tsrs: "0.20.1-ts7.1.0-beta",
    "@vue/content-mapper": "4.0.0-alpha.3",
  };
  for (const [name, version] of Object.entries(pins))
    assert.equal(
      JSON.parse(
        readFileSync(join(TYPECHECK_HOSTS_DIR, "node_modules", name, "package.json"), "utf8"),
      ).version,
      version,
    );
  const authorities = {
    base: ownedNativeAuthority(builds.base, sources.base),
    head: ownedNativeAuthority(builds.head, sources.head),
  };
  assert.notEqual(
    authorities.base.token,
    authorities.head.token,
    "base/head must be independently built native images",
  );
  const hosts = {
    base: resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(builds.base, "npm/cli")),
    head: resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(builds.head, "npm/cli")),
  };
  const providers = [
    TYPECHECK_HOSTS_DIR,
    join(root, "tools/benchmarks/typecheck-corpora"),
    join(root, "tools/benchmarks/nuxt-app"),
    ...Object.values(builds).flatMap((directory) => [
      join(directory, "npm/cli"),
      join(directory, "npm/native"),
      join(directory, "Cargo.lock"),
      join(directory, "rust-toolchain.toml"),
      join(directory, "pnpm-lock.yaml"),
    ]),
  ];
  const providerBefore = providerClosure(providers);
  writeJson(join(out, "providers-before.json"), providerBefore);
  const experiment = {
    driverSource: sourceRevision(root),
    sources,
    boundary,
    pins,
    authorities,
    runner: process.env.VIZE_PAIR_RUNNER_LABEL,
    node: process.version,
    runnerImage: process.env.ImageVersion,
    processCold: true,
    coldDefinition:
      "cold is the first timed process AFTER untimed admissions/probes; no OS page-cache eviction",
    userCache: "fresh empty for every call",
    pageCache: "uncontrolled OS page cache; warm denotes order, not persistent checker",
    nativeHook:
      "actual loaded image/token observed in each Node tree; hook overhead included symmetrically",
    workloadIds: workloads,
    laneIds: lanes.map((lane) => lane.id),
    timing:
      "wall + reaped process-tree CPU + GNU max SINGLE-process RSS; separate 100ms sampled TREE RSS/PSS pass",
    diagnosticLimits:
      "complete CLI packet/ordered host diagnostic text; host CLI does not expose unprinted structured end/related fields",
    qualified: false,
  };
  writeJson(join(out, "experiment.json"), experiment);
  const allResults: object[] = [];
  const memoryQualification: Array<{
    workload: string;
    lane: string;
    qualified: boolean;
    memory: object | null;
  }> = [];
  for (const specification of workloads) {
    const workload = prepareOwnedWorkload(specification.id, work, out, hosts.base);
    for (const mapper of ["vize", "vue"] as const) prepareOwnedProbe(workload.directory, mapper);
    const probeAuthority = ownedProbeAuthority(workload.directory);
    const inputs = snapshotInputs(workload.directory);
    const expectedSfcFiles = inputs
      .filter((row) => row.file.endsWith(".vue"))
      .map((row) => row.file);
    assert.equal(
      expectedSfcFiles.length,
      workload.sfcs,
      "workload source count differs from the declared program",
    );
    writeJson(join(out, `${workload.id}-inputs.json`), inputs);
    const first = new Map<string, ReturnType<typeof diagnosticEvidence>>();
    const samples = new Map<
      string,
      Array<{
        wallMs: number;
        treeCpuMs: number | null;
        maxSingleProcessRssKb: number;
        phase: string;
      }>
    >();
    const projection = new Map<string, ReturnType<typeof ownedProbeRows>>();
    const invoke = async (
      role: "base" | "head",
      id: string,
      phase: string,
      memory: boolean,
      probe = false,
      vue = false,
    ) => {
      installRole(workload, hosts[role]);
      const removedCaches = clearProjectCaches(workload.directory);
      assertSameInputs(inputs, snapshotInputs(workload.directory));
      const mapper = vue ? "vue" : "vize",
        format = id.startsWith("batch-") ? "json" : "text";
      const host = MAPPER_HOSTS.find((host) => host.id === (id === "tsrs" ? "tsrs" : "tsc"))!;
      const config = probe
        ? `tsconfig.${mapper}-owned-pair-probe.json`
        : `tsconfig.${mapper}-mapper.json`;
      const traceArgs =
        probe && host.id === "tsc"
          ? ["--generateTrace", join(out, workload.id, `${vue ? "vue" : role}-trace`)]
          : [];
      const command =
        format === "json"
          ? batchCommand(
              builds[role],
              workload.directory,
              hostCorsaPath(TYPECHECK_HOSTS_DIR),
              id === "batch-1" ? 1 : 8,
            )
          : [
              process.execPath,
              hosts[role][host.id],
              ...hostArgs(host, workload.directory, config, traceArgs),
            ];
      const directory = join(out, workload.id, `${vue ? "vue" : role}-${id}`, phase);
      mkdirSync(directory, { recursive: true });
      const runtimeBefore = runtimeProviderView(
        workload.directory,
        hosts[role].vizeCliDir,
        providerBefore,
        probeAuthority,
      );
      writeJson(join(directory, "runtime-providers-before.json"), runtimeBefore);
      const run = await runOwnedInvocation({
        command,
        cwd: workload.directory,
        out: directory,
        native: vue ? null : authorities[role],
        timeoutMs: 15 * 60_000,
        memory,
        ...(probe
          ? {
              probeEnv: {
                VIZE_MAPPER_PROBE_RECORD: join(directory, "projection.jsonl"),
                VIZE_MAPPER_PROBE_DUMP: join(directory, "virtual-ts"),
                VIZE_MAPPER_PROBE_ROOT: workload.directory,
              },
            }
          : {}),
      });
      const runtimeAfter = runtimeProviderView(
        workload.directory,
        hosts[role].vizeCliDir,
        providerBefore,
        probeAuthority,
      );
      writeJson(join(directory, "runtime-providers-after.json"), runtimeAfter);
      assert.deepEqual(
        runtimeAfter,
        runtimeBefore,
        "actual runtime package/probe resolution drift",
      );
      writeJson(join(directory, "project-cache.json"), { removedCaches });
      if (memory) {
        assert.deepEqual(run.receipt.executableErrors, []);
        const nativeExecutables = run.receipt.executablePaths.filter(
          (path) => ![realpathSync(process.execPath), realpathSync("/usr/bin/time")].includes(path),
        );
        assert.ok(nativeExecutables.length > 0, "no actual native host executable sampled");
        for (const path of nativeExecutables) {
          const proof = providerBefore.find((row) => row.path === path);
          assert.ok(proof, `unknown runtime executable: ${path}`);
          assert.equal(sha256(readFileSync(path)), proof.sha256, "runtime host bytes changed");
        }
        if (id !== "tsrs")
          assert.ok(
            nativeExecutables.includes(realpathSync(hostCorsaPath(TYPECHECK_HOSTS_DIR))),
            "TSC/batch must execute the explicit pinned native host",
          );
        memoryQualification.push({
          workload: workload.id,
          lane: `${role}-${id}`,
          qualified: run.receipt.memoryQualified === true,
          memory: run.receipt.memory,
        });
      }
      const evidence = diagnosticEvidence(run.raw, workload.directory, format);
      if (format === "json")
        assertFileCoverage(checkPacket(run.raw.stdout.toString("utf8")), expectedSfcFiles);
      writeJson(join(directory, "diagnostic-evidence.json"), evidence);
      const parsedRun = parseRun(
        { status: run.raw.status, stdout: run.raw.stdout.toString("utf8"), stderr: "" },
        workload.directory,
        format,
      );
      writeJson(join(directory, "statistics.json"), parsedRun.stats);
      const key = `${vue ? "vue" : role}-${id}`;
      if (first.has(key)) assertSameEvidence(first.get(key)!, evidence);
      else first.set(key, evidence);
      if (!vue && workload.generated) {
        const parsed = parsedRun;
        const tool =
          format === "json"
            ? "vize-check"
            : id === "tsrs"
              ? "tsrs-fast-vize-mapper"
              : "tsc-vize-mapper";
        const verdict = gate(tool, parsed.diagnostics, workload.generated.plants);
        writeJson(join(directory, "plants.json"), verdict);
        assert.ok(verdict.ok, "planted diagnostic law failed; timing is not accepted");
      }
      if (!memory && !probe && id !== "batch-1") {
        const key = `${vue ? "vue" : role}-${id}`;
        const sample = {
          phase,
          wallMs: run.receipt.wallMs,
          treeCpuMs: run.receipt.treeCpuMs,
          maxSingleProcessRssKb: run.receipt.gnuTime.maxSingleProcessRssKb,
        };
        samples.set(key, [...(samples.get(key) ?? []), sample]);
      }
      if (probe)
        projection.set(
          vue ? "vue" : role,
          ownedProbeRows(
            readFileSync(join(directory, "projection.jsonl"), "utf8"),
            workload.directory,
            workload.sfcs,
            expectedSfcFiles,
          ),
        );
      assertSameInputs(inputs, snapshotInputs(workload.directory));
      console.log(`${workload.id} ${key} ${phase}: accepted diagnostic/raw/provider gates`);
      return evidence;
    };
    for (const role of ["base", "head"] as const)
      await invoke(role, "tsc", "projection", false, true);
    await invoke("base", "tsc", "projection-reference", false, true, true);
    writeJson(
      join(out, `${workload.id}-projection.json`),
      compareOwnedProjections(
        projection.get("base")!,
        projection.get("head")!,
        projection.get("vue")!,
      ),
    );
    writeJson(
      join(out, `${workload.id}-checker-traces.json`),
      Object.fromEntries(
        ["base", "head", "vue"].map((role) => {
          const directory = join(out, workload.id, `${role}-trace`);
          const busiest = checkerBusyMs(directory);
          assert.ok(busiest.length > 0, "no checker work in the untimed trace");
          return [
            role,
            {
              busiestCheckerMs: busiest,
              perSfcCheckMs: [...checkMsByFile(directory, workload.directory)].sort(([a], [b]) =>
                a.localeCompare(b),
              ),
            },
          ];
        }),
      ),
    );
    for (const phase of requiredSamples(workload.generated == null)) {
      const index = requiredSamples(workload.generated == null).indexOf(phase);
      for (const lane of lanes) {
        for (const role of sampleOrder(index)) await invoke(role, lane.id, phase, false);
        assertSameEvidence(first.get(`base-${lane.id}`)!, first.get(`head-${lane.id}`)!);
      }
    }
    for (const role of ["base", "head"] as const) {
      if (lanes.some((lane) => lane.id === "batch-8")) {
        const one = await invoke(role, "batch-1", "identity-width-control", false);
        assertSameEvidence(one, first.get(`${role}-batch-8`)!);
      }
      for (const lane of lanes) await invoke(role, lane.id, "memory", true);
    }
    const result = {
      workload: specification,
      memory: memoryQualification.filter((row) => row.workload === workload.id),
      samples: Object.fromEntries(samples),
      medians: Object.fromEntries(
        [...samples].map(([key, values]) => [
          key,
          {
            wallMs: median(
              values.filter((row) => row.phase.startsWith("warm-")).map((row) => row.wallMs),
            ),
            treeCpuMs: median(
              values.filter((row) => row.phase.startsWith("warm-")).map((row) => row.treeCpuMs!),
            ),
            maxSingleProcessRssKb: median(
              values
                .filter((row) => row.phase.startsWith("warm-"))
                .map((row) => row.maxSingleProcessRssKb),
            ),
          },
        ]),
      ),
    };
    allResults.push(result);
    writeJson(join(out, "results.json"), allResults);
  }
  const providerAfter = providerClosure(providers);
  writeJson(join(out, "providers-after.json"), providerAfter);
  assert.deepEqual(providerAfter, providerBefore, "provider closure changed during experiment");
  for (const role of ["base", "head"] as const)
    assert.deepEqual(ownedNativeAuthority(builds[role], sources[role]), authorities[role]);
  writeJson(join(out, "experiment.json"), {
    ...experiment,
    qualified: memoryQualification.every((row) => row.qualified),
    diagnosticIdentityQualified: true,
    wallCpuQualified: true,
    memoryQualification,
    providerClosureSha256: sha256(JSON.stringify(providerAfter)),
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
