/**
 * `vize lsp` against Vue's official language tools on real projects, driven
 * over stdio by one scripted editing session.
 *
 * Every editor gets the same session on the same prepared project, in fresh
 * processes with a fresh `HOME`/`XDG_CACHE_HOME` (cold for on-disk caches; the
 * OS page cache stays warm after the unrecorded warmup). Rounds rotate the
 * fixture and editor order so load drift hits all of them. Reports cold start,
 * process-tree CPU and PSS/RSS over the session, CPU per edit, idle CPU and
 * p50/p99 latency of diagnostics and requests, counting only correct answers;
 * fixtures on one size curve add each metric against SFC count.
 *
 * Usage:
 *   node tools/benchmarks/scripts/lsp-vs-volar.ts --fixture misskey \
 *     --vize head=target/ci-opt/vize --vize base=/tmp/vize-base --tsgo <tsgo> \
 *     --volar-root <dir with node_modules/{@vue/language-server,@vue/typescript-plugin,typescript}> \
 *     --fixtures-dir /tmp/projects --rounds 3 --cycles 10 --out lsp.md --json lsp.json
 *   node tools/benchmarks/scripts/lsp-vs-volar.ts --plan --fixture misskey --fixture scale-1k
 *     (prints the CI jobs: one per fixture, one per size curve)
 */

import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { type Editor, TsgoMapperEditor, VizeEditor } from "./lsp-vs-volar-editors.ts";
import { VolarEditor } from "./lsp-vs-volar-volar.ts";
import {
  FIXTURES,
  type FixtureSpec,
  installContentMapper,
  plantProbes,
  prepareFixture,
} from "./lsp-vs-volar-fixtures.ts";
import { curveMarkdown, curvesOf, planJobs } from "./lsp-vs-volar-curve.ts";
import { markdown, summarize } from "./lsp-vs-volar-report.ts";
import { runSession, type SessionOptions, type SessionResult } from "./lsp-vs-volar-session.ts";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

type EditorFactory = {
  label: string;
  create: (workspace: string, env: Record<string, string>) => Editor;
  /** Per-session project setup this editor needs; returns the restore step. */
  setup?: (workspace: string, spec: FixtureSpec) => () => void;
};

function pairs(
  values: string[] | undefined,
  separator: string,
  what: string,
): Array<[string, string]> {
  return (values ?? []).map((value) => {
    const at = value.indexOf(separator);
    if (at <= 0) throw new Error(`${what} must be <name>${separator}<value>: ${value}`);
    return [value.slice(0, at), value.slice(at + 1)];
  });
}

const rotate = <T>(items: T[], shift: number) => [
  ...items.slice(shift % items.length),
  ...items.slice(0, shift % items.length),
];

function positiveInteger(value: string | undefined, fallback: number, what: string): number {
  const parsed = value == null ? fallback : Number(value);
  if (!Number.isSafeInteger(parsed) || parsed < 0)
    throw new Error(`${what} must be a non-negative integer`);
  return parsed;
}

async function main() {
  const { values } = parseArgs({
    options: {
      fixture: { type: "string", multiple: true },
      plan: { type: "boolean" },
      vize: { type: "string", multiple: true },
      "vize-env": { type: "string", multiple: true },
      tsgo: { type: "string" },
      "volar-root": { type: "string" },
      "tsgo-mapper": { type: "string" },
      "tsgo-mapper-version": { type: "string" },
      "mapper-modules": { type: "string" },
      "fixtures-dir": { type: "string" },
      rounds: { type: "string" },
      warmups: { type: "string" },
      cycles: { type: "string" },
      "idle-ms": { type: "string" },
      "sample-ms": { type: "string" },
      "warm-budget-s": { type: "string" },
      meta: { type: "string", multiple: true },
      out: { type: "string" },
      logs: { type: "string" },
      json: { type: "string" },
    },
  });
  const fixtures = (values.fixture ?? []).map((id) => {
    const spec = FIXTURES.find((candidate) => candidate.id === id);
    if (!spec)
      throw new Error(
        `unknown fixture ${id}; known: ${FIXTURES.map((item) => item.id).join(", ")}`,
      );
    return spec;
  });
  if (!fixtures.length) throw new Error("pass at least one --fixture");
  if (values.plan) {
    console.log(JSON.stringify(planJobs(fixtures)));
    return;
  }
  const vizeEnv = new Map<string, Record<string, string>>();
  for (const [label, assignment] of pairs(values["vize-env"], ":", "--vize-env")) {
    const [key, value] = pairs([assignment], "=", "--vize-env")[0];
    vizeEnv.set(label, { ...vizeEnv.get(label), [key]: value });
  }
  const factories: EditorFactory[] = pairs(values.vize, "=", "--vize").map(([label, binary]) => ({
    label: `vize ${label}`,
    create: (workspace, env) =>
      new VizeEditor(`vize ${label}`, resolve(binary), {
        workspace,
        env: {
          ...env,
          ...(values.tsgo ? { CORSA_PATH: resolve(values.tsgo) } : {}),
          ...vizeEnv.get(label),
        },
      }),
  }));
  if (values["volar-root"]) {
    const root = resolve(values["volar-root"]);
    const version = (name: string) =>
      (
        JSON.parse(readFileSync(join(root, "node_modules", name, "package.json"), "utf8")) as {
          version: string;
        }
      ).version;
    const label = `volar ${version("@vue/language-server")} (ts ${version("typescript")})`;
    factories.push({
      label,
      create: (workspace, env) => new VolarEditor(label, { root }, { workspace, env }),
    });
  }
  if (values["tsgo-mapper"]) {
    if (!values["mapper-modules"]) throw new Error("--tsgo-mapper needs --mapper-modules");
    const tsgo = resolve(values["tsgo-mapper"]);
    const modules = resolve(values["mapper-modules"]);
    const version = (name: string) =>
      (JSON.parse(readFileSync(join(modules, name, "package.json"), "utf8")) as { version: string })
        .version;
    const label = `tsgo ${values["tsgo-mapper-version"] ?? "?"} + vize ${version("vize")} mapper`;
    factories.push({
      label,
      create: (workspace, env) => new TsgoMapperEditor(label, tsgo, { workspace, env }),
      setup: (workspace, spec) => installContentMapper(workspace, spec, modules),
    });
  }
  if (factories.length < 2)
    throw new Error("compare at least two editors (--vize, --volar-root, --tsgo-mapper)");
  const options: SessionOptions = {
    cycles: positiveInteger(values.cycles, 5, "--cycles"),
    coldTimeoutMs: 600_000,
    diagnosticsTimeoutMs: 60_000,
    requestTimeoutMs: 60_000,
    warmBudgetMs: positiveInteger(values["warm-budget-s"], 1_200, "--warm-budget-s") * 1000,
    sampleIntervalMs: positiveInteger(values["sample-ms"], 100, "--sample-ms"),
    idleMs: positiveInteger(values["idle-ms"], 5_000, "--idle-ms"),
  };
  const rounds = positiveInteger(values.rounds, 3, "--rounds");
  const warmups = positiveInteger(values.warmups, 1, "--warmups");
  const meta = Object.fromEntries(pairs(values.meta, "=", "--meta"));
  const fixturesDir = resolve(values["fixtures-dir"] ?? join(os.tmpdir(), "vize-lsp-bench"));

  const all: Record<string, { sessions: SessionResult[]; crashes: string[] }> = {};
  const write = () => {
    const sections = Object.entries(all)
      .filter(([, run]) => run.sessions.length || run.crashes.length)
      .map(([id, { sessions, crashes }]) =>
        markdown(
          id,
          summarize(sessions),
          sessions,
          { ...meta, rounds: String(rounds), cycles: String(options.cycles) },
          crashes,
        ),
      );
    const curves = curvesOf(fixtures, (id) => all[id]?.sessions ?? []);
    for (const { name, ids, curve } of curves) sections.push(curveMarkdown(name, ids, curve));
    const text = ["# vize lsp vs Vue language tools", "", ...sections].join("\n");
    if (values.out) writeFileSync(values.out, text);
    if (values.json) {
      const summaries = Object.fromEntries(
        Object.entries(all).map(([id, run]) => [id, summarize(run.sessions)]),
      );
      writeFileSync(
        values.json,
        `${JSON.stringify({ meta, options, summaries, curves, runs: all }, null, 1)}\n`,
      );
    }
    return text;
  };
  const prepared = fixtures.map((spec) => {
    const { root: workspace, original } = prepareFixture(repoRoot, spec, fixturesDir);
    const filePath = join(workspace, spec.file);
    all[spec.id] = { sessions: [], crashes: [] };
    return { spec, workspace, filePath, original, planted: plantProbes(spec, original) };
  });
  type Prepared = (typeof prepared)[number];
  /** A session that throws (a crashed or hung server) is reported, and the run goes on. */
  const once = async (
    { spec, workspace, filePath, planted }: Prepared,
    factory: EditorFactory,
    round: number,
    session: SessionOptions,
  ) => {
    const run = all[spec.id];
    const home = mkdtempSync(join(os.tmpdir(), "lsp-bench-home-"));
    const env = { HOME: home, XDG_CACHE_HOME: join(home, ".cache") };
    const restore = factory.setup?.(workspace, spec);
    try {
      const result = await runSession(
        factory.create(workspace, env),
        spec,
        workspace,
        filePath,
        planted,
        session,
        round,
      );
      console.error(
        `${spec.id} round ${round} ${factory.label}: cold ${result.cold.totalMs.toFixed(0)}ms, ${result.failures.length} failures`,
      );
      if (round > 0) run.sessions.push(result);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      console.error(`${spec.id} round ${round} ${factory.label} crashed: ${message}`);
      run.crashes.push(
        `${factory.label}, round ${round}: session aborted: ${message.slice(0, 2000)}`,
      );
    } finally {
      restore?.();
      // The server's own log (Vize writes it under XDG_CACHE_HOME) explains where time went.
      const log = join(home, ".cache/vize/logs/lsp.log");
      if (values.logs && round === 1 && existsSync(log)) {
        mkdirSync(values.logs, { recursive: true });
        const name = `${spec.id}-${factory.label.replace(/[^a-z0-9.]+/gi, "-")}-round${round}.log`;
        copyFileSync(log, join(values.logs, name));
      }
      rmSync(home, { recursive: true, force: true });
    }
  };
  // Importers read the edited SFC from disk; every editor sees the same planted file.
  for (const fixture of prepared) writeFileSync(fixture.filePath, fixture.planted);
  try {
    for (const fixture of prepared) {
      for (let warmup = 0; warmup < warmups; warmup += 1) {
        for (const factory of factories)
          await once(fixture, factory, 0, { ...options, cycles: 1, idleMs: 0 });
      }
    }
    // Rounds interleave fixtures too, so drift over a long job cannot pose as a size effect.
    for (let round = 1; round <= rounds; round += 1) {
      for (const fixture of rotate(prepared, round - 1)) {
        for (const factory of rotate(factories, round - 1)) {
          await once(fixture, factory, round, options);
          write();
        }
      }
    }
  } finally {
    for (const fixture of prepared) writeFileSync(fixture.filePath, fixture.original);
  }
  const text = write();
  if (!values.out) console.log(text);
  // Evidence is complete only when every session ran; a crash fails the job after writing it.
  if (Object.values(all).some((run) => run.crashes.length)) process.exitCode = 1;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
