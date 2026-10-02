// Re-run this move on a fresh base before applying dialect policy changes.
import { existsSync, mkdirSync, renameSync } from "node:fs";
import { dirname } from "node:path";

const oldPath = "davinci/vize_l1/src/dialect/vue3/surface/sink.rs";
const newPath = "davinci/vize_l1/src/dialect/vue/surface/sink.rs";
if (existsSync(oldPath) && !existsSync(newPath)) {
  mkdirSync(dirname(newPath), { recursive: true });
  renameSync(oldPath, newPath);
} else if (existsSync(oldPath) || !existsSync(newPath)) {
  throw new Error("Vue surface sink move has an ambiguous source/destination");
}
