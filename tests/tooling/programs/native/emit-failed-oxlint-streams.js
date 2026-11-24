process.stdout.write(Buffer.from([0, 255, 10]));
process.stderr.write("retained stderr");
process.exit(23);
