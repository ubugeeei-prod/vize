// Source-derived whole-repository rule inventory. Counts and tables are emitted
// as CI/release artifacts; authored overrides remain a committed input.
import { aggregateMain } from "./lib/aggregate-artifact.mjs";
import { buildMatrix } from "./lib/rule-parity-matrix.mjs";
import { renderArtifact } from "./lib/rule-parity-render.mjs";

aggregateMain(
  "rule-parity.md",
  () => renderArtifact(buildMatrix()),
  "usage: rust-script tools/commands/davinci/rule-parity.rs --write | --check | --summary [--out-dir <dir>]",
);
