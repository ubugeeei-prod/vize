const [prefix, suffix] = process.argv.slice(2);
process.stdout.write(prefix + "f".repeat(300000) + suffix);
process.stderr.write("private argument-secret");
