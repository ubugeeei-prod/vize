export const mergeToolingShardCount = 4;

// The artifact uploader runs in shard 1, alongside the only test that writes
// its formatter API corpus directory.
export const formatterEvidenceTest = "tests/tooling/differential-formatter-api-execution.test.mjs";

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
  shards[0].push(formatterEvidenceTest);
  let index = 0;
  for (const file of files) {
    if (file === formatterEvidenceTest) continue;
    shards[(index + 1) % mergeToolingShardCount].push(file);
    index += 1;
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
