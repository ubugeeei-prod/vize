if (process.argv.includes("--listFilesOnly")) {
  fs.writeSync(1, process.cwd() + "/src/App.vue\n");
  const filler = process.cwd() + "/src/generated-support-file.ts\n";
  for (let index = 0; index < 50_000; index++) fs.writeSync(1, filler);
  process.exit(0);
}

let output = "src/App.vue(1,1): error TS2322: shared\n";
const source = fs.readFileSync("src/App.vue", "utf8");
const marker = "__vize_typecheck_mutation_probe: string = 1";
if (source.includes(marker)) {
  const line = source.slice(0, source.indexOf(marker)).split(/\r?\n/).length;
  output += `src/App.vue(${line},1): error TS2322: Type 'number' is not assignable to type 'string'.\n`;
}
process.stdout.write(output);
process.exit(2);
