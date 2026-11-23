import fs from "node:fs";
import path from "node:path";
if (process.argv[2] === "--version") process.exit(0);
const output = process.argv[process.argv.indexOf("--output") + 1];
fs.mkdirSync(output, { recursive: true });
fs.writeFileSync(
  path.join(output, "vize-tool-matrix-test.json"),
  JSON.stringify({
    filename: "vize-tool-matrix-test.vue",
    code: "export default {}",
    css: null,
    errors: [],
    warnings: ["synthetic warning"],
    script_lang: "js",
    macro_artifacts: [],
  }) + "\n",
);
process.stdout.write("Built: vize-tool-matrix-test.vue\n");
