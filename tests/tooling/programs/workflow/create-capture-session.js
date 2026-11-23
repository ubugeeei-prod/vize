import fs from "node:fs";
import path from "node:path";
fs.mkdtempSync(path.join(process.argv[2], `${process.pid}-1-`));
console.log(process.pid);
