if (process.argv.includes("--listFilesOnly")) {
  process.stdout.write("coverage-started\n");
  setInterval(() => {}, 1000);
} else {
  process.stdout.write("src/App.vue(1,1): error TS2322: shared\n");
  process.exit(2);
}
