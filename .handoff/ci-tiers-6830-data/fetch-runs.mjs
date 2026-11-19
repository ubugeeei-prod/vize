// node fetch-runs.mjs out.tsv since [event]
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
const [out, since, event = "pull_request"] = process.argv.slice(2);
const lines = [];
for (let page = 1; page <= 30; page++) {
  let raw;
  for (let i = 0; ; i++) {
    try {
      raw = execFileSync("gh", ["api", `repos/ubugeeei-prod/vize/actions/workflows/check.yml/runs?event=${event}&per_page=100&page=${page}&created=${since}&status=success`,
        "--jq", ".workflow_runs[] | [.id, .created_at, .updated_at, .run_started_at, .run_attempt, .conclusion, .head_branch, .head_sha] | @tsv"], { encoding: "utf8", maxBuffer: 1 << 28 });
      break;
    } catch (e) { if (i > 3) throw e; execFileSync("sleep", ["3"]); }
  }
  const rows = raw.trim().split("\n").filter(Boolean);
  lines.push(...rows);
  process.stderr.write(`page ${page} ${rows.length}\n`);
  if (rows.length < 100) break;
}
writeFileSync(out, lines.join("\n") + "\n");
