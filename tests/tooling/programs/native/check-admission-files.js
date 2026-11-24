import { existsSync } from "node:fs";
import { join } from "node:path";
const directory = process.argv[2];
for (const file of ["stdout.raw", "stderr.raw", "spawn.json", "baseline.json"]) {
  if (!existsSync(join(directory, file))) process.exit(9);
}
process.stdout.write("full output");
process.exitCode = 2;
