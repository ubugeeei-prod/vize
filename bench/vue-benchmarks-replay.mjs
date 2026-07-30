#!/usr/bin/env node
/**
 * Replay the MIT-licensed pikax/vue-benchmarks typecheck confirmation corpus
 * against one vize binary (#3283).
 *
 * Every upstream case is scored against its own meta.json expectations, then
 * compared with this repo's pinned expectation table. Any drift fails the
 * replay: a newly failing case is a regression, and a case that starts
 * passing while the table still expects "fail" is a stale suppression that
 * must be removed. Missing vize or tsgo is an explicit error, never a pass.
 */

import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const benchDir = dirname(fileURLToPath(import.meta.url));
const rootDir = dirname(benchDir);

export const UPSTREAM = {
  repository: "pikax/vue-benchmarks",
  url: "https://github.com/pikax/vue-benchmarks",
  commit: "02c0dac76075924bfb8e9b6985e51a9795ff6909",
  license: "MIT",
};

/**
 * Upstream capability tags for the vize-check tool (scripts/confirm suite):
 * plants whose `requires` fall outside this list are recorded as explicit
 * skips, never as silent passes.
 */
const VIZE_CAPABILITIES = new Set([
  "script-types",
  "template-bindings",
  "prop-types",
  "v-if-narrow",
  "emits",
  "v-model",
  "dom-events",
]);

function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i++) {
    if (!argv[i].startsWith("--")) continue;
    const key = argv[i].slice(2);
    const next = argv[i + 1];
    if (next == null || next.startsWith("--")) {
      args[key] = "true";
    } else {
      args[key] = next;
      i++;
    }
  }
  return args;
}

function run(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    cwd: options.cwd,
    encoding: "utf8",
    env: { ...process.env, NO_COLOR: "1", ...options.env },
    maxBuffer: 64 * 1024 * 1024,
    timeout: options.timeout ?? 300_000,
  });
  if (result.error) throw result.error;
  return { status: result.status, stdout: result.stdout ?? "", stderr: result.stderr ?? "" };
}

/** Fail closed: an explicitly pinned binary never falls back to another. */
function requireBinary(label, explicitPath, fallbacks) {
  const candidates = (explicitPath ? [explicitPath] : fallbacks).map((c) => resolve(c));
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) {
    throw new Error(`replay: ${label} not found (looked at: ${candidates.join(", ")})`);
  }
  const probe = run(found, ["--version"]);
  if (probe.status !== 0) throw new Error(`replay: ${label} at ${found} failed --version`);
  return { path: found, version: (probe.stdout || probe.stderr).trim().split("\n")[0] };
}

/** Clone (or reuse) the upstream corpus and pin it to the recorded commit. */
function ensureUpstream(upstreamDir, workRoot) {
  const dir = upstreamDir ? resolve(upstreamDir) : join(workRoot, "vue-benchmarks");
  if (!existsSync(join(dir, ".git"))) {
    rmSync(dir, { recursive: true, force: true });
    const clone = run("git", ["clone", "--quiet", UPSTREAM.url, dir]);
    if (clone.status !== 0) throw new Error(`replay: git clone failed\n${clone.stderr}`);
  }
  const checkout = run("git", ["checkout", "--quiet", UPSTREAM.commit], { cwd: dir });
  if (checkout.status !== 0) {
    throw new Error(`replay: cannot checkout ${UPSTREAM.commit}\n${checkout.stderr}`);
  }
  const head = run("git", ["rev-parse", "HEAD"], { cwd: dir }).stdout.trim();
  if (head !== UPSTREAM.commit) {
    throw new Error(`replay: upstream is at ${head}, expected ${UPSTREAM.commit}`);
  }
  return dir;
}

function resolveVuePackageDir() {
  const candidates = [
    join(benchDir, "node_modules", "vue"),
    join(rootDir, "node_modules", "vue"),
    join(rootDir, "tests", "node_modules", "vue"),
  ];
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) throw new Error("replay: vue package not found in any node_modules");
  return found;
}

/**
 * Materialize one upstream case the way the upstream confirm suite does
 * (shared env.d.ts + tsconfig.base with `paths` pinned at a real vue), minus
 * golar-specific scaffolding, so `vize check` sees the same project shape.
 */
function prepareCase(upstreamDir, caseId, workRoot, vuePackageDir) {
  const source = join(upstreamDir, "tests/confirm/fixtures/typecheck/cases", caseId);
  const shared = join(upstreamDir, "tests/confirm/fixtures/typecheck/_shared");
  const dest = join(workRoot, "cases", caseId);
  rmSync(dest, { recursive: true, force: true });
  mkdirSync(dest, { recursive: true });
  cpSync(source, dest, { recursive: true });
  cpSync(join(shared, "env.d.ts"), join(dest, "env.d.ts"));

  const tsconfig = JSON.parse(readFileSync(join(shared, "tsconfig.base.json"), "utf8"));
  const vuePath = vuePackageDir.replaceAll("\\", "/");
  tsconfig.compilerOptions = {
    ...tsconfig.compilerOptions,
    paths: { vue: [vuePath], "vue/*": [`${vuePath}/*`] },
  };
  tsconfig.exclude = ["node_modules"];
  writeFileSync(join(dest, "tsconfig.json"), `${JSON.stringify(tsconfig, null, 2)}\n`);
  writeFileSync(
    join(dest, "package.json"),
    `${JSON.stringify({ name: `replay-${caseId}`, private: true, type: "module" }, null, 2)}\n`,
  );
  const meta = JSON.parse(readFileSync(join(source, "meta.json"), "utf8"));
  return { dest, meta };
}

/** Score one vize JSON report against upstream meta expectations. */
export function scoreCase(meta, status, report, combined) {
  if (report == null) {
    return { outcome: "fail", detail: `no JSON report (status=${status})` };
  }
  const errors = report.errorCount;
  if (meta.mustNotMatch?.some((needle) => combined.includes(needle))) {
    return { outcome: "fail", detail: "output contained a forbidden pattern" };
  }
  if (meta.expectErrors) {
    const minErrors = meta.minErrors ?? 1;
    if (errors < minErrors) {
      return { outcome: "fail", detail: `expected >=${minErrors} error(s), got ${errors}` };
    }
    if (meta.mustMatch?.length && !meta.mustMatch.some((needle) => combined.includes(needle))) {
      return {
        outcome: "fail",
        detail: `diagnostics did not match any of: ${meta.mustMatch.join(" | ")}`,
      };
    }
    return { outcome: "pass", detail: `caught ${errors} error(s)` };
  }
  if (errors > 0) {
    return { outcome: "fail", detail: `expected clean, got ${errors} error(s)` };
  }
  return { outcome: "pass", detail: "clean" };
}

/** Compare replay outcomes with the pinned expectation table. */
export function diffExpectations(results, expectations) {
  const problems = [];
  for (const result of results) {
    const expected = expectations[result.caseId];
    if (expected == null) {
      problems.push(`${result.caseId}: not in the expectation table (add "${result.outcome}")`);
      continue;
    }
    if (expected === result.outcome) continue;
    if (expected === "fail" && result.outcome === "pass") {
      problems.push(
        `${result.caseId}: stale suppression — the case now passes; remove the "fail" expectation`,
      );
    } else {
      problems.push(
        `${result.caseId}: expected ${expected}, got ${result.outcome} (${result.detail})`,
      );
    }
  }
  for (const caseId of Object.keys(expectations)) {
    if (!results.some((result) => result.caseId === caseId)) {
      problems.push(`${caseId}: in the expectation table but missing from the corpus`);
    }
  }
  return problems;
}

export async function main(argv = process.argv.slice(2)) {
  const args = parseArgs(argv);
  const workRoot = resolve(args["work-root"] ?? join(rootDir, "target", "vue-benchmarks-replay"));
  mkdirSync(workRoot, { recursive: true });

  const vize = requireBinary("vize binary", args["vize-bin"], [
    join(rootDir, "target", "release", "vize"),
  ]);
  const tsgo = requireBinary("tsgo binary", process.env.VIZE_REPLAY_TSGO, [
    join(rootDir, "node_modules", ".bin", "tsgo"),
    join(rootDir, "tests", "node_modules", ".bin", "tsgo"),
  ]);
  const vuePackageDir = resolveVuePackageDir();
  const upstreamDir = ensureUpstream(args.upstream, workRoot);
  const casesRoot = join(upstreamDir, "tests/confirm/fixtures/typecheck/cases");
  const caseIds = readdirSync(casesRoot, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => entry.name)
    .sort();
  if (caseIds.length === 0) throw new Error(`replay: no cases found under ${casesRoot}`);

  const results = [];
  for (const caseId of caseIds) {
    const { dest, meta } = prepareCase(upstreamDir, caseId, workRoot, vuePackageDir);
    const requires = meta.requires ?? [];
    if (!requires.every((capability) => VIZE_CAPABILITIES.has(capability))) {
      results.push({
        caseId,
        outcome: "skip",
        detail: `requires ${requires.join(", ")} (outside the vize capability set)`,
      });
      continue;
    }
    const check = run(
      vize.path,
      [
        "check",
        ".",
        "--quiet",
        "--format",
        "json",
        "--tsconfig",
        "tsconfig.json",
        "--corsa-path",
        tsgo.path,
      ],
      { cwd: dest },
    );
    let report = null;
    try {
      report = JSON.parse(check.stdout);
    } catch {
      report = null;
    }
    const combined = `${check.stdout}\n${check.stderr}`;
    results.push({ caseId, ...scoreCase(meta, check.status, report, combined) });
  }

  const data = {
    schemaVersion: 1,
    kind: "vue-benchmarks-replay",
    generatedAt: new Date().toISOString(),
    upstream: UPSTREAM,
    versions: { vize: vize.version, tsgo: tsgo.version },
    caseCount: results.length,
    results,
  };
  if (args.json) writeFileSync(resolve(args.json), `${JSON.stringify(data, null, 2)}\n`);
  for (const result of results) {
    console.log(`${result.outcome.padEnd(5)} ${result.caseId} — ${result.detail}`);
  }

  if (args.expect) {
    const expectations = JSON.parse(readFileSync(resolve(args.expect), "utf8"));
    const problems = diffExpectations(results, expectations);
    if (problems.length > 0) {
      throw new Error(`replay: expectation drift:\n- ${problems.join("\n- ")}`);
    }
    console.log(`replay: ${results.length} cases match ${args.expect}`);
  } else {
    console.log("replay: no --expect table given; report-only run");
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    await main();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}
