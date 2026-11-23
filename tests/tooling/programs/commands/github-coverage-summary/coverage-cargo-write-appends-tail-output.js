const args = process.argv.slice(2);
if (args.join(" ") !== "run --profile ci -p vize_test_runner --bin coverage") process.exit(99);
process.stdout.write(
  [
    "Coverage report",
    "line-1",
    "line-2",
    "line-3",
    "line-4",
    "line-5",
    "line-6",
    "line-7",
    "line-8",
  ].join("\n"),
);
