// Compile the actual observer against explicitly selected matching OXC rlibs.
// No Cargo build or dependency cache mutation is performed by this replay.
// Usage: vp node ... PARSER_RLIB ALLOCATOR_RLIB SPAN_RLIB DEPS_DIR OUT [BASELINE]
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

const [parser, allocator, span, dependencies, output, baseline] = process.argv.slice(2);
if (!parser || !allocator || !span || !dependencies || !output) {
  throw new Error("Supply parser/allocator/span rlibs, dependency directory and capture path");
}
const observer = path.resolve("tools/support/dependencies/oxc-pure-observe.rs");
const executable = `${output}.observer`;
const build = spawnSync(
  "rustc",
  [
    "--edition=2024",
    observer,
    "-L",
    `dependency=${dependencies}`,
    "--extern",
    `oxc_parser=${parser}`,
    "--extern",
    `oxc_allocator=${allocator}`,
    "--extern",
    `oxc_span=${span}`,
    "-o",
    executable,
  ],
  { encoding: "utf8", maxBuffer: 1024 * 1024 },
);
if (build.error) throw build.error;
if (build.status !== 0) throw new Error(build.stdout + build.stderr);
const replay = spawnSync(executable, [], { encoding: "utf8", maxBuffer: 128 * 1024 * 1024 });
if (replay.error) throw replay.error;
writeFileSync(output, replay.stdout);
writeFileSync(`${output}.log`, replay.stderr);
if (replay.status !== 0) throw new Error(`Parser replay failed: ${replay.stderr}`);
const after = replay.stdout.trimEnd().split("\n");
if (after.length % 3 !== 0) throw new Error("Incomplete source/profile capture");
let changed = 0;
if (baseline) {
  const before = readFileSync(baseline, "utf8").trimEnd().split("\n");
  if (before.length !== after.length) throw new Error("Baseline capture size differs");
  const classify = (value: string) =>
    value.replace(
      /content: (?:None|PureNotApplied|Pure|NoSideEffects)\b/g,
      "content: CLASSIFICATION",
    );
  for (let offset = 0; offset < before.length; offset += 3) {
    if (before[offset] !== after[offset]) throw new Error(`Input/profile differs at ${offset}`);
    if (before[offset + 2] !== after[offset + 2])
      throw new Error(`Diagnostics/status differ: ${after[offset]}`);
    if (before[offset + 1] === after[offset + 1]) continue;
    if (classify(before[offset + 1]) !== classify(after[offset + 1])) {
      throw new Error(`AST structure/flags/spans differ: ${after[offset]}`);
    }
    changed += 1;
  }
}
const digest = (bytes: string | Buffer) => createHash("sha256").update(bytes).digest("hex");
const receipt = {
  combinations: after.length / 3,
  classificationChanges: baseline ? changed : null,
  observerSha256: digest(readFileSync(observer)),
  captureSha256: digest(replay.stdout),
  baselineSha256: baseline ? digest(readFileSync(baseline)) : null,
  parser,
  allocator,
  span,
};
writeFileSync(`${output}.receipt.json`, `${JSON.stringify(receipt, null, 2)}\n`);
console.log(JSON.stringify(receipt, null, 2));
