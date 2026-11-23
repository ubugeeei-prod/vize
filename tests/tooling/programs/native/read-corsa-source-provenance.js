import { execFileSync } from "node:child_process";

const sourceModule = process.argv[2];

const { sourceProvenance } = await import(sourceModule);
const actual = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
process.stdout.write(JSON.stringify(sourceProvenance(actual)));
