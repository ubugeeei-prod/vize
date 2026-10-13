import { spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const benchDir = dirname(fileURLToPath(import.meta.url));
export const rootDir = resolve(benchDir, "..", "..", "..");

export const UPSTREAM = {
  repository: "pikax/vue-benchmarks",
  url: "https://github.com/pikax/vue-benchmarks",
  commit: "65c6102504b14cd49c0b03305be8dd0b9d208c59",
  license: "MIT",
};

/**
 * These are the upstream typecheck rows that are closest to Vize's `check`
 * surface. Keep them in the replay artifact so the local evidence tracks the
 * same tools shown in https://github.com/pikax/vue-benchmarks/blob/main/docs/typecheck.md.
 */
export const DEFAULT_REPLAY_TOOLS = ["vize", "verter-tsc", "golar-typecheck", "golar-default"];

export function parseArgs(argv) {
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

export function parseToolList(value) {
  const requested = (value ?? DEFAULT_REPLAY_TOOLS.join(","))
    .split(",")
    .map((tool) => tool.trim())
    .filter(Boolean);
  const unknown = requested.filter((tool) => !DEFAULT_REPLAY_TOOLS.includes(tool));
  if (unknown.length > 0) {
    throw new Error(`replay: unknown tool(s): ${unknown.join(", ")}`);
  }
  if (requested.length === 0) {
    throw new Error("replay: at least one tool must be selected");
  }
  return [...new Set(requested)];
}

export function run(command, commandArgs, options = {}) {
  const result = spawnSync(command, commandArgs, {
    cwd: options.cwd,
    encoding: "utf8",
    env: { ...process.env, NO_COLOR: "1", ...options.env },
    maxBuffer: 64 * 1024 * 1024,
    timeout: options.timeout ?? 300_000,
  });
  if (result.error) throw result.error;
  return {
    status: result.status,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

/** Fail closed: an explicitly pinned binary never falls back to another. */
export function requireBinary(label, explicitPath, fallbacks) {
  const candidates = (explicitPath ? [explicitPath] : fallbacks).map((c) => resolve(c));
  const found = candidates.find((candidate) => existsSync(candidate));
  if (!found) {
    throw new Error(`replay: ${label} not found (looked at: ${candidates.join(", ")})`);
  }
  const probe = run(found, ["--version"]);
  if (probe.status !== 0) throw new Error(`replay: ${label} at ${found} failed --version`);
  return {
    path: found,
    version: (probe.stdout || probe.stderr).trim().split("\n")[0],
  };
}

export function workspaceBinCandidates(name) {
  const suffixes = process.platform === "win32" ? ["", ".cmd", ".ps1"] : [""];
  const roots = [rootDir, benchDir, join(rootDir, "tests")];
  return roots.flatMap((base) =>
    suffixes.map((suffix) => join(base, "node_modules", ".bin", `${name}${suffix}`)),
  );
}

/** Clone (or reuse) the upstream corpus and pin it to the recorded commit. */
export function ensureUpstream(upstreamDir, workRoot) {
  const dir = upstreamDir ? resolve(upstreamDir) : join(workRoot, "vue-benchmarks");
  if (!existsSync(join(dir, ".git"))) {
    rmSync(dir, { recursive: true, force: true });
    const clone = run("git", ["clone", "--quiet", UPSTREAM.url, dir]);
    if (clone.status !== 0) throw new Error(`replay: git clone failed\n${clone.stderr}`);
  }
  const hasPinnedCommit = run("git", ["cat-file", "-e", `${UPSTREAM.commit}^{tree}`], {
    cwd: dir,
  });
  if (hasPinnedCommit.status !== 0) {
    const fetch = run("git", ["fetch", "--quiet", "origin", UPSTREAM.commit], {
      cwd: dir,
    });
    if (fetch.status !== 0) {
      throw new Error(`replay: cannot fetch ${UPSTREAM.commit}\n${fetch.stderr}`);
    }
  }
  const checkout = run("git", ["checkout", "--quiet", UPSTREAM.commit], {
    cwd: dir,
  });
  if (checkout.status !== 0) {
    throw new Error(`replay: cannot checkout ${UPSTREAM.commit}\n${checkout.stderr}`);
  }
  const head = run("git", ["rev-parse", "HEAD"], { cwd: dir }).stdout.trim();
  if (head !== UPSTREAM.commit) {
    throw new Error(`replay: upstream is at ${head}, expected ${UPSTREAM.commit}`);
  }
  return dir;
}

export function resolveVuePackageDir() {
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
 * (shared env.d.ts + tsconfig.base with `paths` pinned at a real vue), with
 * only the minimal config file needed by the Golar CLI row.
 */
export function prepareCase(upstreamDir, caseId, workRoot, vuePackageDir) {
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

export function reportFromText(status, combined) {
  const diagnosticCodes = combined.match(/\bTS\d{4}\b/g) ?? [];
  return {
    errorCount: diagnosticCodes.length > 0 ? diagnosticCodes.length : status === 0 ? 0 : 1,
  };
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
      return {
        outcome: "fail",
        detail: `expected >=${minErrors} error(s), got ${errors}`,
      };
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
    return {
      outcome: "fail",
      detail: `expected clean, got ${errors} error(s)`,
    };
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
