import fs from "node:fs";
const commits = {
  base: process.env.COMPARISON_BASE,
  head: process.env.REQUESTED_HEAD,
  requested_base: process.env.REQUESTED_BASE,
};
fs.writeFileSync("measurements/commits.json", JSON.stringify(commits, null, 2));
