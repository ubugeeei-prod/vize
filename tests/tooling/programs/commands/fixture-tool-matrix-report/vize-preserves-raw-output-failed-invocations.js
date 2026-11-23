if (process.argv[2] === "--version") process.exit(0);
process.stdout.write("not json");
process.stderr.write("synthetic failure");
process.exit(2);
