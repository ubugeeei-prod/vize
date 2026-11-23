const fs = require("node:fs");
const [binding, list] = process.argv.slice(2);
const native = require(binding);
const sources = fs
  .readFileSync(list, "utf8")
  .split("\n")
  .filter(Boolean)
  .map((file) => [file, fs.readFileSync(file, "utf8")]);
const started = process.hrtime.bigint();
let bytes = 0;
for (let pass = 0; pass < 5; pass += 1) {
  for (const [filename, source] of sources)
    bytes += native.compileSfc(source, { filename }).code.length;
}
process.stdout.write(String(Number(process.hrtime.bigint() - started) / 1e9) + " " + bytes);
