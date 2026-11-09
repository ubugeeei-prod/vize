// Source-derived whole-repository rule inventory. Counts and tables are emitted
// as CI/release artifacts; authored overrides remain a committed input.
import { aggregateMain } from "./lib/aggregate-artifact.ts";
import { buildMatrix } from "./lib/rule-parity-matrix.ts";
import { renderArtifact } from "./lib/rule-parity-render.ts";

aggregateMain(
  "rule-parity.md",
  () => renderArtifact(buildMatrix()),
  "usage: node tools/support/compat/davinci/rule-parity.ts --write | --check | --summary [--out-dir <dir>]",
);
