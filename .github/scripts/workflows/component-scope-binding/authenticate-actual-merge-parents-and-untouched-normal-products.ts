import fs from "node:fs";
const p = process.env.EVIDENCE;
const b = Object.fromEntries(
  ["base", "head"].map((k) => [
    k,
    JSON.parse(fs.readFileSync(p + "/" + k + "-normal-binary.json", "utf8")),
  ]),
);
if (
  JSON.parse(fs.readFileSync(p + "/provenance.json", "utf8")).observerPurpose !== "correctness" &&
  b.base.sha256 === b.head.sha256
)
  throw Error("identical binaries");
fs.writeFileSync(p + "/binaries.json", JSON.stringify(b, null, 2));
