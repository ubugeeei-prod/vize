/** Fresh-process entry point for one Musea benchmark measurement. */

import { measureMuseaInProcess } from "./musea.mjs";

try {
  const options = JSON.parse(process.argv[2] ?? "{}");
  const data = await measureMuseaInProcess(options);
  process.stdout.write(`${JSON.stringify(data)}\n`);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
