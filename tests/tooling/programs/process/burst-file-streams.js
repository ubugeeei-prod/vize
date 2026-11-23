import { fstatSync, writeSync } from "node:fs";

writeSync(1, String(fstatSync(1).isFile()) + " " + String(fstatSync(2).isFile()) + "\n");
for (let i = 0; i < 64; i++)
  writeSync(1, String(i).padStart(2, "0") + " " + "x".repeat(8190) + "\n");
writeSync(2, "burst stderr\n");
