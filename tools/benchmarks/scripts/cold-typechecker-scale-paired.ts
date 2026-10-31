/** Exact-source normal A/B commands; full emitted virtual-code parity is untimed. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { run } from "./canon-instance-directives-io.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import {
  captureColdScale,
  prepareColdScale,
  verifyColdDiagnostics,
  coldScaleSupportScriptFiles,
} from "./cold-typechecker-scale.ts";
import { coldScalePacketView } from "./cold-typechecker-scale-packet.ts";
import { linuxPeakRss } from "./cold-typechecker-scale-phase-baseline.ts";
import { pairedColdStatistics } from "./cold-typechecker-scale-paired-stats.ts";
import { verifyColdScaleBuildPair } from "./cold-typechecker-scale-build.ts";

const hash = (file: string) => createHash("sha256").update(readFileSync(file)).digest("hex");
type Build = {
  sourceSha: string;
  treeSha: string;
  binary: string;
  binarySha256: string;
  allocation: boolean;
  manifests: unknown;
};
type Sample = {
  role: string;
  order: number;
  wallMs: number;
  peakRssBytes: number;
  directory: string;
};

function retainInputs(source: string, destination: string) {
  mkdirSync(destination, { recursive: true });
  for (const entry of readdirSync(source, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.isSymbolicLink()) continue;
    if (entry.isDirectory()) retainInputs(join(source, entry.name), join(destination, entry.name));
    else copyFileSync(join(source, entry.name), join(destination, entry.name));
  }
}

export function pairedColdComparison(options: {
  provider: string;
  out: string;
  baseReceipt: string;
  headReceipt: string;
}) {
  const { provider, out } = options;
  verifyColdScaleBuildPair(options.baseReceipt, options.headReceipt, process.cwd());
  const readBuild = (file: string) => JSON.parse(readFileSync(file, "utf8")) as Build;
  const base = readBuild(options.baseReceipt),
    head = readBuild(options.headReceipt);
  assert.equal(base.allocation, false);
  assert.equal(head.allocation, false);
  assert.deepEqual(
    base.manifests,
    head.manifests,
    "dependency and manifest changes are outside this comparison",
  );
  const providerLock = hash(join(provider, "package-lock.json"));
  const files = 2500 + coldScaleSupportScriptFiles;
  const parityDirectory = join(out, "virtual-output");
  mkdirSync(join(parityDirectory, "tmp"), { recursive: true });
  const corpus = prepareColdScale(join(parityDirectory, "project"), provider, files, false);
  retainInputs(join(parityDirectory, "project"), join(out, "authored-inputs"));
  writeFileSync(join(parityDirectory, "corpus.json"), JSON.stringify(corpus, null, 2) + "\n");
  let completeOutput: unknown;
  for (const [role, build] of [
    ["base", base],
    ["head", head],
  ] as const) {
    for (const generated of [
      join(parityDirectory, "xdg-cache"),
      join(parityDirectory, "project/node_modules/.vize"),
      join(parityDirectory, "tmp"),
    ])
      rmSync(generated, { recursive: true, force: true });
    mkdirSync(join(parityDirectory, "tmp"), { recursive: true });
    const { command } = coldScaleCommand({
      provider,
      out: parityDirectory,
      app: corpus.app,
      tool: "cli",
      binary: build.binary,
    });
    const captured = run(
      parityDirectory,
      role,
      command.executable,
      [...command.args, "--show-virtual-ts"],
      corpus.app,
      { ...process.env, ...command.env },
    );
    assert.equal(captured.status, 0);
    verifyColdDiagnostics(captured.stdout.toString(), "cli", files, false);
    const value: unknown = JSON.parse(captured.stdout.toString());
    if (completeOutput === undefined) completeOutput = value;
    else
      assert.deepEqual(
        value,
        completeOutput,
        "full Virtual TS and complete result changed at the same authored project path",
      );
  }
  rmSync(join(parityDirectory, "project"), { recursive: true });
  const normalPacket = (directory: string) => {
    const census = JSON.parse(readFileSync(join(directory, "corpus.json"), "utf8")) as {
      app: string;
      entries: { path: string }[];
    };
    return coldScalePacketView(JSON.parse(readFileSync(join(directory, "stdout.txt"), "utf8")), {
      ...census,
      root: join(directory, "project"),
    });
  };
  let expected: unknown;
  const pairs: { pair: number; base: Sample; head: Sample }[] = [];
  for (let pair = 1; pair <= 40; pair++) {
    const collected = new Map<string, Sample>();
    const order = pair % 2 ? ["base", "head"] : ["head", "base"];
    for (const [index, role] of order.entries()) {
      const build = role === "base" ? base : head,
        directory = join(out, `${pair}-${role}`);
      const measurement = captureColdScale({
        provider,
        out: directory,
        files,
        tool: "cli",
        invalid: false,
        binary: build.binary,
      });
      assert.equal(measurement.corpusSha256, corpus.corpusSha256);
      const packet = normalPacket(directory);
      if (expected === undefined) expected = packet;
      else assert.deepEqual(packet, expected, "complete timed command result changed");
      copyFileSync(
        join(directory, "project/apps/web/tsconfig.json"),
        join(directory, "tsconfig.json"),
      );
      collected.set(role, {
        role,
        order: index + 1,
        wallMs: measurement.wallMs,
        peakRssBytes: linuxPeakRss(readFileSync(join(directory, "stderr.txt"), "utf8")),
        directory,
      });
      // The exact common authored corpus is retained above; each sample keeps
      // its census, config and complete raw command result, not runtime caches.
      for (const name of ["project", "xdg-cache", "tmp"])
        rmSync(join(directory, name), { recursive: true, force: true });
    }
    pairs.push({ pair, base: collected.get("base")!, head: collected.get("head")! });
  }
  let negative: unknown;
  for (const [role, build] of [
    ["base", base],
    ["head", head],
  ] as const) {
    const directory = join(out, `invalid-${role}`);
    captureColdScale({
      provider,
      out: directory,
      files,
      tool: "cli",
      invalid: true,
      binary: build.binary,
    });
    const packet = normalPacket(directory);
    if (negative === undefined) negative = packet;
    else assert.deepEqual(packet, negative, "complete invalid-result vector changed");
  }
  for (const tool of ["mapper", "vue-tsc"])
    for (const invalid of [false, true])
      captureColdScale({
        provider,
        out: join(out, `${invalid ? "invalid" : "valid"}-${tool}`),
        files,
        tool,
        invalid,
      });
  assert.equal(hash(join(provider, "package-lock.json")), providerLock);
  assert.equal(hash(base.binary), base.binarySha256);
  assert.equal(hash(head.binary), head.binarySha256);
  const summary = {
    source: { base, head },
    providerLock,
    corpusSha256: corpus.corpusSha256,
    files,
    sfcs: 2500,
    pairs,
    wallMs: pairedColdStatistics(
      pairs.map((pair) => ({ base: pair.base.wallMs, head: pair.head.wallMs })),
    ),
    peakRssBytes: pairedColdStatistics(
      pairs.map((pair) => ({ base: pair.base.peakRssBytes, head: pair.head.peakRssBytes })),
    ),
    rssScope: "GNU time child high-water RSS; not simultaneous process-tree RSS",
    fullVirtualOutput:
      "same authored project path; every complete JSON field including all Virtual TS equal without normalization",
    timedOutput:
      "only declared filename identities mapped through each exact authored census; every other field and order exact",
    fullVirtualOutputSha256: createHash("sha256")
      .update(JSON.stringify(completeOutput))
      .digest("hex"),
    allCorrectnessGuardsPassed: true,
    coldCacheQualified: false,
    comparisonQualified: true,
  };
  writeFileSync(join(out, "paired-comparison.json"), JSON.stringify(summary, null, 2) + "\n");
  console.log(
    JSON.stringify({
      base: base.sourceSha,
      head: head.sourceSha,
      wallMs: summary.wallMs,
      peakRssBytes: summary.peakRssBytes,
      allCorrectnessGuardsPassed: true,
    }),
  );
  return summary;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      provider: { type: "string" },
      out: { type: "string" },
      base: { type: "string" },
      head: { type: "string" },
    },
  });
  assert.ok(values.provider && values.out && values.base && values.head);
  pairedColdComparison({
    provider: resolve(values.provider),
    out: resolve(values.out),
    baseReceipt: resolve(values.base),
    headReceipt: resolve(values.head),
  });
}
