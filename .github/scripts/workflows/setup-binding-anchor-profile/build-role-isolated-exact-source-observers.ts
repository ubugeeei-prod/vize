import fs from "node:fs";
const p = process.env.RUNNER_TEMP + "/binding-evidence/";
const x = Object.fromEntries(
  ["base", "head"].map((k) => [k, JSON.parse(fs.readFileSync(p + k + "-binary.json", "utf8"))]),
);
if (x.base.sha256 === x.head.sha256) throw Error("identical binaries");
fs.writeFileSync(p + "binaries.json", JSON.stringify(x, null, 2));
