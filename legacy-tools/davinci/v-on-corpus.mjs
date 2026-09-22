// Natural modified-v-on corpus inventory (evidence for the two-entry inline
// v-on buckets; tests/tooling/davinci-v-on-storage.test.ts).
//
// Writes one TSV per source area under davinci-road/plan/v-on-corpus/ with
// every natural `@event.modifier` / `v-on:event.modifier` spelling in the
// Git-tracked corpus, one per line. Adding a v-on fixture regenerates only its
// own area's shard, so parallel PRs stop conflicting on a single hand-edited
// count, while the gate still byte-compares the live scan to the committed
// inventory (an intentional update, reviewed per spelling).
//
// Usage:
//   rust-script tools/commands/davinci/v-on-corpus.rs --write   # regenerate
//   rust-script tools/commands/davinci/v-on-corpus.rs --check   # diff committed

import { checkArtifactSet, writeArtifactSet } from "./lib/artifact-set.mjs";
import {
  V_ON_CORPUS_DIR,
  V_ON_CORPUS_REGEN,
  renderVOnCorpus,
  trackedNaturalSources,
} from "./lib/v-on-corpus.mjs";

function main() {
  const mode = process.argv[2];
  if (mode !== "--write" && mode !== "--check") {
    console.error("usage: rust-script tools/commands/davinci/v-on-corpus.rs --write | --check");
    process.exit(2);
  }
  const set = {
    label: "natural v-on corpus inventory",
    files: renderVOnCorpus(trackedNaturalSources()),
    ownedDirs: [V_ON_CORPUS_DIR],
    regenCommand: V_ON_CORPUS_REGEN,
  };
  if (mode === "--write") writeArtifactSet(set);
  else checkArtifactSet(set);
}

main();
