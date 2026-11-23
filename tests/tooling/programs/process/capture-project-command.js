import fs from "node:fs";

process.stdout.write(fs.readFileSync(0));
process.stdout.write(
  JSON.stringify([process.argv.slice(2), process.cwd(), process.env.VIZE_CAPTURE]),
);
process.stderr.write(Buffer.from([0, 255, 228, 184, 173]));
process.exitCode = 7;
