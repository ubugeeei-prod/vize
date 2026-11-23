const [stdout, stderr, cwd] = process.argv.slice(2);
process.stdout.write(Buffer.from(stdout, "base64"));
process.stderr.write(Buffer.from(stderr, "base64"));
process.exitCode =
  process.env.LANG === "C" && process.env.LC_ALL === "C" && process.cwd() === cwd ? 23 : 24;
