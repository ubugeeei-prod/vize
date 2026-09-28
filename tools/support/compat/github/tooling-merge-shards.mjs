export const mergeToolingShardCount = 4;

// The artifact uploader runs in shard 1, alongside the only test that writes
// its formatter API corpus directory.
export const formatterEvidenceTest = "tests/tooling/differential-formatter-api-execution.test.mjs";

// Rounded file durations of at least four seconds from the complete Actions
// diagnostic run 36445933394. Unknown and quicker files retain a one-second
// fallback, so new tests enter the complete merge suite automatically.
const measuredSeconds = new Map([
  ["tests/tooling/canon-css-module-contracts.test.ts", 5],
  ["tests/tooling/canon-external-contracts.test.ts", 9],
  ["tests/tooling/canon-functional-slot-contracts.test.ts", 42],
  ["tests/tooling/canon-generic-fallthrough-roots.test.ts", 7],
  ["tests/tooling/canon-jsx-module-contracts.test.ts", 18],
  ["tests/tooling/canon-model-macro-contracts.test.ts", 23],
  ["tests/tooling/canon-options-binding-contracts.test.ts", 5],
  ["tests/tooling/canon-own-attrs-contracts.test.ts", 13],
  ["tests/tooling/canon-own-slot-contracts.test.ts", 14],
  ["tests/tooling/canon-script-syntax-ownership.test.ts", 25],
  ["tests/tooling/canon-self-component-contracts.test.ts", 23],
  ["tests/tooling/canon-template-contracts.test.ts", 6],
  ["tests/tooling/canon-template-directives.test.ts", 7],
  ["tests/tooling/canon-template-public-contracts.test.ts", 7],
  ["tests/tooling/canon-template-ref-instances.test.ts", 7],
  ["tests/tooling/canon-upstream-diagnostics.test.ts", 41],
  ["tests/tooling/canon-vfor-scope-contracts.test.ts", 4],
  ["tests/tooling/cli-check-diagnostics.test.ts", 4],
  ["tests/tooling/davinci-button-activation.test.ts", 13],
  ["tests/tooling/davinci-complexity-corpus.test.ts", 15],
  ["tests/tooling/davinci-consumer-migration-surfaces.test.mjs", 6],
  ["tests/tooling/davinci-fact-spec-corpus.test.ts", 23],
  ["tests/tooling/davinci-generated-ledgers.test.ts", 16],
  ["tests/tooling/davinci-matrices.test.ts", 9],
  ["tests/tooling/davinci-metamorphic-corpus.test.ts", 18],
  ["tests/tooling/davinci-plugin-sdk-package.test.ts", 5],
  ["tests/tooling/differential-formatter-api-execution.test.mjs", 14],
  ["tests/tooling/lint-divergence-pug-surface.test.ts", 5],
  ["tests/tooling/lint-divergence-report-runner.test.ts", 5],
  ["tests/tooling/lsp-call-hierarchy-project.test.ts", 7],
  ["tests/tooling/lsp-call-hierarchy-vue-exports.test.ts", 5],
  ["tests/tooling/lsp-component-documentation.test.ts", 7],
  ["tests/tooling/lsp-native-code-actions.test.ts", 6],
  ["tests/tooling/lsp-native-event-authoring.test.ts", 8],
  ["tests/tooling/lsp-own-attrs-authoring.test.ts", 4],
  ["tests/tooling/lsp-own-slot-authoring.test.ts", 6],
  ["tests/tooling/lsp-rich-authoring.test.ts", 6],
  ["tests/tooling/lsp-smoke.test.ts", 6],
  ["tests/tooling/lsp-watched-refresh.test.ts", 7],
  ["tests/tooling/moonbit-publish-crates.test.ts", 4],
  ["tests/tooling/moonbit-warnings.test.ts", 7],
  ["tests/tooling/pattern-vapor-parity-reference.test.ts", 6],
  ["tests/tooling/real-project-lsp.test.ts", 28],
  ["tests/tooling/release/release-pr.test.ts", 5],
  ["tests/tooling/release/release-preflight-runner.test.ts", 4],
  ["tests/tooling/release/release-smoke-runtime-oracle.test.ts", 7],
  ["tests/tooling/rust-cache-backend-policy.test.mjs", 6],
  ["tests/tooling/rust-cache-large-targets.test.mjs", 5],
  ["tests/tooling/rust-cache-nested-post.test.mjs", 5],
  ["tests/tooling/vite-plus-fast-path.test.ts", 19],
]);

function estimatedSeconds(file) {
  return measuredSeconds.get(file) ?? 1;
}

export function toolingShardIds(tier) {
  if (tier === "pr") return [1];
  if (tier === "merge")
    return Array.from({ length: mergeToolingShardCount }, (_, index) => index + 1);
  throw new Error("tier must be pr or merge");
}

export function partitionMergeToolingTests(files) {
  if (!Array.isArray(files) || files.length === 0 || new Set(files).size !== files.length) {
    throw new Error("merge tooling inventory must be nonempty and unique");
  }
  if (!files.includes(formatterEvidenceTest)) {
    throw new Error("merge tooling inventory lost formatter API evidence test");
  }
  const shards = Array.from({ length: mergeToolingShardCount }, () => []);
  const loads = Array.from({ length: mergeToolingShardCount }, () => 0);
  shards[0].push(formatterEvidenceTest);
  loads[0] = estimatedSeconds(formatterEvidenceTest);
  const remaining = files
    .filter((file) => file !== formatterEvidenceTest)
    .sort((a, b) => estimatedSeconds(b) - estimatedSeconds(a) || (a < b ? -1 : 1));
  for (const file of remaining) {
    const lightest = loads.indexOf(Math.min(...loads));
    shards[lightest].push(file);
    loads[lightest] += estimatedSeconds(file);
  }
  if (
    shards.some((shard) => shard.length === 0) ||
    shards.flat().length !== files.length ||
    new Set(shards.flat()).size !== files.length
  ) {
    throw new Error("merge tooling shards do not cover every test exactly once");
  }
  return shards.map((shard) => shard.sort());
}

export function mergeToolingShardTests(files, shardId) {
  if (!Number.isInteger(shardId) || shardId < 1 || shardId > mergeToolingShardCount) {
    throw new Error("merge tooling shard must be an integer from 1 to 4");
  }
  return partitionMergeToolingTests(files)[shardId - 1];
}
