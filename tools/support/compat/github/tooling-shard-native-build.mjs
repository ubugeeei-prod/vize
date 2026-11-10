import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { toolingShardNativeBuildRequired } from "./tooling-test-shards.ts";

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [planPath, shard] = process.argv.slice(2);
  if (!planPath || !shard) throw new Error("expected tooling plan and shard coordinates");
  const plan = JSON.parse(readFileSync(planPath, "utf8"));
  process.stdout.write(`${toolingShardNativeBuildRequired(plan, shard)}\n`);
}
