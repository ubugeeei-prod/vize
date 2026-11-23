const fs = require("node:fs");
const path = require("node:path");
const cache = process.env.XDG_CACHE_HOME;
fs.appendFileSync(
  path.join(__dirname, "commands.jsonl"),
  JSON.stringify({
    cache,
    empty: cache !== undefined && fs.readdirSync(cache).length === 0,
    threads: process.env.RAYON_NUM_THREADS,
    bench: process.env.VIZE_BENCH,
  }) + "\n",
);
if (cache) fs.writeFileSync(path.join(cache, "tsconfig.tsbuildinfo"), "checked");
console.log(JSON.stringify({ files: [], errorCount: 0, warningCount: 0 }));
console.error("Type checked 1 files in 1s (collect: 0.1s, imports: 0.2s, gen: 0.3s, corsa: 0.4s)");
