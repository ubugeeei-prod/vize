import assert from "node:assert/strict";
import path from "node:path";
import { pathToFileURL } from "node:url";

const actionPath = process.env.VIZE_CACHE_ACTION_PATH;
assert.ok(actionPath, "the cache action must supply its source directory");
const entry = path.join(actionPath, "cache-policy.ts");
const { runCachePolicy } = (await import(
  pathToFileURL(entry).href
)) as typeof import("../cache-policy.ts");
runCachePolicy(process.env.VIZE_CACHE_MODE);
