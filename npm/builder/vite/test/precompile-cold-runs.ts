import { coldRun, manifestFiles } from "../src/test/precompile-cache-project.ts";

const [root] = process.argv.slice(2);
const runs = [];
for (let run = 0; run < 2; run++) {
  const warnings: string[] = [];
  console.warn = (...args) => {
    warnings.push(args.map(String).join(" "));
  };
  const { log } = await coldRun(root);
  runs.push({ log, warnings, manifests: manifestFiles(root).length });
}
process.stdout.write(JSON.stringify(runs));
