import { normalHostArguments } from "./template-alias-paired-11822-hosts.ts";
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { cpus, availableParallelism, release, totalmem } from "node:os";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  realpathSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";
import { materializeCorpus } from "./template-alias-paired-11822-corpus.ts";
import { verifiedSourcePair } from "./template-alias-paired-11822-source.ts";
import {
  assertCampaignEnvironment,
  remainingBudget,
} from "./template-alias-paired-11822-process.ts";
import { observeMemory } from "./template-alias-paired-11822-memory.ts";
import { qualifySemantics } from "./template-alias-paired-11822-semantics.ts";
import { attestSource } from "./template-alias-paired-11822-attest.ts";
import {
  collectTransforms,
  qualifyCapturedLifecycle,
} from "./template-alias-paired-11822-protocol.ts";
import {
  capture,
  closureSnapshot,
  diagnostics,
  evidence,
  git,
  inventory,
  json,
  metrics,
  prepareWorkspace,
  pristine,
  write,
} from "./template-alias-paired-11822-custody.ts";
import type { DiagnosticPacket, Observation } from "./template-alias-paired-11822-types.ts";

export function median(values: number[]): number {
  assert.ok(values.length > 0 && values.length % 2 === 1);
  assert.ok(values.every((value) => Number.isFinite(value) && value >= 0));
  return [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
}

export function population(observations: Observation[], expected: number) {
  assert.equal(observations.length, expected, "incomplete primary population has no median");
  assert.ok(expected % 2 === 1);
  return {
    wallSeconds: median(observations.map((row) => row.usage.wall_s)),
    waitedUserSeconds: median(observations.map((row) => row.usage.user_s)),
    waitedSystemSeconds: median(observations.map((row) => row.usage.system_s)),
    maximumSingleProcessRssKiB: median(observations.map((row) => row.usage.max_rss_kib)),
  };
}

function host(provider: string, name: "typescript" | "tsrs", version: string) {
  const root = realpathSync(join(provider, "node_modules", name));
  assert.equal(json(join(root, "package.json")).version, version);
  const require = createRequire(join(root, "package.json"));
  const platform =
    name === "typescript" ? "@typescript/typescript-linux-x64" : "@tsrs/linux-x64-gnu";
  const nativeRoot = dirname(require.resolve(platform + "/package.json"));
  assert.equal(json(join(nativeRoot, "package.json")).version, version);
  const binaries = readdirSync(nativeRoot, { recursive: true, withFileTypes: true })
    .filter((entry) => entry.isFile() && ["tsc", "tsgo", "tsrs"].includes(entry.name))
    .map((entry) => join(entry.parentPath, entry.name))
    .filter((path) =>
      readFileSync(path)
        .subarray(0, 4)
        .equals(Buffer.from([127, 69, 76, 70])),
    );
  assert.equal(binaries.length, 1, "one pinned native host image required");
  const bytes = readFileSync(binaries[0]);
  assert.equal(bytes[4], 2);
  assert.equal(bytes[5], 1);
  assert.equal(bytes.readUInt16LE(18), 62);
  const command = join(root, "bin", name === "typescript" ? "tsc" : "tsrs");
  return { name, version, command, launcher: evidence(command), image: evidence(binaries[0]) };
}

export function emittedPacket(
  row: Observation,
  workspace: string,
  expected: Parameters<typeof diagnostics>[2],
) {
  assert.equal(row.status, 1, "the complete negative oracle must exit with diagnostics");
  const stdout = readFileSync(row.stdout.path, "utf8").replaceAll(workspace + "/", "");
  const stderr = readFileSync(row.stderr.path, "utf8").replaceAll(workspace + "/", "");
  const packet: DiagnosticPacket = {
    status: row.status,
    diagnostics: diagnostics(stdout + "\n" + stderr, workspace, expected),
  };
  return { packet, stdout, stderr };
}

export async function main() {
  assertCampaignEnvironment();
  assert.equal(process.version, "v26.12.1");
  const { values } = parseArgs({
    options: {
      base: { type: "string" },
      head: { type: "string" },
      provider: { type: "string" },
      output: { type: "string" },
      "base-sha": { type: "string" },
      "head-sha": { type: "string" },
      "driver-sha": { type: "string" },
    },
  });
  for (const key of [
    "base",
    "head",
    "provider",
    "output",
    "base-sha",
    "head-sha",
    "driver-sha",
  ] as const)
    assert.ok(values[key], key);
  const base = resolve(values.base!),
    head = resolve(values.head!),
    provider = resolve(values.provider!),
    output = resolve(values.output!);
  const driver = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
  const sources = {
    base: pristine(base, values["base-sha"]!),
    head: pristine(head, values["head-sha"]!),
  };
  const requireCurrentMain = () =>
    assert.equal(
      git(base, "ls-remote", "origin", "refs/heads/main").split(/\s+/u)[0],
      sources.base.sha,
      "public main advanced: rebind source tuple before dispatch",
    );
  requireCurrentMain();
  git(head, "merge-base", "--is-ancestor", sources.base.sha, sources.head.sha);
  const driverIdentity = pristine(driver, values["driver-sha"]!);
  // This campaign isolates dispatcher admission. Projection/checker/SDK changes require a new qualification design.
  const changed = git(head, "diff", "--name-only", sources.base.sha, sources.head.sha)
    .split("\n")
    .filter(Boolean);
  assert.ok(changed.includes("crates/vize/src/commands/content_mapper/dispatch.rs"));
  assert.ok(
    changed.every(
      (path) =>
        path.startsWith(".github/") ||
        path.startsWith("tests/") ||
        path.startsWith("tools/") ||
        [
          "crates/vize/src/commands/content_mapper/dispatch.rs",
          "crates/vize/src/commands/content_mapper/dispatch/tests.rs",
        ].includes(path),
    ),
    "candidate contains production changes outside dispatcher admission",
  );
  const builds = verifiedSourcePair(base, head, sources);
  mkdirSync(output);
  for (const [id, build] of Object.entries(builds)) attestSource(output, id, build);
  const corpus = join(output, "authored-corpus");
  const manifest = materializeCorpus(corpus);
  const authored = inventory(join(corpus, "src"));
  const paths = authored.filter((row) => row.path.endsWith(".vue")).map((row) => "src/" + row.path);
  const providerBefore = closureSnapshot([provider]);
  const hosts = [
    host(provider, "typescript", "7.1.0-beta"),
    host(provider, "tsrs", "0.21.0-ts7.1.0-beta"),
  ];
  assert.equal(json(join(provider, "node_modules/vue/package.json")).version, "3.5.44");
  const context = {
    schema: "vize.public-dispatcher-campaign.v1",
    driver: driverIdentity,
    sources,
    changed,
    builds,
    hosts,
    publicInputs:
      "newly authored 2500 SFCs; independent public npm lock; no historical/private corpus inheritance",
    corpus: manifest,
    provider: providerBefore,
    environment: {
      node: process.version,
      platform: process.platform,
      arch: process.arch,
      release: release(),
      cpus: cpus(),
      availableParallelism: availableParallelism(),
      totalmem: totalmem(),
      cpuMax: readFileSync("/sys/fs/cgroup/cpu.max", "utf8"),
      affinity: process.env.TEMPLATE_ALIAS_AFFINITY,
    },
    primary: {
      pairs: 5,
      modes: ["normal"],
      cache: [
        "fresh process / empty owned cache and temp",
        "fresh process / same workspace and reused owned cache and temp",
      ],
      persistentWarmProcess: false,
      osPageCache: "unmanaged",
      cpuWidths: [availableParallelism()],
      memory: "GNU time maximum single process RSS, separate sampled tree memory runs required",
    },
    diagnosticIdentity: {
      status: true,
      code: true,
      authoredStart: true,
      multilineMessage: true,
      duplicates: true,
      rawOrder: true,
      end: "unavailable from CLI",
      related: "unavailable from CLI",
      fullAuthoredRangeQualification: false,
    },
  };
  write(join(output, "context.json"), context);
  const rows: Record<string, Observation[]> = {};
  const packets = new Map<string, ReturnType<typeof emittedPacket>>();
  for (let pair = 0; pair < 5; pair++) {
    const variants = pair % 2 ? ["candidate", "currentmain"] : ["currentmain", "candidate"];
    for (const host of hosts)
      for (const id of variants) {
        const directory = join(output, `primary-${host.name}-${id}-${pair}`);
        const workspace = join(directory, "workspace");
        mkdirSync(directory);
        const closure = prepareWorkspace(corpus, provider, workspace, {
          id: id as "currentmain" | "candidate",
          package: builds[id].cliPackage,
        });
        write(join(directory, "workspace.json"), closure);
        for (const cachePolicy of ["empty-owned-cache", "reused-owned-cache"] as const) {
          const key = host.name + "/" + id + "/" + cachePolicy;
          const row = await capture(
            join(directory, cachePolicy),
            process.execPath,
            normalHostArguments(host.command, closure.configFile),
            workspace,
            {},
            cachePolicy === "reused-owned-cache",
          );
          const packet = emittedPacket(row, workspace, manifest.expectedErrors);
          if (packets.has(host.name))
            assert.deepEqual(
              packet,
              packets.get(host.name),
              "complete same-host emitted diagnostics or order changed",
            );
          else packets.set(host.name, packet);
          (rows[key] ??= []).push(row);
          write(join(directory, cachePolicy, "qualified.json"), {
            diagnostic: packet,
            timingEligible: true,
            cachePolicy,
            freshProcess: true,
            persistentWarmProcess: false,
            osPageCache: "unmanaged",
          });
          assert.deepEqual(inventory(join(workspace, "src")), authored);
        }
      }
  }
  const projections = new Map<string, unknown>();
  for (const host of hosts)
    for (const id of ["currentmain", "candidate"] as const) {
      const directory = join(output, `capture-${host.name}-${id}`);
      mkdirSync(directory);
      const workspace = join(directory, "workspace"),
        wire = join(directory, "wire");
      const closure = prepareWorkspace(
        corpus,
        provider,
        workspace,
        { id, package: builds[id].cliPackage },
        wire,
      );
      const row = await capture(
        join(directory, "observation"),
        process.execPath,
        normalHostArguments(host.command, closure.configFile),
        workspace,
      );
      assert.deepEqual(
        emittedPacket(row, workspace, manifest.expectedErrors),
        packets.get(host.name),
      );
      collectTransforms(wire, workspace, join(directory, "projections"), paths);
      write(join(directory, "lifecycle.json"), qualifyCapturedLifecycle(wire));
      const projection = paths.sort().map((path) => {
        const matches = [".ts", ".tsx", ".js", ".jsx"]
          .map((extension) => join(directory, "projections", path + extension))
          .filter(existsSync);
        assert.equal(matches.length, 1);
        const metadata = json<{ result: unknown }>(matches[0] + ".metadata.json").result;
        return {
          path,
          text: readFileSync(matches[0], "utf8").replaceAll(workspace + "/", "$WORKSPACE/"),
          metadata: JSON.parse(
            JSON.stringify(metadata).replaceAll(workspace + "/", "$WORKSPACE/"),
          ) as unknown,
        };
      });
      if (projections.has(host.name))
        assert.deepEqual(
          projection,
          projections.get(host.name),
          "whole transform text/maps/metadata changed",
        );
      else projections.set(host.name, projection);
      write(join(directory, "qualified.json"), {
        timingEligible: false,
        completeTransforms: paths.length,
        projectionEquality: true,
      });
    }
  await qualifySemantics({
    corpus,
    provider,
    output,
    hosts,
    packages: Object.fromEntries(
      Object.entries(builds).map(([id, build]) => [id, build.cliPackage]),
    ),
  });
  for (const host of hosts)
    for (const id of ["currentmain", "candidate"] as const) {
      const directory = join(output, `phase-${host.name}-${id}`),
        workspace = join(directory, "workspace");
      mkdirSync(directory);
      const closure = prepareWorkspace(corpus, provider, workspace, {
        id,
        package: builds[id].cliPackage,
      });
      const row = await capture(
        join(directory, "observation"),
        process.execPath,
        [...normalHostArguments(host.command, closure.configFile), "--extendedDiagnostics"],
        workspace,
      );
      assert.equal(row.status, 1);
      assert.deepEqual(
        {
          status: row.status,
          diagnostics: diagnostics(row.text, workspace, manifest.expectedErrors),
        },
        packets.get(host.name)!.packet,
      );
      write(join(directory, "qualified.json"), {
        timingEligible: false,
        phase: metrics(row.text),
        note:
          host.name === "typescript"
            ? "official extendedDiagnostics performs additional GC; reported memory is terminal Alloc, not peak or RSS"
            : "TSRS counters are its own implementation; no official GC or memory semantics inferred",
      });
    }
  const memory: Record<string, Awaited<ReturnType<typeof observeMemory>>[]> = {};
  for (let pair = 0; pair < 3; pair++)
    for (const host of hosts)
      for (const id of pair % 2 ? ["candidate", "currentmain"] : ["currentmain", "candidate"]) {
        const directory = join(output, `memory-${host.name}-${id}-${pair}`),
          workspace = join(directory, "workspace");
        mkdirSync(directory);
        const closure = prepareWorkspace(corpus, provider, workspace, {
          id: id as "currentmain" | "candidate",
          package: builds[id].cliPackage,
        });
        const cache = join(workspace, ".empty-cache"),
          temporary = join(workspace, ".empty-tmp");
        mkdirSync(cache);
        mkdirSync(temporary);
        const prefix = join(directory, "observation");
        const row = await observeMemory({
          command: [process.execPath, ...normalHostArguments(host.command, closure.configFile)],
          cwd: workspace,
          output: prefix,
          hostImage: host.image.path,
          mapperImage: builds[id].image.path,
          timeoutMilliseconds: Math.min(300_000, remainingBudget()),
          environment: { XDG_CACHE_HOME: cache, TMPDIR: temporary },
        });
        const stdout = readFileSync(prefix + ".stdout", "utf8").replaceAll(workspace + "/", ""),
          stderr = readFileSync(prefix + ".stderr", "utf8").replaceAll(workspace + "/", "");
        const packet = {
          packet: {
            status: row.status,
            diagnostics: diagnostics(stdout + "\n" + stderr, workspace, manifest.expectedErrors),
          },
          stdout,
          stderr,
        };
        assert.deepEqual(
          packet,
          packets.get(host.name),
          "memory run diagnostic/status identity changed",
        );
        (memory[host.name + "/" + id] ??= []).push(row);
        write(join(directory, "qualified.json"), {
          timingEligible: false,
          fullDiagnosticIdentity: packet,
          observedTreeMemoryEligible: true,
        });
      }
  const summary = Object.fromEntries(
    Object.entries(rows).map(([key, observations]) => [key, population(observations, 5)]),
  );
  assert.deepEqual(closureSnapshot([provider]), providerBefore);
  verifiedSourcePair(base, head, sources);
  requireCurrentMain();
  assert.deepEqual(inventory(join(corpus, "src")), authored);
  copyFileSync(join(provider, "package-lock.json"), join(output, "public-provider-lock.json"));
  write(join(output, "primary-summary.json"), {
    summary,
    timingComplete: true,
    memory: Object.fromEntries(
      Object.entries(memory).map(([key, observations]) => {
        assert.equal(observations.length, 3, "incomplete memory population");
        return [
          key,
          {
            observedTreeRssKiB: median(observations.map((row) => row.maximumObservedTreeRssKiB)),
            observedTreePssKiB: median(observations.map((row) => row.maximumObservedTreePssKiB)),
            scope: observations[0].scope,
          },
        ];
      }),
    ),
    correctnessComplete: true,
    acceptedPerformanceConclusion: false,
    acceptance:
      "raw results require source-bound peer review; persistent warm-process and real-project scope remain separate",
    coldWarmScope: "fresh processes; unmanaged OS caches; no persistent warm cohort",
  });
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  main().catch((error: unknown) => {
    console.error(error);
    process.exitCode = 1;
  });
