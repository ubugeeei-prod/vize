// node plan-logs.mjs jobs.json out.tsv : extract planner summaries per run
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
const [file, out] = process.argv.slice(2);
const runs = JSON.parse(readFileSync(file, "utf8"));
const rows = [];
for (const run of runs) {
  let ids;
  try {
    ids = execFileSync("gh", ["api", "--paginate", `repos/ubugeeei-prod/vize/actions/runs/${run.id}/attempts/${run.attempt}/jobs?per_page=100`, "--jq", '.jobs[] | select(.name|test("Plan PR source")) | .id'], { encoding: "utf8" }).trim().split("\n");
  } catch { continue; }
  let log = "";
  try {
    log = execFileSync("gh", ["api", "--allow-escape-sequences", `repos/ubugeeei-prod/vize/actions/jobs/${ids[0]}/logs`], { encoding: "utf8", maxBuffer: 1 << 28 });
  } catch { continue; }
  const pick = (re) => (log.match(re)?.[0] ?? "").replace(/\s+/g, " ");
  rows.push([run.id, pick(/\d+ changed paths; Rust:[^\n]*/), pick(/Tooling tests: [^\n]*/)].join("\t"));
  process.stderr.write(".");
}
writeFileSync(out, rows.join("\n") + "\n");
