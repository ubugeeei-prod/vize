import { execFileSync } from "node:child_process";
const dir = new URL(".", import.meta.url).pathname;
for (const r of process.argv.slice(2)) {
  execFileSync("node", [dir + "fetch-runs.mjs", `${dir}runs-${r}.tsv`, r], { stdio: "inherit" });
}
