// node fetch-jobs.mjs runs.tsv out.json [limit]
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
const [runsFile, out, limit = "40"] = process.argv.slice(2);
const rows = readFileSync(runsFile, "utf8").trim().split("\n").slice(0, Number(limit));
const result = [];
for (const row of rows) {
  const [id, created, updated, started, attempt, branch, sha] = row.split("\t");
  let raw;
  for (let i = 0; ; i++) {
    try {
      raw = execFileSync(
        "gh",
        ["api", "--paginate", `repos/ubugeeei-prod/vize/actions/runs/${id}/attempts/${attempt}/jobs?per_page=100`, "--jq", ".jobs[]"],
        { encoding: "utf8", maxBuffer: 1 << 28 },
      );
      break;
    } catch (e) { if (i > 2) { raw = null; break; } execFileSync("sleep", ["3"]); }
  }
  if (raw === null) { process.stderr.write(`skip ${id}\n`); continue; }
  const jobs = raw.trim().split("\n").filter(Boolean).map((l) => JSON.parse(l)).map((j) => ({
    name: j.name, conclusion: j.conclusion, created: j.created_at, started: j.started_at, completed: j.completed_at,
    runner: j.runner_name, labels: j.labels,
    steps: (j.steps || []).map((s) => ({ name: s.name, conclusion: s.conclusion, started: s.started_at, completed: s.completed_at })),
  }));
  result.push({ id, created, updated, started, attempt, branch, sha, jobs });
  process.stderr.write(`${id} ${jobs.length}\n`);
}
writeFileSync(out, JSON.stringify(result));
