import assert from "node:assert/strict";
import fs from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";

/** Follow the live output's footer; a previous build can leave orphaned maps. */
export async function readPublishedMap(filename) {
  const bundle = await fs.readFile(filename, "utf8");
  const footer =
    /(?:\/\/[#@]\s*sourceMappingURL=([^\r\n]+)|\/\*[#@]\s*sourceMappingURL=([^*]+)\*\/)\s*$/u.exec(
      bundle,
    );
  const reference = (footer?.[1] ?? footer?.[2])?.trim();
  if (!reference) return null;
  const url = new URL(reference, pathToFileURL(filename));
  assert.equal(url.protocol, "file:", "this host fixture must publish a local source map");
  const mapFile = fileURLToPath(url);
  return { bundle, reference, mapFile, bytes: await fs.readFile(mapFile) };
}
