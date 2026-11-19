// node failures.mjs out.json : which jobs failed in recent failed PR Check runs
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
const out = process.argv[2];
const gh = (args) => execFileSync("gh", args, { encoding: "utf8", maxBuffer: 1 << 28 });
const runs = gh(["api", "repos/ubugeeei-prod/vize/actions/workflows/check.yml/runs?event=pull_request&status=failure&per_page=80", "--jq", ".workflow_runs[] | [.id, .run_attempt, .created_at, .head_branch] | @tsv"]).trim().split("\n");
const rows = [];
for (const line of runs) {
  const [id, attempt, created, branch] = line.split("\t");
  try {
    const failed = gh(["api", "--paginate", `repos/ubugeeei-prod/vize/actions/runs/${id}/attempts/${attempt}/jobs?per_page=100`, "--jq", '.jobs[] | select(.conclusion=="failure") | .name']).trim().split("\n").filter(Boolean);
    rows.push({ id, created, branch, failed });
  } catch {}
}
writeFileSync(out, JSON.stringify(rows, null, 1));
