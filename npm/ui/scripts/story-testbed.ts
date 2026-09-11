import path from "node:path";
import { pathToFileURL } from "node:url";

export { runUiStoryTestbedCli } from "./story-testbed-cli.ts";
import { runUiStoryTestbedCli } from "./story-testbed-cli.ts";

const isMain =
  process.argv[1] != null && pathToFileURL(path.resolve(process.argv[1])).href === import.meta.url;

if (isMain) {
  process.exitCode = runUiStoryTestbedCli(process.argv.slice(2));
}
