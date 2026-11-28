import { appendFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { labels, type Receipt, type RecordRow } from "./shipped-cli-nuxt-protocol.ts";

const median = (values: number[]) => {
  const sorted = [...values].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
};

export function publish(out: string, receipt: Receipt, records: RecordRow[], complete: boolean) {
  const lines = [
    "## Shipped CLI Nuxt consumer",
    "",
    receipt.scope,
    "",
    `Base \`${receipt.producer.baseSha}\` · Head \`${receipt.producer.headSha}\``,
    "",
    `Retained producer run: https://github.com/${receipt.producer.repository}/actions/runs/${receipt.producer.runId}`,
    "",
    `Complete output and all 1,107 program inputs: **${complete ? "equal in all 4 warmups and 32 measured runs" : "incomplete; no qualification credit"}**.`,
    "",
    "| Independent image | Runs | Wall median (s) | Total CPU median (s) | User median (s) | System median (s) | GNU time maximum RSS median (KiB) |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
  ];
  const medians: Record<string, Record<string, number>> = {};
  for (const label of labels) {
    const rows = records.filter((r) => r.label === label);
    if (!rows.length) continue;
    medians[label] = Object.fromEntries(
      ["elapsed", "user", "system", "maxRssKiB"].map((key) => [
        key,
        median(rows.map((r) => r[key as "elapsed"])),
      ]),
    );
    medians[label].cpuTotal = median(rows.map((r) => r.user + r.system));
    const m = medians[label];
    lines.push(
      `| ${label} | ${rows.length} | ${m.elapsed.toFixed(4)} | ${m.cpuTotal.toFixed(4)} | ${m.user.toFixed(4)} | ${m.system.toFixed(4)} | ${m.maxRssKiB.toFixed(0)} |`,
    );
  }
  if (complete) {
    const change = (a: string, b: string) =>
      (100 * (medians[b].elapsed / medians[a].elapsed - 1)).toFixed(3) + "%";
    lines.push(
      "",
      `Wall median deltas: primary head/base ${change("base", "head")}; twin head/base ${change("base-twin", "head-twin")}; base A/A ${change("base", "base-twin")}; head A/A ${change("head", "head-twin")}.`,
    );
  }
  lines.push(
    "",
    "Fixed schedule: four unmeasured warmups, then four repetitions of A B B′ A′ B′ A′ A B (eight observations per independent build). GNU time covers the CLI and its children; complete raw JSON/stdout, stderr, and time records are retained. The same physical invocation path is replaced and checked outside each timed interval.",
    "",
    "The Actions result checks artifact integrity, frozen workload integrity, and complete functional equivalence. Timing and memory are reported without an acceptance threshold; passing this workflow alone does not establish absence of a performance regression.",
    "",
  );
  writeFileSync(join(out, "summary.md"), lines.join("\n"));
  writeFileSync(
    join(out, "result.json"),
    JSON.stringify({ complete, receipt, medians, records }, null, 2) + "\n",
  );
  if (process.env.GITHUB_STEP_SUMMARY)
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, lines.join("\n"));
}
