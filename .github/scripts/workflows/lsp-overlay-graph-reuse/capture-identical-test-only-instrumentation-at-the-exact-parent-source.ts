import fs from "node:fs";
const rows = fs
  .readFileSync(process.env.OBSERVATION_ROOT + "/before-build.jsonl", "utf8")
  .trim()
  .split("\n")
  .map((line) => JSON.parse(line));
const executable = rows.findLast(
  (row) =>
    row.reason === "compiler-artifact" &&
    row.target.name === "vize_canon" &&
    row.profile.test &&
    row.executable,
)?.executable;
if (!executable) throw new Error("missing exact parent test executable");
fs.writeFileSync(process.env.OBSERVATION_ROOT + "/before-executable.txt", executable);
