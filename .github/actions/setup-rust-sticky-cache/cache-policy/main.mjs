import path from "node:path";
import { pathToFileURL } from "node:url";

const entry = path.join(process.env.VIZE_CACHE_ACTION_PATH, "cache-policy.ts");
const { runCachePolicy } = await import(pathToFileURL(entry).href);
runCachePolicy(process.env.VIZE_CACHE_MODE);
