// node pr-side-runs.mjs since out.json : pull_request runs per workflow (all workflows)
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
const [since, out] = process.argv.slice(2);
const rows = [];
for (let page = 1; page <= 10; page++) {
  const raw = execFileSync("gh", ["api", `repos/ubugeeei-prod/vize/actions/runs?event=pull_request&per_page=100&page=${page}&created=>=${since}`,
    "--jq", ".workflow_runs[] | [.id, .path, .created_at, .updated_at, .conclusion // \"\", .head_branch] | @tsv"], { encoding: "utf8", maxBuffer: 1 << 28 });
  const lines = raw.trim().split("\n").filter(Boolean);
  rows.push(...lines.map((l) => l.split("\t")));
  if (lines.length < 100) break;
}
const by = new Map();
for (const [id, path, c, u, concl, branch] of rows) {
  const k = path.replace(".github/workflows/", "");
  const e = by.get(k) ?? { runs: 0, minutes: 0, branches: new Set() };
  e.runs++; e.minutes += (Date.parse(u) - Date.parse(c)) / 60000; e.branches.add(branch);
  by.set(k, e);
}
const list = [...by].map(([k, e]) => ({ workflow: k, runs: e.runs, branches: e.branches.size, wallMin: Math.round(e.minutes) }))
  .sort((a, b) => b.runs - a.runs);
writeFileSync(out, JSON.stringify({ total: rows.length, list }, null, 1));
console.log(rows.length, "runs");
for (const r of list.slice(0, 40)) console.log(r.runs, r.branches, r.wallMin, r.workflow);
