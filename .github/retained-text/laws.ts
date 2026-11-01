import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export type Json = Record<string, unknown>;
export type Role = "B" | "C";
export const order: readonly Role[] = ["B", "C", "C", "B"];
export const base = "ec8474e0ea9ed05062542c57c857f36b9467ac15";
export const candidate = "437f32b11bb5d8ab9275583a4d0dbdb1f3e8b048";
export const fixtureRevision = "748b182e440863711dc9624924e92440f5ddc4d3";
export const recipe = "cargo build --profile ci -p vize --features legacy";
export const lanes = [
  "initialize",
  "coldOpen",
  "coldOpenSecondApp",
  "completion",
  "hover",
  "warmNoop",
  "leafBroken",
  "leafRepaired",
  "sharedBroken",
  "sharedBrokenSecondApp",
  "sharedRepaired",
  "sharedRepairedSecondApp",
  "warmNoopSecondApp",
  "cancellationConverge",
] as const;
export const laneCeilings = [
  1000, 8000, 1500, 1000, 1000, 1500, 1500, 1500, 3000, 3000, 3000, 3000, 1500, 6000,
];
export const unitNames = [
  "equal_unicode_crlf_text_keeps_the_original_owned_key_and_value",
  "changed_values_keep_the_original_allocation_alive_through_the_callers_scope",
  "duplicate_batch_entries_keep_sequential_last_writer_and_dirty_state",
  "the_single_writer_keeps_equal_storage_without_skipping_diagnostic_clearing",
  "equal_logical_batches_still_retain_the_complete_revision_and_uri_mappings",
  "equal_retained_materialized_bytes_still_repair_a_stale_disk_member",
  "independent_clients_and_removed_membership_do_not_share_retained_storage",
].map((name) => `lsp_client::retained_text_tests::${name}`);

export function object(value: unknown): Json {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  return value as Json;
}
export function number(value: unknown): number {
  assert.equal(typeof value, "number");
  assert.ok(Number.isFinite(value));
  return value as number;
}
export function positive(value: unknown): number {
  const n = number(value);
  assert.ok(n > 0);
  return n;
}
export function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
export function requiredTests(output: string): void {
  assert.match(output, /^running [1-9][0-9]* tests?$/m);
  assert.match(output, /^test result: ok\. [1-9][0-9]* passed; 0 failed;/m);
}
export function exactUnits(output: string): void {
  const found = [...output.matchAll(/^test (\S+) \.\.\. ok$/gm)].map((row) => row[1]);
  assert.deepEqual(found.sort(), [...unitNames].sort());
  assert.deepEqual(
    [...output.matchAll(/^running (\d+) tests$/gm)].map((row) => row[1]),
    ["7"],
  );
  assert.match(
    output,
    /^test result: ok\. 7 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;/m,
  );
}
export function cargoMessages(text: string): Json[] {
  return text.split("\n").flatMap((line) => {
    try {
      return [object(JSON.parse(line))];
    } catch {
      return [];
    }
  });
}
export function cliArtifact(text: string): Json {
  const messages = cargoMessages(text);
  assert.ok(messages.some((row) => row.reason === "build-finished" && row.success === true));
  const bins = messages.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      object(row.target).name === "vize" &&
      array(object(row.target).kind).includes("bin"),
  );
  assert.equal(bins.length, 1);
  const artifact = bins[0];
  assert.deepEqual([...array(artifact.features)].sort(), ["default", "glyph", "legacy", "maestro"]);
  const profile = object(artifact.profile);
  assert.equal(profile.test, false);
  assert.equal(profile.opt_level, "0");
  assert.ok(profile.debuginfo === 0 || profile.debuginfo === "none");
  assert.equal(profile.debug_assertions, true);
  assert.equal(typeof artifact.executable, "string");
  return artifact;
}

/** Additional evidence checks never replace the original test's settlement. */
export function inspectMetrics(value: unknown, source: string): Json {
  const data = object(value);
  assert.equal(data.schemaVersion, 3);
  assert.equal(data.commit, source);
  assert.equal(data.fixtureRevision, fixtureRevision);
  assert.ok(data.status === "passed" || data.status === "failed");
  const budget = object(data.budget);
  assert.equal(budget.scale, 1);
  assert.equal(budget.suite, "vben-lsp-incremental");
  assert.equal(budget.laneHardTimeoutMs, 60000);
  assert.equal(budget.maxPeakRssMiB, 256);
  assert.equal(budget.maxPeakProcessTreeRssMiB, 512);
  assert.equal(budget.maxProcessTreeSize, 4);
  assert.deepEqual(
    budget.laneBudgetsMs,
    Object.fromEntries(lanes.map((name, i) => [name, laneCeilings[i]])),
  );
  const timings = object(data.timingsMs);
  assert.deepEqual(Object.keys(timings), [...lanes]);
  for (const lane of lanes) positive(timings[lane]);
  const corpus = object(data.corpus);
  assert.ok(number(corpus.vueFiles) >= 500);
  assert.ok(number(corpus.vueAndTypeScriptFiles) >= number(corpus.vueFiles));
  const snapshots = object(data.processTreeSamples);
  assert.ok(Object.keys(snapshots).length > 0);
  for (const raw of Object.values(snapshots)) {
    const sample = object(raw);
    const members = array(sample.members).map(object);
    assert.equal(sample.processes, members.length);
    assert.equal(
      sample.totalKiB,
      members.reduce((sum, row) => sum + number(row.rssKiB), 0),
    );
    assert.equal(members.filter((row) => row.role === "root").length, 1);
    for (const member of members) {
      positive(member.pid);
      positive(member.parentPid);
      const identity = object(member.identity);
      assert.match(String(identity.birthTicks), /^\d+$/);
      assert.match(String(identity.executableSha256), /^[a-f0-9]{64}$/);
    }
  }
  return data;
}
export function candidatePass(result: Json, metrics: Json): void {
  assert.equal(result.status, 0);
  assert.equal(result.signal, null);
  assert.equal(result.error, null);
  assert.equal(metrics.status, "passed", String(metrics.failure));
  assert.equal(metrics.failure, null);
  assert.ok(positive(metrics.sampledPeakRssKiB) <= 256 * 1024);
  assert.ok(positive(metrics.sampledPeakProcessTreeRssKiB) <= 512 * 1024);
  assert.ok(positive(metrics.sampledPeakProcessTreeSize) <= 4);
  for (const [index, lane] of lanes.entries())
    assert.ok(number(object(metrics.timingsMs)[lane]) <= laneCeilings[index]);
}

export function verifyGuardRoster(value: unknown, original: string): Json[] {
  assert.equal(
    createHash("sha256").update(original).digest("hex"),
    "77cc3931fc18a4bd519e4780fda07aec5ae3cbbd38fc5bed39054572a7c9f9d4",
  );
  const body = original.slice(
    original.indexOf("      - name: Protect cold local responses"),
    original.indexOf("      - name: Retain source, binary"),
  );
  const expected = body
    .split("\n")
    .map((line) => line.trim())
    .filter((line) =>
      /^(?:node --test |cargo (?:test|clippy) |run_required_rust_tests [a-z])/.test(line),
    )
    .map((line) => line.replace(/^run_required_rust_tests \S+ /, "").split(" > ")[0]);
  assert.equal(expected.length, 16);
  const rows = array(object(value).commands).map(object);
  assert.deepEqual(
    rows.map((row) => array(row.argv).map(String).join(" ")),
    expected,
  );
  assert.deepEqual(
    rows.map((row) => row.label),
    [
      "controls",
      "initial-diagnostics",
      "native-requests",
      "project-contexts",
      "vize_maestro-clippy",
      "symlink-workspace",
      "alias-transition",
      "alias-cache",
      "vize_canon-clippy",
      "native-view",
      "issued-candidates",
      "initial-api-read",
      "initial-api-dependencies",
      "completion",
      "rename",
      "symlink",
    ],
  );
  for (const row of rows) {
    const isRust = array(row.argv)[0] === "cargo" && array(row.argv)[1] === "test";
    assert.equal(row.requireRust, isRust ? true : undefined);
    const retainer = ["completion", "rename", "symlink"].includes(String(row.label))
      ? `--retain-${row.label}-producer`
      : undefined;
    assert.equal(row.retain, retainer);
    assert.deepEqual(
      Object.keys(row).sort(),
      ["label", "argv", ...(isRust ? ["requireRust"] : []), ...(retainer ? ["retain"] : [])].sort(),
    );
  }
  return rows;
}
export function postPassed(rows: Json[], labels: string[]): boolean {
  assert.deepEqual(
    rows.map((row) => row.label),
    labels,
  );
  return rows.every((row) => row.passed === true);
}

export function report(outcomes: Json[], admitted: boolean): string {
  const rows = [
    "# Original instrumented legacy-ci Vben B/C/C/B",
    "",
    `Campaign admission: ${admitted}.`,
    "Base failures remain observed failures. Two candidate passes are mandatory; this is not general/shipping or allocation-volume evidence.",
    "",
    "| Attempt | Original status | Root KiB | Tree KiB | Processes |",
    "| --- | --- | ---: | ---: | ---: |",
  ];
  for (const row of outcomes) {
    const m = row.metrics ? object(row.metrics) : {};
    rows.push(
      `| ${row.label} | ${m.status ?? "incomplete"} | ${m.sampledPeakRssKiB ?? "?"} | ${m.sampledPeakProcessTreeRssKiB ?? "?"} | ${m.sampledPeakProcessTreeSize ?? "?"} |`,
    );
  }
  rows.push(
    "",
    "| Lane | 1-B ms | 2-C ms | 3-C ms | 4-B ms |",
    "| --- | ---: | ---: | ---: | ---: |",
  );
  for (const lane of lanes) {
    rows.push(
      `| ${lane} | ${outcomes.map((row) => (row.metrics ? (object(object(row.metrics).timingsMs)[lane] ?? "?") : "?")).join(" | ")} |`,
    );
  }
  rows.push(
    "",
    "Raw metrics retain every process identity/member sum and attribution setup/capture cost. No favorable selection or retry. A failed original settlement does not establish later guard execution.",
  );
  return rows.join("\n") + "\n";
}
