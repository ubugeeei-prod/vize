import fs from "node:fs";
import { fileURLToPath } from "node:url";
if (process.argv[2] === "--version") {
  fs.unlinkSync(fileURLToPath(import.meta.url));
  process.exit(0);
}
