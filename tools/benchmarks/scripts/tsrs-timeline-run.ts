/**
 * Bench-branch only: run the patched source-built tsrs (TSRS_CM_PROFILE timeline) and the
 * released tsrs 0.23 on the profile's prepared head projects, alternating, cold caches.
 *   node tsrs-timeline-run.ts <work> <out> <source-bin> <release-bin> [runs]
 */
import { spawnSync } from "node:child_process";
import { mkdirSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [work, out, sourceBin, releaseBin, runsText] = process.argv.slice(2) as [string, string, string, string, string?];
const runs = Number(runsText ?? 5);
mkdirSync(out, { recursive: true });
const summary: unknown[] = [];
for (const dir of readdirSync(work).filter((name) => name.endsWith("-head")).sort()) {
  const project = join(work, dir);
  const corpus = dir.replace(/-head$/u, "");
  for (let i = 0; i < runs; i++) {
    for (const [lane, bin] of (i % 2 === 0 ? [["source", sourceBin], ["release", releaseBin]] : [["release", releaseBin], ["source", sourceBin]]) as Array<[string, string]>) {
      const cache = join(work, "timeline-cache");
      rmSync(cache, { recursive: true, force: true });
      mkdirSync(cache, { recursive: true });
      const csv = join(out, `${corpus}-${lane}-${i}.csv`);
      const env = { ...process.env, XDG_CACHE_HOME: cache, TMPDIR: cache, ...(lane === "source" ? { TSRS_CM_PROFILE: csv } : {}) };
      const start = performance.now();
      const result = spawnSync(bin, ["--noEmit", "--fast", "--runExternalCode", "--extendedDiagnostics", "--pretty", "false", "-p", join(project, "tsconfig.vize-mapper.json")], { cwd: project, env, encoding: "utf8", maxBuffer: 1 << 30 });
      const wallMs = performance.now() - start;
      const stdout = result.stdout ?? "";
      const stats = Object.fromEntries(stdout.split("\n").map((line) => line.match(/^([A-Za-z][^:]*):\s+([0-9.]+)(s|K)?$/u)).filter(Boolean).map((m) => [m![1]!.trim(), Number(m![2])]));
      const diagnostics = stdout.split("\n").filter((line) => / error TS\d+:/u.test(line)).length;
      summary.push({ corpus, lane, run: i, wallMs, status: result.status, diagnostics, stats });
      console.log(`${corpus} ${lane} #${i}: ${wallMs.toFixed(0)} ms, ${diagnostics} diagnostics, wait ${stats["Content mapper request wait time"]}, check ${stats["Check time"]}`);
    }
  }
}
writeFileSync(join(out, "timeline-summary.json"), `${JSON.stringify(summary, null, 1)}\n`);
