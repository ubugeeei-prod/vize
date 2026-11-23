const coverage = process.argv.includes("--listFilesOnly");
process.stdout.write((coverage ? "coverage" : "baseline") + "-started\n");
await new Promise((resolve) => setTimeout(resolve, 800));
if (coverage) {
  console.log(process.cwd() + "/src/App.vue");
} else {
  process.stdout.write("src/App.vue(1,1): error TS2322: shared\n");
}
process.exit(coverage ? 0 : 2);
