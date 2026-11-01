/** One fixed comparative campaign. No PR/schedule automatically launches this entry point. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";
import { availableParallelism, cpus, totalmem } from "node:os";
import {
  command,
  environment,
  affinity,
  type Command,
} from "./typecheck-vs-vue-tsc-frozen-command.ts";
import {
  compareFindings,
  diagnostics,
  hash,
  identity,
  orders,
  save,
  source,
  statusConsistent,
  assessStderr,
  assertStatusStable,
} from "./typecheck-vs-vue-tsc-frozen-evidence.ts";
import {
  prepareProjects,
  verifyProvider,
  verifySource,
  type Project,
  cacheFacts,
  inputSnapshot,
  assertInputs,
} from "./typecheck-vs-vue-tsc-frozen-projects.ts";
import { collect, forwarder } from "./typecheck-vs-vue-tsc-frozen-native.ts";
type Tool = "vize" | "vue-tsc";
export function specification(
  project: Project,
  tool: Tool,
  binary: string,
  provider: string,
  native: string,
): Command {
  return {
    cwd: project.cwd,
    executable: tool === "vize" ? binary : process.execPath,
    args:
      tool === "vize"
        ? [
            "check",
            "--no-config",
            "--format",
            "json",
            "--tsconfig",
            project.tsconfig,
            "--corsa-path",
            native,
          ]
        : [
            path.join(provider, "node_modules/vue-tsc/bin/vue-tsc.js"),
            "--noEmit",
            "--pretty",
            "false",
            "-p",
            project.tsconfig,
          ],
  };
}
export function controlSource(invalid: boolean) {
  const expression = invalid
    ? "value.definitelyMissing + acceptNumber('bad')"
    : "value.length + acceptNumber(1)";
  const nodes = [
    `  <div>{{ ${expression} }}</div>`,
    ...Array.from(
      { length: 62 },
      (_, i) => `  <span :title="value">{{ value.length + ${i} }}</span>`,
    ),
    `  <div>{{ ${expression} }}</div>`,
  ];
  return (
    '<script setup lang="ts">\nconst value: string = "ok";\nfunction acceptNumber(n: number): number { return n; }\n</script>\n<template>\n' +
    nodes.join("\n") +
    "\n</template>\n"
  );
}
export function assertControl(
  value: ReturnType<typeof diagnostics>,
  control: string,
  invalid: boolean,
) {
  const rows = value.findings.filter((row) => row.file === control);
  if (!invalid) assert.deepEqual(rows, []);
  else {
    assert.equal(
      rows.length,
      4,
      "both wrong argument and unknown member at first AND final real template nodes",
    );
    assert.deepEqual(
      rows
        .map((row) => [row.line, row.code])
        .sort((a, b) => Number(a[0]) - Number(b[0]) || Number(a[1]) - Number(b[1])),
      [
        [6, 2339],
        [6, 2345],
        [69, 2339],
        [69, 2345],
      ],
    );
  }
}
function median(values: number[]) {
  const sorted = values.toSorted((a, b) => a - b);
  return (sorted[1] + sorted[2]) / 2;
}
export async function campaign(options: {
  out: string;
  sourceRoot: string;
  binary: string;
  provider: string;
}) {
  assert.equal(process.platform, "linux");
  assert.equal(process.arch, "x64");
  assert.equal(process.version, "v24.14.0");
  fs.mkdirSync(options.out); // Unique owned attempt before first evidence write.
  const { out, sourceRoot, binary, provider } = options;
  const closure = verifySource(sourceRoot, provider),
    frozenProvider = verifyProvider(provider);
  const native = frozenProvider.native.path,
    vize = identity(binary);
  const build = JSON.parse(fs.readFileSync(binary + ".build.json", "utf8"));
  assert.equal(build.sourceSha, source);
  assert.equal(build.binarySha256, vize.sha256);
  assert.equal(build.allocation, false);
  const cpuset = affinity();
  save(path.join(out, "closure.json"), {
    closure,
    frozenProvider,
    build,
    vize,
    cpuset,
    hardware: { cpus: cpus(), availableParallelism: availableParallelism(), totalmem: totalmem() },
    controller: { sha: process.env.GITHUB_SHA, ref: process.env.GITHUB_REF },
    population: {
      cohorts: 2,
      perCohort: {
        nativeObserver: 1,
        incumbentListFiles: 1,
        correctness: 8,
        freshNormal: 8,
        discardedWarmup: 2,
        cacheReusedNormal: 8,
      },
      outerCommands: 56,
      setupCommandsSeparate: true,
      forwardedNativeChildrenCount: "observed separately; not inferred from outer count",
    },
    knownLimits: [
      "cross-engine comparison; no Rust/Go attribution",
      "OS page cache uncontrolled",
      "50ms RSS sampling is not PSS/exact process-tree peak",
      "loaded native file vector unobserved; requested/materialized potential inputs are separately sealed",
      "no original Nuxt956/WSL or universal10x claim",
      "historical raw624TS5097/status2 versus authoredCLI0 remains separate",
    ],
  });
  const projects = prepareProjects(out, provider),
    summaries = [],
    allTransactions: unknown[] = [];
  const observer = forwarder(out);
  async function invoke(
    project: Project,
    tool: Tool,
    name: string,
    cache: string,
    overrides: NodeJS.ProcessEnv = {},
    checker = native,
    extra: string[] = [],
  ) {
    const directory = path.join(out, name),
      env = { ...environment(cache), ...overrides };
    const spec = specification(project, tool, binary, provider, checker);
    spec.args.push(...extra);
    const measured = await command(directory, spec, env);
    const stdout = fs.readFileSync(path.join(directory, "stdout.bin"), "utf8"),
      stderr = fs.readFileSync(path.join(directory, "stderr.bin"), "utf8");
    const parsed = extra.includes("--listFilesOnly")
      ? null
      : diagnostics(tool, stdout, project.cwd);
    const result = {
      name,
      project: project.id,
      tool,
      command: spec,
      measurement: measured,
      diagnostics: parsed,
      statusConsistent: parsed ? statusConsistent(measured.status, parsed) : measured.status === 0,
      stderrAssessment: {
        ...assessStderr(stderr),
        raw: identity(path.join(directory, "stderr.bin")),
      },
    };
    save(path.join(directory, "result.json"), result);
    allTransactions.push(result);
    save(
      path.join(out, `transaction-${String(allTransactions.length).padStart(3, "0")}.json`),
      result,
    );
    return result;
  }
  for (const project of projects) {
    const cache = cacheFacts(project, provider),
      inputsBefore = inputSnapshot(project.root, provider);
    save(path.join(out, project.id + "-cache-facts.json"), cache);
    save(path.join(out, project.id + "-inputs-before.json"), inputsBefore);
    const prefix = project.id,
      observed = path.join(out, prefix + "-calls");
    fs.mkdirSync(observed);
    const proof = await invoke(
      project,
      "vize",
      prefix + "-native-proof",
      path.join(out, prefix + "-proof-cache"),
      {
        VIZE_COLD_NATIVE: native,
        VIZE_COLD_CAPTURE: observed,
        VIZE_COLD_PROJECT: project.root,
        VIZE_COLD_PROVIDER: provider,
      },
      observer,
    );
    const nativeProof = collect(observed, native, project.root, provider);
    const listed = await invoke(
      project,
      "vue-tsc",
      prefix + "-incumbent-inventory",
      path.join(out, prefix + "-inventory-cache"),
      {},
      native,
      ["--listFilesOnly"],
    );
    const fileVector = fs
      .readFileSync(path.join(out, listed.name, "stdout.bin"), "utf8")
      .split(/\r?\n/u)
      .filter(Boolean);
    save(path.join(out, prefix + "-incumbent-files.json"), {
      fileVector,
      scope:
        "separate untimed --listFilesOnly program enumeration, not proof of which files a timed checker visited",
      stderr: listed.stderrAssessment,
    });
    assert(!fs.existsSync(project.control), "no overwrite of original authored input");
    const controls = new Map<
      Tool,
      { diagnostics: ReturnType<typeof diagnostics>; status: number | null }[]
    >();
    try {
      for (const [phase, invalid] of [
        ["valid", false],
        ["invalid", true],
        ["repair", false],
        ["restore", true],
      ] as const) {
        const body = controlSource(invalid);
        fs.writeFileSync(project.control, body);
        save(path.join(out, prefix + "-control-" + phase + ".json"), {
          file: project.control,
          body,
          bytes: Buffer.byteLength(body),
          sha256: hash(body),
          templateNodes: 64,
          firstRealNodeLine: 6,
          finalRealNodeLine: 69,
        });
        for (const tool of ["vize", "vue-tsc"] as const) {
          const result = await invoke(
            project,
            tool,
            prefix + "-control-" + phase + "-" + tool,
            path.join(out, prefix + "-control-cache-" + phase + "-" + tool),
          );
          assert(result.diagnostics);
          assertControl(result.diagnostics, project.control, invalid);
          if (invalid)
            assert.notEqual(
              result.measurement.status,
              0,
              "explicit authored errors must fail the CLI",
            );
          assert.equal(result.diagnostics.unclassified.length, 0);
          const values = controls.get(tool) ?? [];
          values.push({ diagnostics: result.diagnostics, status: result.measurement.status });
          controls.set(tool, values);
        }
      }
      for (const [tool, values] of controls) {
        assert.deepEqual(values[0], values[2], tool + " whole repair packet/vector");
        assert.deepEqual(values[1], values[3], tool + " whole restored-error packet/vector");
      }
    } finally {
      fs.unlinkSync(project.control);
    }
    const afterControls = inputSnapshot(project.root, provider);
    save(path.join(out, prefix + "-inputs-after-controls.json"), afterControls);
    assertInputs(inputsBefore, afterControls, cache.excludedOwnedOutputs);
    const timed: Awaited<ReturnType<typeof invoke>>[] = [],
      warmups: typeof timed = [];
    for (const arm of ["fresh-app-cache", "process-restart-cache-reused"] as const) {
      const warmCache = Object.fromEntries(
        (["vize", "vue-tsc"] as const).map((tool) => [
          tool,
          path.join(out, prefix + "-warm-cache-" + tool),
        ]),
      );
      if (arm === "process-restart-cache-reused")
        for (const tool of ["vize", "vue-tsc"] as const)
          warmups.push(
            await invoke(project, tool, prefix + "-discarded-warmup-" + tool, warmCache[tool]),
          );
      for (const [pair, order] of orders.entries())
        for (const tool of order) {
          const name = `${prefix}-${arm}-${pair}-${tool}`;
          const result = await invoke(
            project,
            tool,
            name,
            arm === "fresh-app-cache" ? path.join(out, name + "-cache") : warmCache[tool],
          );
          timed.push(result);
          assertStatusStable(
            timed.find((row) => row.tool === tool)!.measurement.status,
            result.measurement.status,
          );
          assert.deepEqual(
            result.diagnostics,
            proof.diagnostics && tool === "vize"
              ? proof.diagnostics
              : timed.find((row) => row.tool === tool)!.diagnostics,
            "same-tool whole normal packets cannot drift",
          );
          assert.deepEqual(identity(binary), vize);
          assert.equal(identity(native).sha256, frozenProvider.native.sha256);
        }
    }
    const inputsAfter = inputSnapshot(project.root, provider);
    save(path.join(out, prefix + "-inputs-after-timing.json"), inputsAfter);
    assertInputs(inputsBefore, inputsAfter, cache.excludedOwnedOutputs);
    const first = (tool: Tool) => timed.find((row) => row.tool === tool)!;
    const comparison = compareFindings(
      first("vize").diagnostics!.findings,
      first("vue-tsc").diagnostics!.findings,
    );
    const rows = ["fresh-app-cache", "process-restart-cache-reused"].map((arm) => {
      const tools = (["vize", "vue-tsc"] as const).map((tool) => {
        const values = timed.filter((row) => row.tool === tool && row.name.includes(arm));
        assert.equal(values.length, 4);
        return {
          tool,
          rawSampleNames: values.map((row) => row.name),
          medianWallMs: median(values.map((row) => row.measurement.wallMs)),
          medianCommandCpuMs: median(values.map((row) => row.measurement.cpuMs)),
          medianChildHighWaterKiB: median(values.map((row) => row.measurement.childHighWaterKiB)),
          medianObservedSimultaneousTreeHighWaterKiB: median(
            values.map((row) =>
              Math.max(
                0,
                ...row.measurement.samples.map((sample) => sample.simultaneousObservedTreeRssKiB),
              ),
            ),
          ),
        };
      });
      return {
        arm,
        tools,
        descriptiveWallRatioVizeOverVueTsc: tools[0].medianWallMs / tools[1].medianWallMs,
        applicationCacheQualified: false,
        cacheScope:
          arm === "fresh-app-cache"
            ? "fresh HOME/XDG/TMP only; project-declared incremental outputs may remain; actual cache trees retained"
            : "same HOME/XDG/TMP after one discarded process; no same-process native Program reuse",
        osColdQualified: false,
        programReusedInProcess: false,
      };
    });
    const qualified =
      comparison.qualified &&
      nativeProof.complete &&
      nativeProof.nativeClean &&
      timed.every(
        (row) =>
          row.diagnostics!.unclassified.length === 0 &&
          row.statusConsistent &&
          row.stderrAssessment.unresolvedLines.length === 0,
      );
    const summary = {
      project: project.id,
      comparison,
      nativeProof: {
        complete: nativeProof.complete,
        nativeClean: nativeProof.nativeClean,
        observedMainCalls: nativeProof.observedMainCalls,
        refusals: nativeProof.refusals,
      },
      fileInventory: identity(path.join(out, prefix + "-incumbent-files.json")),
      rows,
      warmupsDiscarded: warmups.map((row) => row.name),
      qualifiedHeadline: false,
      preliminaryChecksPass: qualified,
      pendingFinalClosure: true,
      tenTimesFasterClaim: false,
      requiredHumanClassification: comparison.unmatched,
      scope:
        "PROVISIONAL/UNQUALIFIED until final source/provider closure; all raw/descriptive rows retained",
    };
    summaries.push(summary);
    save(path.join(out, prefix + "-summary.json"), summary);
  }
  assert.equal(allTransactions.length, 56);
  verifySource(sourceRoot, provider);
  verifyProvider(provider);
  assert.deepEqual(identity(binary), vize);
  const finalSummaries = summaries.map((row) => ({
    ...row,
    qualifiedHeadline: row.preliminaryChecksPass,
    pendingFinalClosure: false,
    scope: row.preliminaryChecksPass
      ? "this frozen cross-engine cohort only"
      : "UNQUALIFIED descriptive observations; unmatched findings/native debt remain, no speed headline",
  }));
  save(path.join(out, "summary.json"), {
    source,
    summaries: finalSummaries,
    completeOuterTransactions: allTransactions.length,
    favorableRepeat: false,
  });
  const markdown =
    "# Frozen typecheck comparison\n\n" +
    finalSummaries
      .map(
        (row) =>
          `- ${row.project}: headline qualified=${row.qualifiedHeadline}; unmatched=${row.comparison.unmatched.length}. See complete raw summary.json.\n`,
      )
      .join("") +
    "\nJS/native engines differ. OS cache is uncontrolled. All raw command/resource packets and first/final-node controls are retained. No universal speedup claim.\n";
  fs.writeFileSync(path.join(out, "summary.md"), markdown, { flag: "wx" });
  return finalSummaries;
}
if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      out: { type: "string" },
      "source-root": { type: "string" },
      binary: { type: "string" },
      provider: { type: "string" },
    },
  });
  assert(values.out && values["source-root"] && values.binary && values.provider);
  const options = {
    out: path.resolve(values.out),
    sourceRoot: path.resolve(values["source-root"]),
    binary: path.resolve(values.binary),
    provider: path.resolve(values.provider),
  };
  try {
    await campaign(options);
  } catch (error) {
    if (fs.existsSync(options.out))
      save(path.join(options.out, "failure.json"), {
        error: String(error),
        stack: error instanceof Error ? error.stack : null,
        favorableRetryAllowed: false,
      });
    throw error;
  }
}
