/** Aggregate three independent same-head production comparisons, report only. */
import { appendFileSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const [root, output] = process.argv.slice(2);
if (!root || !output) throw new Error("usage: davinci-production-summary.mjs <artifact-root> <output.md>");
const files = readdirSync(root)
  .filter((name) => /^davinci-production-perf-[123]$/.test(name))
  .sort()
  .map((name) => join(root, name, "samples.json"));
if (files.length !== 3) throw new Error(`Expected three independent reports, got ${files.length}`);
const reports = files.map((file) => JSON.parse(readFileSync(file, "utf8")));
const identity = (report) => JSON.stringify({
  schema: report.schema_version, head: report.head_sha,
  manifest: report.manifest_sha256, files: report.files,
  profile: report.profile, allocator: report.allocator,
  features: report.features, options: report.options,
  window: report.window, sampling: report.sampling,
});
if (!/^[0-9a-f]{40}$/.test(reports[0].head_sha)) throw new Error("Missing exact benchmark head");
for (const report of reports) {
  if (identity(report) !== identity(reports[0])) throw new Error("Reports have different input/build identities");
  if (report.shapes.length !== 4) throw new Error("Expected all four shipping shapes");
}
const median = (values) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)];
const lines = [
  "# Davinci production comparison", "",
  `Exact head: \`${reports[0].head_sha}\`.`, "",
  `Input manifest: \`${reports[0].manifest_sha256}\` (${reports[0].files} committed SFCs).`, "",
  `Profile: ${reports[0].profile}; allocator: ${reports[0].allocator}.`, "",
  "Ratios are selected / retained; below 1 is faster. The center is the median of three independent runner ratios, each derived from nine alternating paired batch samples. Ranges retain all three observations. These report-only results change no budget.", "",
  "| Requested shape | Cohort | Files | Median ratio | Runner range | Three runner ratios |",
  "| --- | --- | ---: | ---: | --- | --- |",
];
for (const shape of reports[0].shapes) {
  const peers = reports.map((report) => report.shapes.find((entry) => entry.shape === shape.shape));
  if (peers.some((peer) => !peer)) throw new Error(`Missing shape ${shape.shape}`);
  if (shape.observations.length !== reports[0].files) throw new Error("Missing input observations");
  for (const peer of peers) {
    if (JSON.stringify(peer.observations) !== JSON.stringify(shape.observations)) {
      throw new Error(`Nondeterministic output/admission observations: ${shape.shape}`);
    }
  }
  for (const [name, cohort] of Object.entries(shape.cohorts)) {
    const expected = name === "all" ? shape.observations.length
      : shape.observations.filter((observation) => observation.cohort === name).length;
    if (cohort.files !== expected) throw new Error(`Cohort count mismatch: ${shape.shape}/${name}`);
    if (!cohort.files) continue;
    const ratios = peers.map((peer) => {
      const sample = peer.cohorts[name];
      if (sample.files !== cohort.files || sample.passes_per_sample !== 5) throw new Error("Different timed workloads");
      for (const key of ["selected_batch_ns", "retained_batch_ns"]) {
        if (sample[key].length !== 9 || sample[key].some((value) => !Number.isFinite(value) || value <= 0)) {
          throw new Error("Invalid raw timing samples");
        }
      }
      const selected = median(sample.selected_batch_ns);
      const retained = median(sample.retained_batch_ns);
      if (selected !== sample.selected_median_batch_ns || retained !== sample.retained_median_batch_ns) {
        throw new Error("Stored median differs from raw samples");
      }
      return selected / retained;
    });
    lines.push(`| ${shape.shape} | ${name} | ${cohort.files} | ${median(ratios).toFixed(4)} | ${Math.min(...ratios).toFixed(4)}–${Math.max(...ratios).toFixed(4)} | ${ratios.map((ratio) => ratio.toFixed(4)).join(", ")} |`);
  }
}
lines.push("", "## Production adoption observations", "",
  "Counts include diagnosed inputs if the backend actually recorded acceptance; clean accepted timing cohorts exclude warnings/errors and routed Vapor. Requested DOM shapes keep explicit Vapor routes separate. This table does not replace the P3-17 fixed reach-floor gate.", "",
  "| Requested shape | Compiled templates | Native for requested backend | Routed Vapor accepted | Fallback/rejected/unrecorded templates | Code differences on clean accepted pairs | Diagnostic-message differences |",
  "| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
for (const shape of reports[0].shapes) {
  const templates = shape.observations.filter((entry) => entry.has_template && entry.compiled);
  const native = templates.filter((entry) => entry.selected_lane === "accepted" && entry.backend === shape.shape);
  const routed = templates.filter((entry) => entry.selected_lane === "accepted" && entry.backend !== shape.shape);
  const differentCode = shape.observations.filter((entry) => entry.cohort === "accepted" && !entry.code_equal).length;
  const differentMessages = shape.observations.filter((entry) => !entry.messages_equal).length;
  lines.push(`| ${shape.shape} | ${templates.length} | ${native.length} | ${routed.length} | ${templates.length - native.length - routed.length} | ${differentCode} | ${differentMessages} |`);
}
lines.push("", "The profiler is disabled for all timed samples. Separate enabled attribution exports follow timing. Input I/O is excluded; SFC parsing, script/template/style compilation, assembly and result destruction are included. Maps are default/off. Vapor output equality is an observation; TS-33 runtime and source-map gates remain required. The historical P0-3 numeric baseline and broader hydrated project corpus are outside this measurement.", "");
const summary = lines.join("\n");
writeFileSync(output, summary);
if (process.env.GITHUB_STEP_SUMMARY) appendFileSync(process.env.GITHUB_STEP_SUMMARY, summary);
