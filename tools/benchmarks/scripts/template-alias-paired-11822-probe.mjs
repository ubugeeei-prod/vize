import { createRequire } from "node:module";
import { readFileSync, realpathSync, writeSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { createHash } from "node:crypto";

const packageRoot = process.argv[2];
const require = createRequire(join(packageRoot, "package.json"));
const binding = require("@vizejs/native");
const images = Object.keys(require.cache)
  .filter((path) => path.endsWith(".node"))
  .map((path) => {
    const file = realpathSync(path);
    return { path: file, sha256: createHash("sha256").update(readFileSync(file)).digest("hex") };
  });
const sharedObjects = process.report.getReport().sharedObjects.map((path) => {
  try {
    return realpathSync(path);
  } catch {
    return path;
  }
});
writeSync(
  2,
  `TEMPLATE_ALIAS_NATIVE_IDENTITY ${JSON.stringify({ images, sharedObjects, token: binding.getNativeBuildToken?.() ?? null, nativeEntrypoint: require.resolve("@vizejs/native") })}\n`,
);
const launcher = join(packageRoot, "bin/vize");
process.argv = [process.execPath, launcher, "--version"];
await import(pathToFileURL(launcher));
