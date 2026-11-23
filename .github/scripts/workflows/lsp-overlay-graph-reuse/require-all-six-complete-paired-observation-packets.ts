import { requiredEnv } from "../shared/files.ts";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
const root = requiredEnv("OBSERVATION_ROOT");
interface Scenario {
  mode: string;
  files: number;
  cold_ms: number;
  warm_ms: number[];
  edits: {
    elapsed_ms: number;
    cycle: number;
    host_context_reused: boolean;
    host_source_registrations: number;
    residency: { rss_kib: number; high_water_kib: number };
  }[];
  complete_unchanged_sources: [unknown, { source_path: string; materialized_path: string }][];
  complete_final_sibling: { source_path: string };
  complete_final_native_documents: unknown[];
}
const records: (Scenario & { variant: string; round: number })[] = [];
for (const variant of ["before", "after"]) {
  for (let round = 1; round <= 3; round++) {
    const prefix = path.join(root, `${variant}-${round}`);
    const stdout = fs.readFileSync(prefix + ".stdout", "utf8");
    assert.match(
      stdout,
      /^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out; finished in [0-9.]+s$/m,
    );
    const packet = JSON.parse(fs.readFileSync(prefix + ".json", "utf8")) as {
      schema: string;
      scenarios: Scenario[];
    };
    assert.equal(packet.schema, "vize.lsp.overlay-graph-profile.v1");
    assert.deepEqual(
      packet.scenarios.map((row) => row.mode),
      ["query-edited-sibling-first", "query-unchanged-host-first"],
    );
    for (const scenario of packet.scenarios) {
      assert.equal(scenario.files, 302);
      assert.equal(scenario.warm_ms.length, 5);
      assert.equal(scenario.edits.length, 8);
      assert.ok(scenario.complete_unchanged_sources.length > 300);
      assert.ok(scenario.complete_final_native_documents.length > 300);
      for (const elapsed of [
        scenario.cold_ms,
        ...scenario.warm_ms,
        ...scenario.edits.map((row) => row.elapsed_ms),
      ])
        assert.ok(Number.isFinite(elapsed) && elapsed >= 0);
      for (const [index, row] of scenario.edits.entries()) {
        assert.equal(row.cycle, index + 1);
        assert.equal(typeof row.host_context_reused, "boolean");
        assert.ok(Number.isInteger(row.host_source_registrations));
        assert.ok(
          row.residency.rss_kib > 0 && row.residency.high_water_kib >= row.residency.rss_kib,
        );
      }
      records.push({ variant, round, ...scenario });
    }
  }
}
const normalize = (scenario: Scenario) => {
  const hosts = scenario.complete_unchanged_sources.filter(
    ([, packet]) => path.basename(packet.source_path) === "A.vue",
  );
  assert.equal(hosts.length, 1);
  const authored = path.dirname(hosts[0][1].source_path);
  const mirror = path.dirname(hosts[0][1].materialized_path);
  const visit = (value: unknown): unknown =>
    typeof value === "string"
      ? value.split(mirror).join("<mirror>").split(authored).join("<authored>")
      : Array.isArray(value)
        ? value.map(visit)
        : value && typeof value === "object"
          ? Object.fromEntries(Object.entries(value).map(([key, value]) => [key, visit(value)]))
          : value;
  assert.equal(path.basename(scenario.complete_final_sibling.source_path), "B.vue");
  return visit({
    sources: scenario.complete_unchanged_sources,
    sibling: scenario.complete_final_sibling,
    native: scenario.complete_final_native_documents,
  });
};
for (const mode of ["query-edited-sibling-first", "query-unchanged-host-first"]) {
  const sameInput = records.filter((row) => row.mode === mode);
  const reference = normalize(sameInput[0]);
  for (const observation of sameInput)
    assert.deepEqual(
      normalize(observation),
      reference,
      `complete parent/candidate code, mappings and native batch: ${mode}/${observation.variant}/${observation.round}`,
    );
}
const median = (values: number[]) => {
  const sorted = values.toSorted((a, b) => a - b);
  return sorted[Math.floor(sorted.length / 2)];
};
const summaries = [];
for (const mode of ["query-edited-sibling-first", "query-unchanged-host-first"]) {
  for (const variant of ["before", "after"]) {
    const rows = records.filter((row) => row.mode === mode && row.variant === variant);
    assert.equal(rows.length, 3);
    summaries.push({
      mode,
      variant,
      cold_ms: median(rows.map((row) => row.cold_ms)),
      warm_ms: median(rows.flatMap((row) => row.warm_ms)),
      edit_ms: median(rows.flatMap((row) => row.edits.map((edit) => edit.elapsed_ms))),
      rust_rss_kib: median(rows.map((row) => row.edits.at(-1)!.residency.rss_kib)),
      reused: rows.flatMap((row) => row.edits).filter((row) => row.host_context_reused).length,
      samples: 24,
    });
  }
}
fs.writeFileSync(
  path.join(root, "summary.json"),
  JSON.stringify(
    {
      schema: "vize.lsp.overlay-graph-summary.v1",
      buildProfile: "ci",
      scope:
        "Rust editor preparation with one retained original query; not whole-editor paired RSS",
      summaries,
    },
    null,
    2,
  ) + "\n",
);
fs.appendFileSync(
  requiredEnv("GITHUB_STEP_SUMMARY"),
  "### Paired Rust editor preparation observations\n\nProfile ci, same runner and identical instrumentation. Fresh editor sessions; filesystem caches uncontrolled. RSS covers the Rust test process with one retained original query. Native editor process-tree RSS is observed separately at the candidate only.\n\n| Mode | Source | Cold ms | Warm ms | Edit ms | Rust RSS KiB | Graph reuse |\n|---|---|---:|---:|---:|---:|---:|\n" +
    summaries
      .map(
        (row) =>
          `| ${row.mode} | ${row.variant} | ${row.cold_ms.toFixed(2)} | ${row.warm_ms.toFixed(2)} | ${row.edit_ms.toFixed(2)} | ${row.rust_rss_kib} | ${row.reused}/${row.samples} |`,
      )
      .join("\n") +
    "\n",
);
