import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
const { observeSource, providerIdentities } = await import(pathToFileURL(process.argv[2]).href);
const before = await observeSource(readFileSync(process.argv[3], "utf8"), "original", [
  false,
  true,
]);
const after = await observeSource(readFileSync(process.argv[4], "utf8"), "formatted", [
  false,
  true,
]);
process.stdout.write(JSON.stringify({ before, after, providers: providerIdentities }) + "\n");
