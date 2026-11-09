/**
 * Planted-diagnostic gate of the real-corpus type-check pair. Copies misskey
 * and n8n once per build, applies every mutation of
 * tools/benchmarks/typecheck-corpora/mutations, checks the copies with tsc
 * and tsrs through that build's Vize mapper, and judges each run against the
 * unmutated diagnostics the measure step wrote to <out>/diagnostics. Vue's
 * mapper under tsc and vue-tsc check the mutated base copy as references.
 *
 *   node typecheck-corpus-mutations-run.ts --base <worktree> --head <worktree> \
 *     --work <dir> --out <dir>
 *
 * Writes mutations.json and mutations.md, appends the table to summary.md,
 * and exits 1 when the head gate fails.
 */

import { spawnSync } from "node:child_process";
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { runTypecheckCommand } from "./typecheck-command.mjs";
import { CORPORA, prepareProject, resolveVueTsc } from "./typecheck-corpus-pair.ts";
import { type Diagnostic, parseHostRun } from "./typecheck-corpus-pair-parse.ts";
import { hostArgs } from "./typecheck-corpus-pair-host.ts";
import {
  type CorpusMutationReport,
  type LaneJudgement,
  type MutationSet,
  type Region,
  gateMutations,
  judgeLane,
  loadMutationSet,
  mutateSources,
  renderMutations,
} from "./typecheck-corpus-mutations.ts";
import {
  CONTENT_MAPPERS,
  MAPPER_HOSTS,
  TYPECHECK_HOSTS_DIR,
  mapperTsconfigName,
  resolveMapperHosts,
} from "./typecheck-mapper-hosts.ts";

const repoRoot = resolve(fileURLToPath(import.meta.url), "..", "..", "..", "..");
type Corpus = (typeof CORPORA)[number];
const mapper = (id: string) => CONTENT_MAPPERS.find((entry) => entry.id === id)!;
const tsc = MAPPER_HOSTS.find((host) => host.id === "tsc")!;

/** The gitlink the driver checkout pins; the set must name the same commit. */
function checkRevision(set: MutationSet, corpus: Corpus): void {
  if (!corpus.source.startsWith(`${set.submodule}/`)) {
    throw new Error(`${set.corpus}: ${corpus.source} is not inside ${set.submodule}`);
  }
  const tree = spawnSync("git", ["-C", repoRoot, "ls-tree", "HEAD", set.submodule], {
    encoding: "utf8",
  });
  const pinned = /^160000 commit ([0-9a-f]{40})\t/u.exec(tree.stdout)?.[1];
  if (pinned !== set.revision) {
    throw new Error(
      `${set.corpus}: the mutations target ${set.revision}, the checkout pins ${pinned ?? tree.stderr}; re-pin tools/benchmarks/typecheck-corpora/mutations/${set.corpus}.json`,
    );
  }
}

function mutatedProject(
  dir: string,
  corpus: Corpus,
  set: MutationSet,
  hosts: ReturnType<typeof resolveMapperHosts>,
): Map<string, Region> {
  prepareProject(dir, corpus, hosts);
  const { files, regions } = mutateSources(set, (file) => readFileSync(join(dir, file), "utf8"));
  for (const [file, text] of files) writeFileSync(join(dir, file), text);
  return regions;
}

function check(
  command: string,
  args: string[],
  cwd: string,
): { diagnostics: Diagnostic[]; ms: number } {
  const result = runTypecheckCommand(command, args, { cwd }) as Parameters<
    typeof parseHostRun
  >[0] & {
    ms: number;
  };
  return { diagnostics: parseHostRun(result, cwd).diagnostics, ms: result.ms };
}

function main(): void {
  const { values } = parseArgs({
    options: {
      base: { type: "string" },
      head: { type: "string" },
      work: { type: "string" },
      out: { type: "string" },
    },
  });
  if (!values.base || !values.head || !values.work || !values.out) {
    throw new Error("--base, --head, --work and --out are required");
  }
  const builds = {
    base: resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(resolve(values.base), "npm", "cli")),
    head: resolveMapperHosts(TYPECHECK_HOSTS_DIR, join(resolve(values.head), "npm", "cli")),
  };
  const work = resolve(values.work);
  const out = resolve(values.out);
  const diagnosticsDir = join(out, "diagnostics");
  mkdirSync(diagnosticsDir, { recursive: true });
  const pristine = (lane: string) =>
    JSON.parse(readFileSync(join(diagnosticsDir, `${lane}.json`), "utf8")) as Diagnostic[];
  const vueTsc = resolveVueTsc();
  let hostMs = 0;
  const judge = (
    lane: string,
    set: MutationSet,
    regions: Map<string, Region>,
    run: { diagnostics: Diagnostic[]; ms: number },
  ): LaneJudgement => {
    hostMs += run.ms;
    writeFileSync(
      join(diagnosticsDir, `${lane}-mutated.json`),
      `${JSON.stringify(run.diagnostics, null, 1)}\n`,
    );
    const judgement = judgeLane(set, regions, run.diagnostics, pristine(lane));
    const caught = Object.values(judgement.plants).filter((plant) => plant.count === 1).length;
    console.log(`${lane}: ${caught}/${set.mutations.length} plants exactly once`);
    return judgement;
  };

  const reports: CorpusMutationReport[] = [];
  const failures: string[] = [];
  const flips: string[] = [];
  for (const corpus of CORPORA) {
    const set = loadMutationSet(corpus.id);
    checkRevision(set, corpus);
    const vize: CorpusMutationReport["vize"] = { base: {}, head: {} };
    let references: CorpusMutationReport["references"] = {};
    for (const role of ["base", "head"] as const) {
      const dir = join(work, `${corpus.id}-${role}-mutated`);
      const regions = mutatedProject(dir, corpus, set, builds[role]);
      for (const host of MAPPER_HOSTS) {
        const args = hostArgs(host, dir, mapperTsconfigName(mapper("vize")));
        const lane = `${corpus.id}-${host.id}-vize-${role}`;
        vize[role][host.id] = judge(lane, set, regions, check(builds.base[host.id], args, dir));
      }
      if (role !== "base") continue;
      const vueArgs = hostArgs(tsc, dir, mapperTsconfigName(mapper("vue")));
      const vueTscArgs = ["--max-old-space-size=16384", vueTsc.bin, "--noEmit"];
      vueTscArgs.push(
        "--extendedDiagnostics",
        "--pretty",
        "false",
        "-p",
        join(dir, "tsconfig.json"),
      );
      references = {
        "Vue mapper": judge(
          `${corpus.id}-tsc-vue`,
          set,
          regions,
          check(builds.base.tsc, vueArgs, dir),
        ),
        "vue-tsc": judge(
          `${corpus.id}-vue-tsc`,
          set,
          regions,
          check(process.execPath, vueTscArgs, dir),
        ),
      };
    }
    const verdict = gateMutations(set, vize.base, vize.head);
    failures.push(...verdict.failures);
    flips.push(...verdict.flips);
    reports.push({ set, vize, references });
  }
  const markdown = renderMutations(reports, { failures, flips });
  const cost = `Mutated runs: ${(hostMs / 1000).toFixed(1)} s host wall in total.\n`;
  writeFileSync(
    join(out, "mutations.json"),
    `${JSON.stringify({ reports, failures, flips }, null, 1)}\n`,
  );
  writeFileSync(join(out, "mutations.md"), `${markdown}\n${cost}`);
  appendFileSync(join(out, "summary.md"), `\n${markdown}\n${cost}`);
  for (const flip of flips) console.log(`flip: ${flip}`);
  if (failures.length > 0) {
    console.error(failures.join("\n"));
    process.exitCode = 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
