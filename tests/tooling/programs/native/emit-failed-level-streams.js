process.stdout.write(Buffer.from([0, 255, 10]));
process.stderr.write("whole stderr\n");
process.exit(7);
