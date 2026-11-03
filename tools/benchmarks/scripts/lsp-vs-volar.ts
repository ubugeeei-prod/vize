/**
 * `vize lsp` against Vue's official language tools on real projects, driven
 * over stdio by one scripted editing session.
 *
 * Every editor gets the same session on the same prepared project, in fresh
 * processes with a fresh `HOME`/`XDG_CACHE_HOME` (cold for on-disk caches; the
 * OS page cache stays warm after the unrecorded warmup). Rounds rotate the
 * editor order so load drift hits all of them. Reports cold start, process-tree
 * CPU and PSS/RSS over the session, CPU per edit, idle CPU and p50/p99 latency
 * of diagnostics and requests, counting only correct answers.
 *
 * Usage:
 *   node tools/benchmarks/scripts/lsp-vs-volar.ts --fixture misskey \
 *     --vize head=target/ci-opt/vize --vize base=/tmp/vize-base --tsgo <tsgo> \
 *     --volar-root <dir with node_modules/{@vue/language-server,@vue/typescript-plugin,typescript}> \
 *     --fixtures-dir /tmp/projects --rounds 3 --cycles 10 --out lsp.md --json lsp.json
 */

import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { type Editor, VizeEditor } from "./lsp-vs-volar-editors.ts";
import { VolarEditor } from "./lsp-vs-volar-volar.ts";
import { FIXTURES, plantProbes, prepareFixture } from "./lsp-vs-volar-fixtures.ts";
import { markdown, summarize } from "./lsp-vs-volar-report.ts";
import { runSession, type SessionOptions, type SessionResult } from "./lsp-vs-volar-session.ts";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

type EditorFactory = {
  label: string;
  create: (workspace: string, env: Record<string, string>) => Editor;
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
      vize: { type: "string", multiple: true },
      "vize-env": { type: "string", multiple: true },
      tsgo: { type: "string" },
      "volar-root": { type: "string" },
      "fixtures-dir": { type: "string" },
      rounds: { type: "string" },
      warmups: { type: "string" },
      cycles: { type: "string" },
      "idle-ms": { type: "string" },
      "sample-ms": { type: "string" },
      meta: { type: "string", multiple: true },
      out: { type: "string" },
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
  if (factories.length < 2)
    throw new Error("compare at least two editors (--vize ... and/or --volar-root)");
  const options: SessionOptions = {
    cycles: positiveInteger(values.cycles, 10, "--cycles"),
    diagnosticsTimeoutMs: 180_000,
    requestTimeoutMs: 60_000,
    sampleIntervalMs: positiveInteger(values["sample-ms"], 100, "--sample-ms"),
    idleMs: positiveInteger(values["idle-ms"], 5_000, "--idle-ms"),
  };
  const rounds = positiveInteger(values.rounds, 3, "--rounds");
  const warmups = positiveInteger(values.warmups, 1, "--warmups");
  const meta = Object.fromEntries(pairs(values.meta, "=", "--meta"));
  const fixturesDir = resolve(values["fixtures-dir"] ?? join(os.tmpdir(), "vize-lsp-bench"));

  const report: string[] = ["# vize lsp vs Vue language tools", ""];
  const all: Record<string, SessionResult[]> = {};
  for (const spec of fixtures) {
    const workspace = prepareFixture(repoRoot, spec, fixturesDir);
    const filePath = join(workspace, spec.file);
    const original = readFileSync(filePath, "utf8");
    const planted = plantProbes(spec, original);
    const sessions: SessionResult[] = [];
    // Importers read the edited SFC from disk; both editors see the same planted file.
    writeFileSync(filePath, planted);
    try {
      const once = async (factory: EditorFactory, round: number, session: SessionOptions) => {
        const home = mkdtempSync(join(os.tmpdir(), "lsp-bench-home-"));
        const env = { HOME: home, XDG_CACHE_HOME: join(home, ".cache") };
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
          return result;
        } finally {
          rmSync(home, { recursive: true, force: true });
        }
      };
      for (let warmup = 0; warmup < warmups; warmup += 1) {
        for (const factory of factories)
          await once(factory, 0, { ...options, cycles: 1, idleMs: 0 });
      }
      for (let round = 1; round <= rounds; round += 1) {
        const shift = (round - 1) % factories.length;
        for (const factory of [...factories.slice(shift), ...factories.slice(0, shift)]) {
          sessions.push(await once(factory, round, options));
        }
      }
    } finally {
      writeFileSync(filePath, original);
    }
    all[spec.id] = sessions;
    report.push(
      markdown(spec.id, summarize(sessions), sessions, {
        ...meta,
        rounds: String(rounds),
        cycles: String(options.cycles),
      }),
    );
  }
  const text = report.join("\n");
  if (values.out) writeFileSync(values.out, text);
  else console.log(text);
  if (values.json) {
    const summaries = Object.fromEntries(
      Object.entries(all).map(([id, sessions]) => [id, summarize(sessions)]),
    );
    writeFileSync(
      values.json,
      `${JSON.stringify({ meta, options, summaries, sessions: all }, null, 1)}\n`,
    );
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
