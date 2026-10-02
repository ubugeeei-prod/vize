// Restore only the proven original runtime source for an isolated Actions capture.
import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
const revision = process.argv[2];
if (!revision) throw new Error("Supply the reviewed pure source-copy commit");
const file = "vendor/oxc_parser/src/lexer/trivia_builder.rs";
const read = spawnSync("git", ["show", `${revision}:${file}`], { maxBuffer: 1024 * 1024 });
if (read.error) throw read.error;
if (read.status !== 0) throw new Error(read.stderr.toString());
const oid = spawnSync("git", ["hash-object", "--stdin"], { input: read.stdout, encoding: "utf8" });
if (oid.error) throw oid.error;
// The exact official fc702c1 trivia_builder blob, independent of candidate bytes.
if (oid.status !== 0 || oid.stdout.trim() !== "518ccd0ee513138d5f7860fc99e92f51496c6dc8")
  throw new Error("Original blob identity mismatch");
writeFileSync(file, read.stdout);
console.log(`Restored original ${file} blob ${oid.stdout.trim()} from ${revision}`);
