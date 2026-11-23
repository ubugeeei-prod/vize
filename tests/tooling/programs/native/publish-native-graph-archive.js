import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";

const archiveModule = process.argv[2];

const { createNativeGraphArchive } = await import(archiveModule);
const options = JSON.parse(process.argv[3]);
const path = process.argv[4];
options.configBytes = readFileSync(options.config);
const bytes = readFileSync(path);
const members = [
  { path, bytes: bytes.length, sha256: createHash("sha256").update(bytes).digest("hex") },
];
for (const phase of ["before", "after"]) {
  writeFileSync(options.stem + ".membership-" + phase + ".stdout.txt", path + "\n");
  writeFileSync(options.stem + ".membership-" + phase + ".stderr.txt", "");
}
const archive = createNativeGraphArchive(options);
archive.captureMembers(members);
archive.finish(members);
