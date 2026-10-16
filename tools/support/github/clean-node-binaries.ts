import { readdirSync, statSync, unlinkSync } from "node:fs";
import { join } from "node:path";
import { isEntrypoint, runEntrypoint } from "../command-runtime.ts";

/** Delete only top-level native outputs; preserve nested downloaded artifacts. */
export function cleanNodeBinaries(directories: readonly string[]): void {
  if (directories.length === 0)
    throw new Error("At least one native package directory is required");
  for (const directory of directories) {
    for (const name of readdirSync(directory)) {
      const file = join(directory, name);
      if (name.endsWith(".node") && statSync(file, { throwIfNoEntry: false })?.isFile())
        unlinkSync(file);
    }
  }
}

if (isEntrypoint(import.meta.url))
  runEntrypoint(() => {
    cleanNodeBinaries(process.argv.slice(2));
    return 0;
  });
