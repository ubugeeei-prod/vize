/** Fresh source-isolated mapper allocation evidence; no production source overlays. */
import assert from "node:assert/strict";
import { spawnSync, type SpawnSyncOptions } from "node:child_process";
import { createHash, type BinaryLike } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const families = [
  "script-only",
  "native",
  "components",
  "loop-if",
  "slot-ref",
  "refusal",
] as const;
export const counts = [32, 256] as const;
const roles = ["base", "head"] as const;
const modes = ["runtime-off", "allocation"] as const;
const rounds = [1, 2, 3, 4, 5];
type Role = (typeof roles)[number];
export type Sample = {
  role: Role;
  family: (typeof families)[number];
  count: number;
  mode: (typeof modes)[number];
  round: number;
  schema: string;
  sourceRevision: string;
  cold: { generationCalls: number; warmups: number; persistentCache: boolean };
  source: string;
  packet: string;
  elapsedNanoseconds: number;
  peakRssKiB: number;
  allocation: { calls: number; requestedBytes: number; releasedBytes: number; failures: number };
};
type Cell = Pick<Sample, "role" | "family" | "count" | "mode" | "round">;
type Metrics = {
  generationNs: number;
  processPeakRssKiB: number;
  allocationCalls: number;
  requestedBytes: number;
};
type Summary = {
  family: Sample["family"];
  count: number;
  base: Metrics;
  head: Metrics;
  sourceSha256: string;
  packetSha256: string;
  headOverBase: Record<keyof Metrics, number | null>;
};
type CargoArtifact = { reason: string; target: { name: string }; executable: string | null };
type BinaryRecord = {
  source: string;
  sourceRoot: string;
  targetDirectory: string;
  binary: string;
  sha256: string;
  bytes: number;
  artifact: CargoArtifact;
  coreSha256: string;
  publicationSha256: string;
};
const observer = "tools/benchmarks/crates/vize/tests/content_mapper_span_profile.rs";
const core = "crates/vize_canon/src/batch/virtual_project/content_mapper/span_normalize.rs";
const publication = "crates/vize_canon/src/batch/virtual_project/content_mapper/publish.rs";
const hash = (bytes: BinaryLike) => createHash("sha256").update(bytes).digest("hex");
const read = <T>(file: string): T => JSON.parse(readFileSync(file, "utf8")) as T;
const write = (file: string, value: unknown) =>
  writeFileSync(file, JSON.stringify(value, null, 2) + "\n");
const median = (values: number[]) => [...values].sort((a, b) => a - b)[2];
const key = (row: Cell) => [row.role, row.family, row.count, row.mode, row.round].join("-");

function run(
  command: string,
  args: readonly string[],
  options: Pick<SpawnSyncOptions, "cwd" | "env"> = {},
  capture?: string,
) {
  const result = spawnSync(command, args, { maxBuffer: 128 * 1024 * 1024, ...options });
  if (capture) {
    writeFileSync(capture + ".stdout", result.stdout ?? "");
    writeFileSync(capture + ".stderr", result.stderr ?? "");
    write(capture + ".process.json", {
      command,
      args,
      cwd: options.cwd ?? process.cwd(),
      status: result.status,
      signal: result.signal,
      error: result.error?.message ?? null,
    });
  }
  assert.equal(result.error, undefined, capture);
  assert.equal(result.signal, null, capture);
  assert.equal(result.status, 0, capture ?? command);
  return result.stdout.toString("utf8");
}
const git = (cwd: string, ...args: string[]) => run("git", ["-C", cwd, ...args]).trim();

export function summarize(rows: readonly Sample[], binaries: Record<Role, { source: string }>) {
  assert.equal(
    rows.length,
    families.length * counts.length * roles.length * modes.length * rounds.length,
  );
  const indexed = new Map(rows.map((row) => [key(row), row]));
  assert.equal(indexed.size, rows.length, "duplicate or omitted sample");
  const summaries: Summary[] = [];
  for (const family of families)
    for (const count of counts) {
      let reference: { source: string; packet: string } | undefined;
      const metrics: Partial<Record<Role, Metrics>> = {};
      for (const role of roles) {
        const samples: Sample[] = [];
        for (const round of rounds)
          for (const mode of modes) {
            const row = indexed.get(key({ role, family, count, mode, round }));
            assert.ok(row, "missing matrix cell");
            assert.equal(row.schema, "vize.content-mapper-span-profile.v1");
            assert.equal(row.sourceRevision, binaries[role].source);
            assert.deepEqual(row.cold, { generationCalls: 1, warmups: 0, persistentCache: false });
            assert.equal(typeof row.packet, "string");
            assert.equal(typeof row.source, "string");
            const whole = { source: row.source, packet: row.packet };
            reference ??= whole;
            assert.deepEqual(
              whole,
              reference,
              "complete packet bytes differ across sources, modes or repeats",
            );
            assert.ok(Number.isSafeInteger(row.elapsedNanoseconds) && row.elapsedNanoseconds > 0);
            assert.ok(Number.isSafeInteger(row.peakRssKiB) && row.peakRssKiB > 0);
            for (const value of Object.values(row.allocation))
              assert.ok(Number.isSafeInteger(value) && value >= 0);
            assert.equal(row.allocation.failures, 0);
            if (mode === "runtime-off")
              assert.deepEqual(row.allocation, {
                calls: 0,
                requestedBytes: 0,
                releasedBytes: 0,
                failures: 0,
              });
            else {
              assert.ok(row.allocation.calls > 0, "allocation tracking must be enabled");
              assert.ok(row.allocation.requestedBytes > 0, "allocation bytes must be recorded");
            }
            samples.push(row);
          }
        const timed = samples.filter((row) => row.mode === "runtime-off"),
          allocated = samples.filter((row) => row.mode === "allocation");
        metrics[role] = {
          generationNs: median(timed.map((row) => row.elapsedNanoseconds)),
          processPeakRssKiB: median(timed.map((row) => row.peakRssKiB)),
          allocationCalls: median(allocated.map((row) => row.allocation.calls)),
          requestedBytes: median(allocated.map((row) => row.allocation.requestedBytes)),
        };
      }
      assert.ok(reference);
      assert.ok(metrics.base);
      assert.ok(metrics.head);
      const result = {
        family,
        count,
        base: metrics.base,
        head: metrics.head,
        sourceSha256: hash(reference.source),
        packetSha256: hash(reference.packet),
      };
      const headOverBase = Object.fromEntries(
        (Object.keys(result.base) as Array<keyof Metrics>).map((field) => [
          field,
          result.base[field] === 0 ? null : result.head[field] / result.base[field],
        ]),
      ) as Summary["headOverBase"];
      summaries.push({ ...result, headOverBase });
    }
  return summaries;
}

function build(directory: string) {
  mkdirSync(directory);
  const root = process.cwd(),
    head = process.env.SPAN_HEAD,
    eventBase = process.env.SPAN_EVENT_BASE;
  assert.ok(head);
  assert.ok(eventBase);
  for (const sha of [head, eventBase]) assert.match(sha, /^[a-f0-9]{40}$/u);
  assert.equal(git(root, "rev-parse", "HEAD"), head);
  const base = git(root, "merge-base", eventBase, head),
    baseline = join(dirname(directory), "span-base-source");
  run("git", ["worktree", "add", "--detach", baseline, base]);
  copyFileSync(join(root, observer), join(baseline, observer));
  write(join(directory, "provenance.json"), {
    BASE_SHA: base,
    HEAD_SHA: head,
    eventBase,
    mergeBase: base,
    baseParents: git(root, "show", "-s", "--format=%P", base),
    headParents: git(root, "show", "-s", "--format=%P", head),
    driver: head,
    observerSha256: hash(readFileSync(observer)),
    node: process.versions,
    scope:
      "separate ci-opt profiling binaries; only the common observer is copied to baseline; no product source overlays",
  });
  const binaries: Partial<Record<Role, BinaryRecord>> = {};
  for (const role of roles) {
    const cwd = role === "base" ? baseline : root,
      env = {
        ...process.env,
        CARGO_TARGET_DIR: join(dirname(directory), "span-" + role + "-target"),
      };
    assert.equal(
      hash(readFileSync(join(cwd, observer))),
      hash(readFileSync(join(root, observer))),
      "one immutable observer in both source roles",
    );
    assert.equal(
      git(
        cwd,
        "diff",
        "--exit-code",
        "HEAD",
        "--",
        "crates",
        "levels",
        "Cargo.toml",
        "Cargo.lock",
        ".cargo",
      ),
      "",
    );
    copyFileSync(join(cwd, core), join(directory, role + "-span_normalize.rs"));
    copyFileSync(join(cwd, publication), join(directory, role + "-publish.rs"));
    for (const [label, filter] of [
      ["normalization", "span_normalize::tests"],
      ["publication", "content_mapper::publish::tests"],
    ] as const)
      run(
        "cargo",
        [
          "test",
          "--locked",
          "--profile",
          "ci-opt",
          "-p",
          "vize_canon",
          "--lib",
          filter,
          "--",
          "--nocapture",
        ],
        { cwd, env },
        join(directory, role + "-" + label + "-tests"),
      );
    const output = run(
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci-opt",
        "-p",
        "vize_benchmarks",
        "--test",
        "content_mapper_span_profile",
        "--no-run",
        "--message-format=json",
      ],
      { cwd, env },
      join(directory, role + "-build"),
    );
    const artifacts = output
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line) as CargoArtifact)
      .filter(
        (row) =>
          row.reason === "compiler-artifact" &&
          row.target.name === "content_mapper_span_profile" &&
          row.executable,
      );
    assert.equal(artifacts.length, 1);
    const artifact = artifacts[0];
    assert.ok(artifact.executable);
    const bytes = readFileSync(artifact.executable);
    binaries[role] = {
      source: git(cwd, "rev-parse", "HEAD"),
      sourceRoot: cwd,
      targetDirectory: env.CARGO_TARGET_DIR,
      binary: artifact.executable,
      sha256: hash(bytes),
      bytes: bytes.length,
      artifact,
      coreSha256: hash(readFileSync(join(cwd, core))),
      publicationSha256: hash(readFileSync(join(cwd, publication))),
    };
  }
  assert.ok(binaries.base);
  assert.ok(binaries.head);
  assert.notEqual(binaries.base.sha256, binaries.head.sha256);
  write(join(directory, "binaries.json"), binaries);
  for (const [name, command, args] of [
    ["rustc", "rustc", ["-Vv"]],
    ["cpu", "lscpu", []],
    ["kernel", "uname", ["-a"]],
  ] as const)
    run(command, args, {}, join(directory, name));
}

function measure(directory: string) {
  const binaries = read<Record<Role, BinaryRecord>>(join(directory, "binaries.json"));
  const affinity = run("python3", [
    "-c",
    "import os; print(','.join(map(str,sorted(os.sched_getaffinity(0))[:8])))",
  ]).trim();
  write(join(directory, "resources.json"), {
    affinity,
    runner: process.env.RUNNER_NAME,
    label: "blacksmith-32vcpu-ubuntu-2404",
    processPeakRssScope:
      "GNU time process lifetime including Rust harness, fixture generation and packet serialization; KiB; not a generation-only heap peak",
  });
  const rows: Sample[] = [];
  for (const round of rounds)
    for (const family of families)
      for (const count of counts)
        for (const role of round % 2 ? roles : [...roles].reverse())
          for (const mode of modes) {
            const stem = key({ role, family, count, mode, round }),
              output = join(directory, stem + ".json"),
              rss = join(directory, stem + ".rss");
            const env = {
              ...process.env,
              VIZE_SPAN_SOURCE: binaries[role].source,
              VIZE_SPAN_FAMILY: family,
              VIZE_SPAN_COUNT: String(count),
              VIZE_SPAN_MODE: mode,
              VIZE_SPAN_OUTPUT: output,
            };
            run(
              "/usr/bin/time",
              [
                "-f",
                "%M",
                "-o",
                rss,
                "taskset",
                "-c",
                affinity,
                binaries[role].binary,
                "--exact",
                "complete_content_mapper_span_profile",
                "--ignored",
                "--nocapture",
              ],
              { cwd: directory, env },
              join(directory, stem),
            );
            const row = {
              ...read<Omit<Sample, "role" | "round" | "peakRssKiB">>(output),
              role,
              round,
              peakRssKiB: Number(readFileSync(rss, "utf8").trim()),
            };
            writeFileSync(join(directory, family + "-" + count + ".vue"), row.source);
            rows.push(row);
          }
  const summary = summarize(rows, binaries);
  write(join(directory, "samples.json"), rows);
  write(join(directory, "summary.json"), summary);
  writeFileSync(
    join(directory, "summary.md"),
    "Fresh profiling binaries; zero warmups or persisted mapper state. Runtime-off generation timing excludes fixtures and serialization. RSS is the whole process peak. All complete packet bytes match; every family remains separate. No release throughput claim or regression threshold.\n\n| Family | Count | Generation ms base → head | Allocations base → head | Requested bytes base → head | Process RSS KiB base → head |\n| --- | ---: | ---: | ---: | ---: | ---: |\n" +
      summary
        .map(
          (row) =>
            `| ${row.family} | ${row.count} | ${(row.base.generationNs / 1e6).toFixed(3)} → ${(row.head.generationNs / 1e6).toFixed(3)} | ${row.base.allocationCalls} → ${row.head.allocationCalls} | ${row.base.requestedBytes} → ${row.head.requestedBytes} | ${row.base.processPeakRssKiB} → ${row.head.processPeakRssKiB} |`,
        )
        .join("\n") +
      "\n",
  );
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [, , mode, directory] = process.argv;
  assert.ok(directory);
  assert.ok(["build", "measure"].includes(mode));
  (mode === "build" ? build : measure)(resolve(directory));
}
