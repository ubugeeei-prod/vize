const { rows } = JSON.parse(require("fs").readFileSync("compile-compare.json", "utf8"));
let ok = true;
// An approved TS-source output change sets COMPILE_BENCH_TS_OUTPUT_MAY_CHANGE; JS-source output must never change.
const keys =
  process.env.COMPILE_BENCH_TS_OUTPUT_MAY_CHANGE === "true" ? ["outputSha256Js"] : ["outputSha256"];
for (const mode of ["loop1t", "batch1t", "batchmax"]) {
  for (const key of keys) {
    const hashes = new Set(rows.filter((row) => row.mode === mode).map((row) => row[key]));
    if (hashes.size !== 1) {
      ok = false;
      console.error(`${mode}: ${key} differs between base and head`);
    }
  }
}
if (!ok) process.exit(1);
