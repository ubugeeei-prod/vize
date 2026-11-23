if (process.argv.at(-1) !== "forwarded") process.exit(9);
process.stdin.pipe(process.stdout);
