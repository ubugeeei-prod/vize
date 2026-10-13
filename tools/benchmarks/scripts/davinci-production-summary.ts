/** Aggregate three independent same-head production comparisons, report only. */
import { appendFileSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { isRecord } from "./script-types.ts";

type ReportShape = {
  shape: string;
  observations: Record<string, unknown>[];
  cohorts: Record<string, unknown>;
};
/** One runner's `samples.json`; fields other than `shapes` are only compared. */
type ProductionReport = Record<string, unknown> & { shapes: ReportShape[] };
type TimedSample = Record<string, unknown> & {
  selected_batch_ns: number[];
  retained_batch_ns: number[];
};

const isReportShape = (value: unknown): value is ReportShape =>
  isRecord(value) &&
  typeof value.shape === "string" &&
  Array.isArray(value.observations) &&
  value.observations.every(isRecord) &&
  isRecord(value.cohorts);
const isProductionReport = (value: unknown): value is ProductionReport =>
  isRecord(value) && Array.isArray(value.shapes) && value.shapes.every(isReportShape);
// Nine positive finite batch samples per side, as the timing protocol records.
const isTimedSample = (sample: Record<string, unknown>): sample is TimedSample =>
  (["selected_batch_ns", "retained_batch_ns"] as const).every((key) => {
    const values = sample[key];
    return (
      Array.isArray(values) &&
      values.length === 9 &&
      values.every(
        (value: unknown) => typeof value === "number" && Number.isFinite(value) && value > 0,
      )
    );
  });

const [root, output] = process.argv.slice(2);
if (!root || !output)
  throw new Error("usage: davinci-production-summary.ts <artifact-root> <output.md>");
const files = readdirSync(root)
  .filter((name) => /^davinci-production-perf-[123]$/.test(name))
  .sort()
  .map((name) => join(root, name, "samples.json"));
if (files.length !== 3) throw new Error(`Expected three independent reports, got ${files.length}`);
const reports = files.map((file) => {
  const report: unknown = JSON.parse(readFileSync(file, "utf8"));
  // A report without well-formed shape entries cannot cover every shipping shape.
  if (!isProductionReport(report)) throw new Error("Expected each shipping shape exactly once");
  return report;
});
const identity = (report: ProductionReport) =>
  JSON.stringify({
    schema: report.schema_version,
    head: report.head_sha,
    manifest: report.manifest_sha256,
    files: report.files,
    profile: report.profile,
    allocator: report.allocator,
    features: report.features,
    options: report.options,
    window: report.window,
    sampling: report.sampling,
  });
if (!/^[0-9a-f]{40}$/.test(String(reports[0].head_sha)))
  throw new Error("Missing exact benchmark head");
const shippingShapes = ["dom_inline", "dom_module", "ssr", "vapor"];
for (const report of reports) {
  if (identity(report) !== identity(reports[0]))
    throw new Error("Reports have different input/build identities");
  if (
    !Array.isArray(report.shapes) ||
    JSON.stringify(report.shapes.map((shape) => shape.shape).sort()) !==
      JSON.stringify(shippingShapes)
  ) {
    throw new Error("Expected each shipping shape exactly once");
  }
}
const median = (values: number[]) =>
  [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
const lines = [
  "# Davinci production comparison",
  "",
  `Exact head: \`${String(reports[0].head_sha)}\`.`,
  "",
  `Input manifest: \`${String(reports[0].manifest_sha256)}\` (${String(reports[0].files)} committed SFCs).`,
  "",
  `Profile: ${String(reports[0].profile)}; allocator: ${String(reports[0].allocator)}.`,
  "",
  "Ratios are selected / retained; below 1 is faster. The center is the median of three independent runner ratios, each derived from nine alternating paired batch samples. Ranges retain all three observations. These report-only results change no budget.",
  "",
  "| Requested shape | Cohort | Files | Median ratio | Runner range | Three runner ratios |",
  "| --- | --- | ---: | ---: | --- | --- |",
];
for (const shape of reports[0].shapes) {
  const peers = reports.map((report) => report.shapes.find((entry) => entry.shape === shape.shape));
  if (peers.some((peer) => !peer)) throw new Error(`Missing shape ${shape.shape}`);
  if (shape.observations.length !== reports[0].files) throw new Error("Missing input observations");
  for (const peer of peers) {
    if (JSON.stringify(peer!.observations) !== JSON.stringify(shape.observations)) {
      throw new Error(`Nondeterministic output/admission observations: ${shape.shape}`);
    }
  }
  for (const [name, cohort] of Object.entries(shape.cohorts)) {
    const expected =
      name === "all"
        ? shape.observations.length
        : shape.observations.filter((observation) => observation.cohort === name).length;
    if (!isRecord(cohort) || typeof cohort.files !== "number" || cohort.files !== expected)
      throw new Error(`Cohort count mismatch: ${shape.shape}/${name}`);
    if (!cohort.files) continue;
    const ratios = peers.map((peer) => {
      const sample = peer!.cohorts[name];
      if (!isRecord(sample) || sample.files !== cohort.files || sample.passes_per_sample !== 5)
        throw new Error("Different timed workloads");
      if (!isTimedSample(sample)) throw new Error("Invalid raw timing samples");
      const selected = median(sample.selected_batch_ns);
      const retained = median(sample.retained_batch_ns);
      if (
        selected !== sample.selected_median_batch_ns ||
        retained !== sample.retained_median_batch_ns
      ) {
        throw new Error("Stored median differs from raw samples");
      }
      return selected / retained;
    });
    lines.push(
      `| ${shape.shape} | ${name} | ${cohort.files} | ${median(ratios).toFixed(4)} | ${Math.min(...ratios).toFixed(4)}–${Math.max(...ratios).toFixed(4)} | ${ratios.map((ratio) => ratio.toFixed(4)).join(", ")} |`,
    );
  }
}
lines.push(
  "",
  "## Backend-selection observations",
  "",
  "Counts include diagnosed inputs if the backend actually recorded acceptance; clean accepted timing cohorts exclude warnings/errors and routed Vapor. Requested DOM shapes keep explicit Vapor routes separate. This table does not replace the P3-17 fixed reach-floor gate.",
  "",
  "Native-only acceptance is not measured by these reports. An accepted selection proves which emitter ran; it does not prove that its facts were produced without legacy code. The DOM emitter can consume Croquis-backed binding and reactivity facts. These counts do not satisfy #6853 native-only acceptance or #6854 zero-fallback deletion criteria.",
  "",
  "| Requested shape | Compiled templates | Emitter selected for requested backend | Routed Vapor accepted | Fallback/rejected/unrecorded templates | Code differences on clean accepted pairs | Diagnostic-message differences |",
  "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
);
for (const shape of reports[0].shapes) {
  const templates = shape.observations.filter((entry) => entry.has_template && entry.compiled);
  const selected = templates.filter(
    (entry) => entry.selected_lane === "accepted" && entry.backend === shape.shape,
  );
  const routed = templates.filter(
    (entry) => entry.selected_lane === "accepted" && entry.backend !== shape.shape,
  );
  const differentCode = shape.observations.filter(
    (entry) => entry.cohort === "accepted" && !entry.code_equal,
  ).length;
  const differentMessages = shape.observations.filter((entry) => !entry.messages_equal).length;
  lines.push(
    `| ${shape.shape} | ${templates.length} | ${selected.length} | ${routed.length} | ${templates.length - selected.length - routed.length} | ${differentCode} | ${differentMessages} |`,
  );
}
lines.push(
  "",
  "The profiler is disabled for all timed samples. Separate enabled attribution exports follow timing. Input I/O is excluded; SFC parsing, script/template/style compilation, assembly and result destruction are included. Maps are default/off. Vapor output equality is an observation; TS-33 runtime and source-map gates remain required. The historical P0-3 numeric baseline and broader hydrated project corpus are outside this measurement.",
  "",
);
const summary = lines.join("\n");
writeFileSync(output, summary);
if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
