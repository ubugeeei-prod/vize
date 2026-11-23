const args = process.argv.slice(2);
if (args.join(" ") !== "run --profile ci -p vize_test_runner --bin coverage") process.exit(99);
process.stdout.write(
  [
    "Vue Compiler Coverage Report",
    "----------------------------",
    "TOTAL:  502/594 (84.5%)",
    "92 tests failed",
  ].join("\n"),
);
process.exit(7);
